use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

pub const LOCAL_IP: Ipv4Addr = Ipv4Addr::new(172, 20, 253, 14);
pub const ANTENNA_IP: Ipv4Addr = Ipv4Addr::new(172, 20, 253, 13);
pub const PREFIX_LENGTH: u8 = 16;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdapterInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub mac: String,
    pub operational_status: String,
    pub connection_status: String,
    pub ipv4_addresses: Vec<String>,
    pub dhcp_enabled: Option<bool>,
    pub default_gateway: Option<String>,
    pub is_physical: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionJournal {
    pub adapter_id: String,
    pub temporary_ip: Ipv4Addr,
    pub prefix_length: u8,
    pub created_by_application: bool,
}

pub fn prefix_to_mask(prefix: u8) -> Option<Ipv4Addr> {
    if prefix > 32 {
        return None;
    }
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };
    Some(Ipv4Addr::from(mask))
}
