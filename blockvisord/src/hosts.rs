use crate::containers::NodeContainer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sysinfo::{DiskExt, System, SystemExt};

pub struct Host {
    pub containers: HashMap<String, Box<dyn NodeContainer>>,
    pub config: HostConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HostConfig {
    pub id: String,
    pub data_dir: String,
    pub pool_dir: String,
    pub token: String,
    pub blockjoy_api_url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HostInfo {
    pub name: Option<String>,
    pub cpu_count: Option<i64>, // because postgres does not have unsigned
    pub mem_size: Option<i64>,
    pub disk_size: Option<i64>,
    pub os: Option<String>,
    pub os_version: Option<String>,
}

pub fn get_host_info() -> HostInfo {
    let sys = System::new_all();

    HostInfo {
        name: sys.name(),
        cpu_count: sys.physical_core_count().map(|x| x as i64),
        mem_size: Some(sys.total_memory() as i64 * 1024),
        disk_size: Some(sys.disks()[0].total_space() as i64),
        os: sys.name(),
        os_version: sys.os_version(),
    }
}

pub fn get_ip_address(ifa_name: &str) -> String {
    let ifas = local_ip_address::list_afinet_netifas().unwrap();
    let (_, ip) = local_ip_address::find_ifa(ifas, ifa_name).unwrap();
    ip.to_string()
}
