# Rhai Plugin Scripting Guide

## Introduction

This is user guide on how to create a "bridge" between protocol specific SW and Blockvisor (aka BV),
by implementing Babel Plugin.
Babel Plugin translates BV protocol agnostic interface (aka Babel API) into protocol specific calls.

Currently, Babel Plugin can be implemented using Rhai scripting language.
See [The Rhai Book](https://rhai.rs/book) for more details on Rhai language itself.

### Core of All Babel Plugins

1. `fn plugin_config()` - The easiest way to describe node initialization and background services is to define this function
as described in [plugin_config](#plugin_config) chapter, to use default implementation for `init()` and `upload()` functions.
2. Dynamic node configuration is accessible via `node_params()` function. This can and should be used in order
to change the behavior of the script according to the node configuration.
3. While protocol data are typically huge, BV provides built-in facility for efficient uploading and downloading
protocol archives.
<br>See [Protocol Data Archives](#protocol-data-archives) chapter for more details.

## Plugin Interface

To add some specific protocol support Babel Plugin must tell BV how to use protocol specific tools inside the image,
to properly setup and maintain protocol node. This chapter describe interface that needs to be implemented by script
for this purpose.

### plugin_config

`plugin_config` function is a convenient way to specify following things:
 - node initialization process
 - auxiliary services to run in background (e.g. for monitoring)
 - download job configuration
 - alternative download command
 - post-download actions
 - configuration for background services, to be run on the node
 - pre-upload actions
 - upload job configuration

See [example](examples/plugin_config.rhai) with comments for more details.

### Functions that SHALL be implemented by Plugin

Functions listed below are required by BV to work properly.

- `init` - Main entrypoint where everything starts. Normally this function is called only once, when node is started first time
  and after node upgrade. It may also be called more than once if it fails and node needs recovery.
  <br>NOTE: For actions that are required to be run once and only once, use jobs with `one-time` flag set to `true`,
  <br>It is the place where protocol services running in background should be described as so called `jobs`.
  Also, any other one-off operations can be done here (e.g. initializing stuff, or downloading protocol data archives).
  <br>See [Engine Interface](#engine-interface) and [Background Jobs](#background-jobs) chapter for more details on how to start jobs.
  <br> BV provide default implementation if [plugin_config](#plugin_config) function is defined.
  Default implementation run all init commands and start init jobs according to config,
  then try to start build-in download job, if fail then fallback to alternative download (if provided),
  finally starts configured services.

  See [minimalistic example](examples/init_minimal.rhai) or go to [Protocol Data Archives](#protocol-data-archives) chapter for example including download step)
- `protocol_status()` - Returns protocol application status in form of following structure.
```rust
{
  // Arbitrary application state name.
  state: String,
  // Node health behind the state. One of healthy, neutral, unhealthy.
  health: NodeHealth,
}
```
e.g. `#{state: "broadcasting", health: "healthy"}`

### Functions that SHOULD be implemented by Plugin

Functions listed below are used BV for collecting metrics and status monitoring.
It is recommended to implement all of them, if feasible.

- `height()` - Returns the height of the blockchain (in blocks).
- `block_age()` - Returns the block age of the blockchain (in seconds).
- `address()` - The address of the node. The meaning of this varies from protocol to protocol.
  <br>**Example**: _/p2p/11Uxv9YpMpXvLf8ZyvGWBdbgq3BXv8z1pra1LBqkRS5wmTEHNW3_
- `name()` - Returns the name of the node. This is usually some random generated name that you may use
to recognise the node, but the purpose may vary per protocol.
  <br>**Example**: _chilly-peach-kangaroo_
- `consensus()` - Returns `bool` whether this node is in consensus or not.
- `upload()` - Upload protocol data snapshot to cloud storage, so it can be quickly reused by newly created nodes.
  <br> BV provide default implementation if [plugin_config](#plugin_config) function is defined.
  Default implementation stop all services, start upload job according to config, and then start services again.

See [example](examples/polygon_functions.rhai) of functions implemented for Polygon.

### Functions that MAY be implemented by Plugin

Plugin may additionally implement an arbitrary custom function that will be then accessible
via BV CLI interface. The only limitation is that custom functions must take only one "string"
argument (it may be more complex structure, but e.g. serialized as JSON) and must return "string" as well.

**Example:**
```
fn some_custom_function(arg) {
    let input = parse_json(arg);
    let output = #{
        result: input.value * 2
    };
    output.to_json();
}
```

### Default implementations

#### Default `init`

If no `init` function is defined, but only `plugin_config`, then `default_init` is used which does following:
- render config files from `plugin_config`
- start auxiliary services from `plugin_config`
- run `init` commands
- start `init` jobs
- start `download` job (if download is not completed yet and not `dev_node`)
- if data archive is not available and `alternative_download` is defined then start it
- start `post_download` jobs
- if neither `download` nor `alternative_download` is provided, start `cold_init` job
- start services
- schedule tasks defined in `plugin_config`

#### Default `upload`

If no `upload` function is defined, but only `plugin_config`, then `default_upload` is used which does following:
- stop services that `use_protocol_data`
- run `pre_upload` commands
- start `pre_upload` jobs
- start `upload` job
- run `post_upload` commands
- start `post_upload` jobs
- start previously stopped services

## Engine Interface

To make implementation of Babel Plugin interface possible, BV provides following functions to Rhai script.

- `create_job(job_name, job_config)` - Create background job with unique name. See 'Backgound Jobs' section for more details.
- `start_job(job_name, job_config)` - Create and immediately start background job with unique name.
- `start_job(job_name)` - Start previously created background job with given name.
- `stop_job(job_name)` - Stop background job with given unique name if running.
- `job_status(job_name)` - Get background job status by unique name.
  <br>**Possible return values**: _pending_, _running_, _stopped_, _finished{exit_code, message}_
- `run_jrpc(request)` - Execute Jrpc request to the current node and return [HttpResponse](#httpresponse) (with default 15s timeout). Request must have following structure:
```rust
{
  // This is the host for the JSON rpc request.
  host: String,
  // The name of the jRPC method that we are going to call into.
  method: String,
  // [optional] Params structure in form of Dynamic object that is serializable into JSON.
  params: Dynamic,
  // [optional] Extra HTTP headers to be added to request.
  headers: HeadersMap
}
```
- `run_jrpc(request, timeout)` - Same as above, but with custom request timeout (in seconds).
- `run_rest(request)` - Execute a Rest request to the current node and return [HttpResponse](#httpresponse) (with default 15s timeout). Request must have following structure:
```rust
{
  // This is the url of the rest endpoint.
  url: String,
  // [optional] Extra HTTP headers to be added to request.
  headers: HeadersMap
}
```
- `run_rest(request, timeout)` - Same as above, but with custom request timeout (in seconds).
- `run_sh(body)` - Run Sh script on the node and return [ShResponse](#shresponse) (with default 15s timeout).
- `run_sh(body, timeout)` - Same as above, but with custom execution timeout (in seconds).
- `parse_hex(hex)` - Convert `0x` hex string into decimal number.
- `parse_json(json)` - Parse json string into a Rhai object.
- `system_time()` - Get system time in seconds since UNIX EPOCH.
- `parse_rfc3339(date_time)` - Parse an RFC 3339 date-and-time string into seconds since UNIX EPOCH.
- `parse_rfc2822(date_time)` - Parse an RFC 2822 date-and-time string into seconds since UNIX EPOCH.
- `parse_time(date_time, format)` - Parse date-and-time string into seconds since UNIX EPOCH, with respect to given format.
  See [chrono::format::strftime](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) for format details.
- `sanitize_sh_param(param)` - Allowing people to substitute arbitrary data into sh-commands is unsafe.
  Call this function over each value before passing it to `run_sh`. This function is deliberately more
  restrictive than needed; it just filters out each character that is not a number or a
  string or absolutely needed to form an url or json file.
- `render_template(template, destination, params)` - This function renders configuration template with provided `params`.
  See [Tera Docs](https://keats.github.io/tera/docs/#templates) for details on templating syntax.
  `params` is expected to be Map object serializable to JSON.
  It assumes that file pointed by `template` argument exists.
  File pointed by `destination` path will be overwritten if exists.
- `node_params()` - Get node params as key-value map.
- `node_env()` - Get node environment/context metadata, see [NodeEnv](#nodeenv).
- `add_task(task_name, schedule, function_name, function_param)` - Schedule Rhai function with given `function_name` and `function_param` (optional), according to cron-compatible `schedule` string.
- `delete_task(task_name)` - Delete previously scheduled task.
- `save_data(value)` - Save plugin data to persistent storage. It takes string as argument. `to_json` can be used for more complex structures.
- `load_data()` - Load plugin data from persistent storage. Returns string (as it was passed to `save_data`).
- `put_secret(key, value)` - Put plugin secret data to encrypted cloud storage. It takes string `key` and BLOB `value` as arguments. Use `to_blob` if you want to store simple string.
- `get_secret(key)` - Get plugin secret data from encrypted cloud storage. It takes string `key` and returns BLOB (as it was passed to `put_secret`). Use `as_string` if you expect simple string.
- `file_write(path, content)` - Write binary `content` (BLOB) into node filesystem. Use `to_blob` if you want to store simple string.
- `file_read(path)` - Read binary content (BLOB) from node filesystem. Use `as_string` if you expect simple string.
  This is the path, where protocol data archives are downloaded to, and where are uploaded from.

### HttpResponse

```rust
struct HttpResponse {
  status_code: i32,
  body: String,
}
```
Above structure provide following helper functions:
- `expect(expected_code)` - Check if `status_code` match expected one and then return `body` parsed as json.
- `expect(check)` - If provided `check` function return `true`, then return `body` parsed as json (e.g. `http_resp.expect(|code| code >= 200).json_field`).

### ShResponse

```rust
struct ShResponse{
  exit_code: i32,
  stdout: String,
  stderr: String,
}
```
Above structure provide following helper function:
- `unwrap()` - Check if `exit_code` is 0 and then return `stdout`.

### NodeEnv
```rust
pub struct NodeEnv {
    /// Node id.
    pub node_id: String,
    /// Node name.
    pub node_name: String,
    /// Node version.
    pub node_version: String,
    /// Node protocol.
    pub node_protocol: String,
    /// Node variant.
    pub node_variant: String,
    /// Node IP.
    pub node_ip: String,
    /// Node gateway IP.
    pub node_gateway: String,
    /// Indicate if node run in dev mode.
    pub dev_mode: bool,
    /// Host id.
    pub bv_host_id: String,
    /// Host name.
    pub bv_host_name: String,
    /// API url used by host.
    pub bv_api_url: String,
    /// Organisation id to which node belongs to.
    pub org_id: String,
    /// Absolute path to directory where data drive is mounted.
    pub data_mount_point: String,
    /// Absolute path to directory where protocol data are stored.
    pub protocol_data_path: String,
}
```

### Background Jobs

Background job is a way to asynchronously run long-running tasks. Currently, three types of tasks are supported:
1. `run_sh` - arbitrary long-running shell script.
2. `download` - download data (e.g. previously archived protocol data, to speedup init process).
3. `upload` - upload data (e.g. archive protocol data).
In particular, it can be used to define protocol service(s) i.e. background process(es) that are automatically started
with the node.

Each background job has its __unique__ name and configuration structure described by following [example](examples/jobs.rhai).

Once job has been created, other functions in the script may fetch for its state with `job_status(job_name)`, start it
or stop on demand with `start_job(job_name)`/`stop_job(job_name)`.

### Logging

Rhai provides `print` and `debug` functions to simply print into stdout. To make plugin logs consistent with BV logs,
it is recommended to use following log functions:

- `debug` (overridden built-in `debug` implementation)
- `info`
- `warn`
- `error`

**Example:**
```
fn some_function() {
    info("some important step logged with INFO level");
}
```
Output:
```
<time>  INFO babel_api::rhai_plugin: node_id: <node_id>|some important step logged with INFO level
```

## Protocol Data Archives

### Uploading Data Archives

To upload protocol data archive on remote object storage, following steps are needed:
1. Stop all protocol synchronization. Protocol data should not be modified while uploading.
2. Start upload job.
3. Once upload is finished protocol synchronization can be turned on again.

Above steps can be implemented in any custom rhai function (typically named `upload`, but name can be arbitrary).
<br>See [example](examples/custom_download_upload.rhai) for details. See also example in [Background Jobs](#background-jobs) chapter for all possible
upload job config options.

To trigger upload, simply call `bv node run upload`.

Define [plugin_config](#plugin_config) function to use default implementation for `upload`.

### Downloading Data Archives

Once protocol data archive is available on remote object storage, it can be used to speedup new nodes setup.

To use default implementation define [plugin_config](#plugin_config) function. Otherwise
see [example](examples/custom_download_upload.rhai) of custom `init()` function.

Typically, download job is started in `init()` before other protocol services are started. This is achieved by job
`needs` configuration. See also example in [Background Jobs](#background-jobs) chapter for all possible
download job config options.

## Common Use Cases

### Add Custom HTTP Headers to JRPC and REST Requests

`request` parameter in `run_jrpc` and `run_rest` functions has optional `headers` field,
that can be used to add custom HTTP headers to the request.

**Example:**
```
let data = #{host: "localhost:8154",
             method: "getBlockByNumber",
             headers: [["content-type", "application/json"]]
            };
run_jrpc(data);
```

### Add custom parameters to JRPC Requests

`request` parameter in `run_jrpc` has optional `params` field,
that can be used to add params to the request.

**Example:**
```
let data = #{host: "localhost:8154",
             method: "getBlockByNumber",
             params: #{"chain": "x"},
             headers: [["content-type", "application/json"]]
            };
run_jrpc(data);
```

### Handling JRPC Output

`run_jrpc` function return raw http response. Use `expect` method or `parse_json` on response body, to easily access json fields.

**Example:**
```
const API_HOST = "http://localhost:4467/";

fn block_age() {
    let resp = run_jrpc(#{host: global::API_HOST, method: "info_block_age"}).expect(200);
    resp.result.block_age
}
```

### Output Mapping

Rhai language has convenient `switch` statement, which is very similar to Rust `match`.
Hence, it is a good candidate for output mapping.

**Example:**
```
fn status() {
    let stdout = run_sh("get_status").unwrap();
    let status = switch stdout {
        "0" => "synced",
        _ => "syncing",
    };
    status
}
```

### Running Sh Script

**IMPORTANT:** Whenever user input (`node_params()` in particular) is passed to sh script,
it should be sanitized with `sanitize_sh_param()`, to avoid malicious code injection.

**Example:**
```
fn custom_function(arg) {
  let user_param = sanitize_sh_param(node_params()["USER-INPUT"]);
  let res = run_sh("echo '" + user_param + "'");
  if res.exit_code == 0 {
    res.stdout
  } else {
    res.stderr
  }
}
```

### Save and Load Plugin Data

Plugin specific data can be persisted with `save/load_data` functions. It takes/returns simple string, but e.g. json can
be easily used for more complex structures.

**Example:**
```
fn custom_function(arg) {
    let info;
    try {
        info = parse_json(load_data());
    } catch {
        info = #{
            some_field: "initial value",
            some_other_field: false,
        };
        save_data(info.to_json());
    };
    info.some_field
}
```

### Put and Get Plugin Secrets

Plugin specific secrets (e.g. node key or config) can be persisted in encrypted cloud storage with `put/get_secret` functions. It takes/returns BLOB for given key.

**Example:**
```
fn custom_function(arg) {
    let peer_key;
    let peer_id;
    try {
        peer_key = get_secret("key");
        peer_id = get_secret("id").as_string();
    } (err) catch {
        if err == "not_found" {
            peer_key = file_read("/blockjoy/peer.key");
            peer_id = file_read("/blockjoy/peer.id").as_string();
            put_secret("key", peer_key);
            put_secret("id", peer_id.to_blob());
        } else {
           throw err;
        }
    };
    // do stuff with peer_key and peer_id
    arg
}
```

### Rendering Configuration File

Some protocols may require configuration files to be filled up with some user data. For that `render_template` can be used.

At first, configuration template file must be included into the image.

**Example:**
```
first_parameter: {{ param1 }}
second_parameter: {{param2}}
optional_parameter: {% if param3 %}{{ param3 }}{% else %}None{% endif %}
array: [{% for p in aParam %}{{p}},{% endfor %}]
```

Then it can be rendered e.g. during node initialization:

**Example:**
```
fn init(keys) {
    let params = #{
        param1: "value1",
        param2: 2,
        aParam: ["a", "bb", "ccc"],
    };
    render_template("/etc/protocol.config.template", "/etc/protocol.config", params);
}
```

After `init` is called, following file should be created:

```
first_parameter: "value1"
second_parameter: 2
optional_parameter: None
array: ["a","bb","ccc",]
```

## Dynamic Configuration

Some protocols may require dynamic configuration parameters.

The Blockvisor system uses two key files to define and use custom dynamic values:

  1. `babel.yaml` - Defines the properties/parameters available for a protocol image
  2. `main.rhai` - Accesses and uses these properties to configure the node runtime
### Steps to Add Custom Properties
  1. Adding Custom Properties in `babel.yaml`

  Properties are defined in the properties section of `babel.yaml`. Here are the supported property types:

```
  Text Input Properties
  properties:
    - key: rpc-port                    # Used to access via node_params()["rpc-port"]
      name: RPC Port                   # Display name in UI
      description: Port for RPC server # Optional description
      dynamic_value: true              # Can be changed after deployment
      default_value: "8545"           # Default value
      ui_type: !text
        new_archive: false            # Whether this requires new protocol data
        add_cpu: 0                    # Additional CPU cores needed
        add_memory_mb: 512            # Additional memory in MB
        add_disk_gb: 1                # Additional disk space in GB

  Switch/Toggle Properties

    - key: enable-metrics
      name: Enable Metrics
      description: Enable prometheus metrics collection
      dynamic_value: true
      default_value: "false"
      ui_type: !switch
        on:
          add_cpu: 1
          add_memory_mb: 256
          add_disk_gb: 0
        off: null

  Enum/Dropdown Properties

    - key: sync-mode
      name: Sync Mode
      description: Blockchain synchronization mode
      dynamic_value: true
      default_value: fast
      ui_type: !enum
        - value: fast
          name: Fast Sync
          impact: null
        - value: full
          name: Full Sync
          impact:
            add_cpu: 2
            add_memory_mb: 1024
            add_disk_gb: 10
```
  2. Using Custom Properties in main.rhai

  Accessing Property Values

  Properties are accessed in Rhai scripts using the node_params() function:
```
  // Get a property value
  let rpc_port = node_params()["rpc-port"];
  let enable_metrics = node_params()["enable-metrics"];
  let sync_mode = node_params()["sync-mode"];
```
  Using Properties in Service Configuration
```
  fn plugin_config() {
    #{
      services: [
        #{
          name: "blockchain_node",
          // Use properties in command line arguments
          run_sh: `/usr/bin/node --rpc-port=${node_params()["rpc-port"]} --sync-mode=${node_params()["sync-mode"]}` +
                  if node_params()["enable-metrics"] == "true" { " --metrics" } else { "" },
          restart_config: #{
            backoff_timeout_ms: 60000,
            backoff_base_ms: 1000,
          },
        }
      ]
    }
  }
```
  Safe Parameter Handling
```
  fn plugin_config() {
    // Safely handle parameters with defaults
    let rpc_port = node_params()["rpc-port"] || "8545";
    let metrics_enabled = node_params()["enable-metrics"] == "true";

    #{
      init: #{
        commands: [
          // Use in shell commands (be sure to sanitize!)
          `echo "Starting with RPC on port ${sanitize_sh_param(rpc_port)}"`,
        ]
      },
      services: [
        #{
          name: "main_service",
          run_sh: build_command(rpc_port, metrics_enabled),
        }
      ]
    }
  }

  fn build_command(port, metrics) {
    let base_cmd = `/usr/bin/service --port=${port}`;
    if metrics {
      base_cmd += " --enable-metrics";
    }
    base_cmd
  }
```
  Configuration File Templates

  Properties can also be used to generate configuration files:
```
  fn plugin_config() {
    #{
      config_files: [
        #{
          template: "/var/lib/babel/templates/node.conf.template",
          destination: "/etc/node.conf",
          params: #{
            rpc_port: node_params()["rpc-port"],
            metrics_enabled: node_params()["enable-metrics"] == "true",
            sync_mode: node_params()["sync-mode"],
            node_name: node_env().node_name,
          },
        },
      ]
    }
  }
```
  3. Complete Example

  Here's a complete example showing both files working together:

  `babel.yaml`
```
  version: 0.0.1
  protocol_key: my-protocol
  container_uri: docker://myorg/my-protocol/mainnet/0.0.1

  properties:
    - key: rpc-port
      name: RPC Port
      description: Port for JSON-RPC server
      dynamic_value: true
      default_value: "8545"
      ui_type: !text
        add_memory_mb: 128

    - key: enable-debug
      name: Enable Debug Logging
      description: Enable detailed debug logs
      dynamic_value: true
      default_value: "false"
      ui_type: !switch
        on:
          add_cpu: 1
          add_memory_mb: 512

    - key: network-mode
      name: Network Mode
      description: Which network to connect to
      dynamic_value: false  # Can't change after deployment
      default_value: mainnet
      ui_type: !enum
        - value: mainnet
          name: Mainnet
          impact: null
        - value: testnet
          name: Testnet
          impact:
            add_memory_mb: -256

  variants:
    - key: mainnet
      min_cpu: 2
      min_memory_mb: 4096
      min_disk_gb: 100
```
  `main.rhai`
```
  fn plugin_config() {
    let rpc_port = node_params()["rpc-port"] || "8545";
    let debug_enabled = node_params()["enable-debug"] == "true";
    let network = node_params()["network-mode"] || "mainnet";

    #{
      init: #{
        commands: [
          `mkdir -p /opt/protocol/data`,
          if debug_enabled { `touch /opt/protocol/debug.log` } else { `` },
        ]
      },

      services: [
        #{
          name: "protocol_node",
          run_sh: `/usr/bin/protocol-node` +
                 ` --rpc-port=${rpc_port}` +
                 ` --network=${network}` +
                 if debug_enabled { ` --debug` } else { `` },
          restart_config: #{
            backoff_timeout_ms: 60000,
            backoff_base_ms: 1000,
          },
          use_protocol_data: true,
        }
      ]
    }
  }

  fn protocol_status() {
    #{state: "broadcasting", health: "healthy"}
  }
```
  4. Key Points

  - Property keys must be lower-kebab-case (e.g., rpc-port, not rpcPort)
  - Access properties via node_params()["property-key"]
  - Always handle missing/default values safely
  - Use sanitize_sh_param() when using properties in shell commands
  - Properties with dynamic_value: true can be changed post-deployment
  - Properties with new_archive: true require new protocol data when changed
  - Resource impacts (CPU/memory/disk) are automatically calculated and displayed in the UI

  This architecture allows you to create flexible, configurable node images where users can customize behavior through the admin interface, and those settings are automatically passed to your Rhai scripts
   for runtime configuration.

## Unit Testing

For now, the easiest way to automatically test Rhai script is to write Rust unit test.

See [test_examples.rs](tests/test_examples.rs) for example.
