use crate::core::{errors::AppError, models::AdapterInfo};
use std::process::Command;

pub fn enumerate_adapters() -> Result<Vec<AdapterInfo>, AppError> {
    // Windows PowerShell 5.1 writes redirected stdout as UTF-16 by default. Set the
    // process encoding explicitly so serde_json receives valid UTF-8 on all Windows 10/11 hosts.
    let command = r#"$OutputEncoding = [System.Text.UTF8Encoding]::new($false); [Console]::OutputEncoding = $OutputEncoding; $items = Get-NetAdapter | ForEach-Object { $a = $_; $ips = @(Get-NetIPAddress -InterfaceIndex $a.ifIndex -AddressFamily IPv4 -ErrorAction SilentlyContinue | ForEach-Object { $_.IPAddress }); $cfg = Get-NetIPConfiguration -InterfaceIndex $a.ifIndex -ErrorAction SilentlyContinue; $ipif = Get-NetIPInterface -InterfaceIndex $a.ifIndex -AddressFamily IPv4 -ErrorAction SilentlyContinue; [pscustomobject]@{ ifIndex=$a.ifIndex; Name=$a.Name; InterfaceDescription=$a.InterfaceDescription; MacAddress=$a.MacAddress; Status=[string]$a.Status; MediaConnectionState=[string]$a.MediaConnectionState; HardwareInterface=$a.HardwareInterface; IPv4Addresses=$ips; DhcpEnabled=([string]$ipif.Dhcp -eq 'Enabled'); DefaultGateway=if ($cfg -and $cfg.IPv4DefaultGateway) { [string]$cfg.IPv4DefaultGateway.NextHop } else { $null } } }; @($items) | ConvertTo-Json -Compress"#;
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            command,
        ])
        .output()
        .map_err(|e| AppError::Platform(e.to_string()))?;
    if !output.status.success() {
        let details = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(AppError::Platform(if details.is_empty() {
            "Get-NetAdapter selhal.".into()
        } else {
            format!("Get-NetAdapter selhal: {details}")
        }));
    }
    let value: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| AppError::Platform(format!("Výstup systému nebyl platný UTF-8/JSON: {e}")))?;
    let rows = value.as_array().cloned().unwrap_or_else(|| vec![value]);
    Ok(rows
        .into_iter()
        .filter_map(|v| {
            Some(AdapterInfo {
                id: v.get("ifIndex")?.to_string(),
                name: v.get("Name")?.as_str().unwrap_or("?").into(),
                description: v
                    .get("InterfaceDescription")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .into(),
                mac: v
                    .get("MacAddress")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .into(),
                operational_status: v
                    .get("Status")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .into(),
                connection_status: v
                    .get("MediaConnectionState")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .into(),
                ipv4_addresses: v
                    .get("IPv4Addresses")
                    .map(|x| match x {
                        serde_json::Value::Array(items) => items
                            .iter()
                            .filter_map(|ip| ip.as_str().map(str::to_owned))
                            .collect(),
                        serde_json::Value::String(ip) => vec![ip.clone()],
                        _ => vec![],
                    })
                    .unwrap_or_default(),
                dhcp_enabled: v.get("DhcpEnabled").and_then(|x| x.as_bool()),
                default_gateway: v
                    .get("DefaultGateway")
                    .and_then(|x| x.as_str())
                    .filter(|x| !x.is_empty())
                    .map(str::to_owned),
                is_physical: v
                    .get("HardwareInterface")
                    .and_then(|x| x.as_bool())
                    .unwrap_or(false),
            })
        })
        .collect())
}
