use crate::core::{
    errors::AppError,
    models::{AdapterInfo, SessionJournal},
};
use std::net::Ipv4Addr;
use std::process::Command;

fn run_powershell(script: &str) -> Result<String, AppError> {
    let script = format!(
        "$ErrorActionPreference = 'Stop'; [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new(); {script}"
    );
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .output()
        .map_err(|e| AppError::Platform(e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        let details = match (stderr.is_empty(), stdout.is_empty()) {
            (false, false) => format!("{stderr}\n{stdout}"),
            (false, true) => stderr,
            (true, false) => stdout,
            (true, true) => format!("PowerShell skončil se stavem {}.", output.status),
        };
        return Err(AppError::Platform(if details.is_empty() {
            "Operace změny IP adresy selhala.".into()
        } else {
            details
        }));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

pub fn add_temporary_address(
    adapter: &AdapterInfo,
    ip: Ipv4Addr,
    prefix: u8,
) -> Result<SessionJournal, AppError> {
    if !cfg!(windows) {
        return Err(AppError::Platform("Windows API není dostupné.".into()));
    }
    let interface_index = adapter
        .id
        .parse::<u32>()
        .map_err(|_| AppError::AdapterUnavailable)?;
    let ip_text = ip.to_string();
    tracing::info!(interface_index, ip = %ip_text, prefix, "přidávám dočasnou IPv4 adresu");
    let existing = format!(
        "@(Get-NetIPAddress -InterfaceIndex {interface_index} -AddressFamily IPv4 -IPAddress {ip_text} -ErrorAction SilentlyContinue | Select-Object -ExpandProperty PrefixLength) -join ','"
    );
    let existing_prefixes = run_powershell(&existing)?;
    if !existing_prefixes.is_empty() {
        let already_configured = existing_prefixes
            .split(',')
            .all(|existing| existing.trim().parse::<u8>() == Ok(prefix));
        if already_configured {
            tracing::info!(interface_index, ip = %ip_text, prefix, "dočasná IPv4 adresa už je správně nastavena");
            return Ok(SessionJournal {
                adapter_id: adapter.id.clone(),
                temporary_ip: ip,
                prefix_length: prefix,
                created_by_application: false,
            });
        }
        tracing::warn!(interface_index, ip = %ip_text, existing_prefixes, expected_prefix = prefix, "existující IPv4 adresa má jiný prefix");
        return Err(AppError::AddressConflict);
    }
    let add = format!("New-NetIPAddress -InterfaceIndex {interface_index} -IPAddress {ip_text} -PrefixLength {prefix} -AddressFamily IPv4 -Type Unicast -ErrorAction Stop | Out-Null");
    run_powershell(&add).map_err(|error| {
        tracing::error!(interface_index, ip = %ip_text, prefix, %error, "přidání IPv4 adresy selhalo");
        error
    })?;
    // DAD can leave a new address in Tentative state while the Ethernet link is
    // disconnected. That is expected in this wizard because PoE is enabled only
    // after the PC address has been prepared. Verify the concrete address and
    // prefix, not the transient AddressState.
    let verify = format!(
        "$x = @(Get-NetIPAddress -InterfaceIndex {interface_index} -AddressFamily IPv4 -ErrorAction SilentlyContinue | Where-Object {{ $_.IPAddress -eq '{ip_text}' -and $_.PrefixLength -eq {prefix} }}); if ($x.Count -eq 0) {{ exit 1 }}"
    );
    run_powershell(&verify).map_err(|error| {
        tracing::error!(interface_index, ip = %ip_text, prefix, %error, "ověření IPv4 adresy selhalo");
        AppError::Platform(format!("Windows nepotvrdil přidání dočasné IPv4 adresy: {error}"))
    })?;
    Ok(SessionJournal {
        adapter_id: adapter.id.clone(),
        temporary_ip: ip,
        prefix_length: prefix,
        created_by_application: true,
    })
}
pub fn remove_temporary_address(journal: &SessionJournal) -> Result<(), AppError> {
    if !journal.created_by_application {
        return Ok(());
    }
    let interface_index = journal
        .adapter_id
        .parse::<u32>()
        .map_err(|_| AppError::AdapterUnavailable)?;
    let ip = journal.temporary_ip;
    let script = format!("$x = Get-NetIPAddress -InterfaceIndex {interface_index} -AddressFamily IPv4 -IPAddress {ip} -ErrorAction SilentlyContinue; if ($x) {{ $x | Remove-NetIPAddress -Confirm:$false -ErrorAction Stop }}");
    run_powershell(&script)?;
    let verify = format!("if (Get-NetIPAddress -InterfaceIndex {interface_index} -AddressFamily IPv4 -IPAddress {ip} -ErrorAction SilentlyContinue) {{ exit 1 }}");
    run_powershell(&verify).map_err(|_| {
        AppError::Platform("Windows nepotvrdil odstranění dočasné IPv4 adresy.".into())
    })?;
    Ok(())
}
pub fn temporary_address_configured(journal: &SessionJournal) -> Result<bool, AppError> {
    let interface_index = journal
        .adapter_id
        .parse::<u32>()
        .map_err(|_| AppError::AdapterUnavailable)?;
    let script = format!(
        "$x = @(Get-NetIPAddress -InterfaceIndex {interface_index} -AddressFamily IPv4 -ErrorAction SilentlyContinue | Where-Object {{ $_.IPAddress -eq '{}' -and $_.PrefixLength -eq {} }}); if ($x.Count -gt 0) {{ exit 0 }} else {{ exit 1 }}",
        journal.temporary_ip,
        journal.prefix_length
    );
    Ok(run_powershell(&script).is_ok())
}
