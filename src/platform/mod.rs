#[cfg(windows)]
pub mod windows;
#[cfg(windows)]
pub use windows::{
    add_temporary_address, enumerate_adapters, launch_browser, remove_temporary_address,
    temporary_address_configured,
};
#[cfg(not(windows))]
pub fn enumerate_adapters(
) -> Result<Vec<crate::core::models::AdapterInfo>, crate::core::errors::AppError> {
    Ok(vec![crate::core::models::AdapterInfo {
        id: "development".into(),
        name: "Vývojové rozhraní".into(),
        description: "Dostupné pouze mimo Windows".into(),
        mac: "-".into(),
        operational_status: "Neznámý".into(),
        connection_status: "Neznámý".into(),
        ipv4_addresses: vec![],
        dhcp_enabled: None,
        default_gateway: None,
        is_physical: true,
    }])
}
#[cfg(not(windows))]
pub fn add_temporary_address(
    a: &crate::core::models::AdapterInfo,
    ip: std::net::Ipv4Addr,
    prefix: u8,
) -> Result<crate::core::models::SessionJournal, crate::core::errors::AppError> {
    Ok(crate::core::models::SessionJournal {
        adapter_id: a.id.clone(),
        temporary_ip: ip,
        prefix_length: prefix,
        created_by_application: true,
    })
}
#[cfg(not(windows))]
pub fn remove_temporary_address(
    _: &crate::core::models::SessionJournal,
) -> Result<(), crate::core::errors::AppError> {
    Ok(())
}
#[cfg(not(windows))]
pub fn temporary_address_configured(
    _: &crate::core::models::SessionJournal,
) -> Result<bool, crate::core::errors::AppError> {
    Ok(true)
}
#[cfg(not(windows))]
pub fn launch_browser(_: &str) -> Result<(), crate::core::errors::AppError> {
    Ok(())
}
