use sysinfo::{DiskExt, System, SystemExt};

#[derive(Debug)]
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

    let disk_size = sys
        .disks()
        .iter()
        .fold(0, |acc, disk| acc + disk.total_space());

    HostInfo {
        name: sys.host_name(),
        cpu_count: sys.physical_core_count().map(|x| x as i64),
        mem_size: Some(sys.total_memory() as i64 * 1024),
        disk_size: Some(disk_size as i64),
        os: sys.name(),
        os_version: sys.os_version(),
    }
}

pub fn get_ip_address(ifa_name: &str) -> String {
    let ifas = local_ip_address::list_afinet_netifas().unwrap();
    let (_, ip) = local_ip_address::find_ifa(ifas, ifa_name).unwrap();
    ip.to_string()
}
