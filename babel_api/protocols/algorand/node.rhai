const METADATA = #{
  min_babel_version: "0.15.4",
  node_version: "3.16.3",
  protocol: "algorand",
  node_type: "node",
  description: "Algorand node",
  requirements: #{
    vcpu_count: 4,
    mem_size_mb: 8096,
    disk_size_gb: 100
  },
  nets: #{
    mainnet: #{
      url: "https://algorand-catchpoints.s3.us-east-2.amazonaws.com/channel/mainnet/latest.catchpoint",
      net_type: "main",
    },
    testnet: #{
      url: "https://algorand-catchpoints.s3.us-east-2.amazonaws.com/channel/testnet/latest.catchpoint",
      net_type: "test",
    },
  },
  babel_config: #{
    data_directory_mount_point: "/blockjoy/algorand/data",
    log_buffer_capacity_ln: 1000,
    swap_size_mb: 1024
  },
  firewall: #{
    enabled: true,
    default_in: "deny",
    default_out: "allow",
    rules: [
      #{
        name: "Allowed incoming traffic on port",
        action: "allow",
        direction: "in",
        protocol: "tcp",
        ports: [8080, 7833]
      },
    ],
  },
  keys: #{},
};

const API_HOST = "http://localhost:8080/";

fn init(keys) {
  run_sh("cp /opt/netdata/usr/lib/netdata/system/systemd/netdata.service /etc/systemd/system/");
  run_sh("cd /etc/systemd/system/multi-user.target.wants && ln -s ../netdata.service && systemctl enable netdata && systemctl start netdata");
  debug("Executing startup scripts...");

  let net = sanitize_sh_param(node_params().NETWORK);
  let metadata = global::METADATA.nets[net];
  let data_directory = global::METADATA.babel_config.data_directory_mount_point;

  start_job("Entrypoint for the Algorand validator node", #{
      job_type: #{
          run_sh: `/usr/bin/algod -d ${data_directory} -l 0.0.0.0:8080 -g /var/lib/algorand/genesis/${net}/genesis.json -o`
      },
      restart: #{
          "always": #{
              backoff_timeout_ms: 60000,
              backoff_base_ms: 10000
          },
      },
      needs: [],
  });
}

fn application_status() {
	let res = run_rest(#{url: "http://localhost:8080/health"});
	debug(res);
	if res.status_code != 200 {
		"delinquent"
	} else {
		"broadcasting"
	}
}

fn get_genesis_info() {
  let res = run_jrpc(#{host: global::API_HOST, method: "genesis"});
  if res.status_code != 200 {
      throw res.status_code;
  }
  parse_json(res.result)
}

fn sync_status() {
  let token = run_sh(`cat ${global::METADATA.babel_config.data_directory_mount_point}/algod.token`).stdout;
  let res = run_sh(`curl http://localhost:8080/v2/status -H "X-Algo-API-Token:${token}" | jq '.["catchup-time"]'`);
  let catchup_time = parse_int(res.stdout);
  if catchup_time > 0 {
  	"syncing"
  } else {
	  "synced"
  }
}

fn height() {
  let token = run_sh(`cat ${global::METADATA.babel_config.data_directory_mount_point}/algod.token`).stdout;
  let res = run_sh(`curl http://localhost:8080/v2/status -H "X-Algo-API-Token:${token}" | jq '.["last-round"]'`);
  parse_int(res.stdout)
}

fn address() {
  run_sh(`cat ${global::METADATA.babel_config.data_directory_mount_point}/algod.token`).stdout
}

fn stop_babelsup(param) {
  run_sh("systemctl stop babelsup");
  for line in run_sh("ps -e|grep babel").split("\n") {
      let parts = line.split();
      if !parts.is_empty() {
          let pid = parts[0];
          pid.trim();
          if !pid.is_empty() {
              run_sh("kill -9 " + pid);
          }
      }
  }
  ""
}

fn disable_babelsup(param) {
  run_sh("systemctl disable babelsup")
}
