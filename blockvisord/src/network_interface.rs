use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::IpAddr;
use zbus::zvariant::Type;

#[derive(Deserialize, Serialize, Debug, Clone, Type)]
pub struct NetworkInterface {
    pub name: String,
    pub ip: IpAddr,
}

impl fmt::Display for NetworkInterface {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.ip)
    }
}
