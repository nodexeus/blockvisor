# BlockVisor

The service that runs on the host systems and is responsible for provisioning and managing one or more protocols on a single server.

## How to release a new version
1. Make sure you have installed:
   - `git-conventional-commits`: `nvm install node; npm install --global git-conventional-commits`
   - `cargo-release`: `cargo install cargo-release`
2. Run `cargo release --execute $(git-conventional-commits version)` 
3. CI `publish` workflow will then build a bundle and create a new GH release
4. Bundle is automatically deployed on DEV environment. When bundle is tested and ready to promote
on PROD environment, use `make promote-prod` (requires `AWS_ACCOUNT_ID, AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, AWS_REGION` env variable set). 

## Host Setup

See [BlockVisor Host Setup Guide](host_setup_guide.md) for more details.

Published version of above guide with `bvup` tool can be found [here](https://github.com/blockjoy/bv-host-setup/releases).

## Babel Plugins

BV is protocol agnostic system that uses plugin system to add support for specific protocols.

So-called Babel Plugin, that translates BV protocol agnostic interface (aka Babel API) into protocol specific calls,
always comes together with node image.

See [Node Image Builder Guide](node_image_builder_guide.md) for more details on how to
add new protocol support to Blockvisor.

## API proto files

API proto files are stored in [separate repository](https://github.com/blockjoy/api-proto).

Note that [git submodules](https://github.blog/2016-02-01-working-with-submodules/) are used to bring the protos to this project.

```
git submodule update --init --recursive
```

## Log Levels Policy
- `error` - internal BV error (potential bug) or nonrecoverable error that requires manual actions;
error should rise alert
- `warn` - abnormal events that BV is capable to handle, e.g. networking issues, node recovery;
may be caused by external errors, but BV should recover when external system get back to normal
- `info` - main actions with minimum context, e.g. node created;
avoid for frequently recurring actions like sending node status
- `debug` - Detailed actions flow with variables, include recurring actions like sending node status;
used during debugging issues, not printed by default
- `trace` - debug messages used on development phase by devs

## Important Paths
### Host
- `/opt/blockvisor/blacklist` bundle versions that failed to be installed
- `/opt/blockvisor/current` symlink to current `<version>`
- `/opt/blockvisor/<version>/` whole bundle
- `/etc/blockvisor.json` generated by `bvup <PROVISION_TOKEN>`, but can be later modified
- `/etc/systemd/system/blockvisor.service`
- `/var/lib/blockvisor/nodes/state.json` nodes_manager state persistence
- `/var/lib/blockvisor/nodes/<uuid>/` node specific data
- `/var/lib/blockvisor/nodes/<uuid>/state.json` node state persistence
- `/var/lib/blockvisor/nodes/<uuid>/plugin.data` Babel plugin data persistence (see load_data/save_data functions in [RHAI plugin scripting guide](babel_api/rhai_plugin_guide.md))
- `/var/lib/blockvisor/nodes/<uuid>/rootfs/` node rootfs (from `os.img`)
- `/var/lib/blockvisor/nodes/<uuid>/data/` protocol data dir, bind to node `/blockjoy/`, persist node upgrade
- `/var/lib/blockvisor/images/<protocol>/<node_type>/<node_version>/` downloaded images cache

### Node
- `/usr/bin/babel`
- `/usr/bin/babel_job_runner`
- `/etc/babel.conf`
- `/var/lib/babel/jobs/<job_name>/config.json`
- `/var/lib/babel/jobs/<job_name>/status.json`
- `/var/lib/babel/jobs/<job_name>/progress.json`
- `/var/lib/babel/jobs/<job_name>/logs`
- `/var/lib/babel/jobs_monitor.socket`
- `/var/lib/babel/node_env`
- `/var/lib/babel/post_setup.sh`
- `/var/lib/babel/plugin/*.rhai` node specific Babel plugin files
- `/blockjoy/.babel_jobs/` archive jobs (e.g. download) metadata dir, in particular `download.completed` file
- `/blockjoy/protocol_data/` directory where protocol data are downloaded (uploaded from)

### Bundle
- `bundle/installer`
- `bundle/babel/`
- `bundle/blockvisor/`

## Testing

See [BV tests](bv/tests/README.md) for more.

# High Level Overview

![](overview.jpg)

## Node Internals

![](node_internals.jpg)

## Basic Scenarios
### Add Host - Host Provisioning

```mermaid
sequenceDiagram
    participant user as User
    participant frontend as Fronted
    participant backend as API
    participant host as Host
    participant storage as Storage
    
    user->>frontend: add new Host
    frontend->>backend: new Host
    backend-->>frontend: PROVISION_TOKEN
    frontend-->>user: bvup with PROVISION_TOKEN
    user->>host: run bvup with PROVISION_TOKEN
    host->>backend: provision host
    host->> storage: download bundle
    host->>host: run bundle installer
```

### Add Node

#### Overview

```mermaid
sequenceDiagram
    participant backend as API
    participant bv as BlockvisorD
    participant apptainer as Apptainer
    
    backend->>bv: NodeCreate
    bv-->>bv: mount node rootfs
    bv-->>backend: InfoUpdate
    backend->>bv: NodeStart
    bv-->>backend: InfoUpdate
    bv->>apptainer: start container
    bv->>apptainer: start babel inside running container (listening on UDS)
```

### Execute Method on Node

```mermaid
sequenceDiagram
    participant cli as BV CLI
    participant bv as BlockvisorD
    participant babel as Babel

    cli->>bv: Node Method
    bv ->> bv: call method on Babel plugin  
    bv ->> babel: run_*, start_job, ...
    Note right of bv: forward run_*, start_job and other calls<br> to bebel, so it can be run on the node
    babel -->> bv: 
    Note right of bv: result is sent back to BVand processed<br>  by Babel plugin 
    bv-->>cli: response
```

### Self update processes

![](host_self_update.jpg)

#### Check for update

```mermaid
sequenceDiagram
    participant repo as Cookbook+Storage
    participant bv as BlockvisorD
    participant installer as bundle/installer
    
    loop configurable check interval
    bv ->> repo: check for update
        alt
            repo -->> bv: no updates
        else
            repo -->> bv: update available
            bv ->> repo: download latest bundle
            bv ->> installer: launch installer
        end
    end
```

#### BlockvisorD update

```mermaid
sequenceDiagram
    participant api as API
    participant bv as BlockvisorD
    participant installer as bundle/installer
    participant sysd as SystemD
    
    alt running version is not blacklisted (this is not rollback)
        installer ->> installer: backup running version
    end
    alt BlockvisorD is running
        installer ->> bv: notify about started update
        activate bv
        bv ->> api: change status to UPDATING
        bv ->> bv: finish pending actions    
        deactivate bv
    end
    installer ->> installer: switch current version (change symlinks)  
    installer ->> sysd: restart BlockvisorD service     
    sysd ->> bv: restart BlockvisorD service
    installer ->> bv: finalize update and get health check status
    activate bv
    alt update succeed
        bv ->> bv: update Babel on running nodes
        bv ->> api: change status to IDLE
        bv -->> installer: ok
        installer ->> installer: cleanup old version
    else update failed
        bv -->> installer: err
        installer ->> installer: mark this version as blacklisted
        alt was it rollback
            installer ->> api: send rollback error (broken host) notification
        else
            installer ->> installer: launch installer from backup version
        end
    end
    deactivate bv
```

#### Babel and JobRunner install/update

```mermaid
sequenceDiagram
    participant bv as BlockvisorD
    participant apptainer as Apptainer
    participant babel as Babel
    
    activate bv
    alt Start node
        bv ->> apptainer: start container
        bv ->> bv: copy fresh babel binary into node rootfs
        bv ->> babel: start babel inside running container
        activate babel
    deactivate babel
    else Attach to running node
        activate babel
        bv -->> babel: request graceful shutdown
        babel ->> babel: finish pending actions
        babel ->> babel: graceful shutdown
        deactivate babel
        bv ->> bv: copy fresh babel binary into node rootfs
        bv ->> babel: start babel again
        activate babel
    end

    bv ->> babel: check JobRunner binary
    alt JobRunner checksum doesn't match
        bv ->> babel: send new JobRunner binary
        babel ->> babel: replace JobRunner binary
        Note right of babel: JobRunner perform protocol action/entrypoint<br> so it is not restarted automatically
    end

    deactivate babel
    deactivate bv
```
