/// Platform-specific privilege elevation for writing the hosts file

/// Write the hosts file using the appropriate elevation mechanism
pub fn write_hosts_elevated(content: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    return write_hosts_macos(content);

    #[cfg(target_os = "linux")]
    return write_hosts_linux(content);

    #[cfg(target_os = "windows")]
    return write_hosts_windows(content);

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    Err("Elevation not supported on this platform".into())
}

/// macOS: use osascript to request TouchID / password authorization
#[cfg(target_os = "macos")]
fn write_hosts_macos(content: &str) -> Result<(), String> {
    let escaped = content.replace('"', "\\\"").replace('\n', "\\n");
    let script = format!(
        r#"do shell script "printf \"{}\" > /etc/hosts" with administrator privileges"#,
        escaped
    );

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("osascript failed: {}", e))?;

    if output.status.success() {
        log::info!("Hosts file updated via osascript (macOS)");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("User cancelled") || stderr.contains("(-128)") {
            Err("User cancelled the authentication dialog".into())
        } else {
            Err(format!("osascript error: {}", stderr))
        }
    }
}

/// Linux: use pkexec to trigger the native Polkit authorization dialog
#[cfg(target_os = "linux")]
fn write_hosts_linux(content: &str) -> Result<(), String> {
    use std::io::Write;

    let tmp = "/tmp/laradoc-hosts.tmp";
    std::fs::write(tmp, content).map_err(|e| format!("Write tmp failed: {}", e))?;

    let output = std::process::Command::new("pkexec")
        .args(["cp", tmp, "/etc/hosts"])
        .output()
        .map_err(|e| format!("pkexec failed: {}", e))?;

    let _ = std::fs::remove_file(tmp);

    if output.status.success() {
        log::info!("Hosts file updated via pkexec (Linux)");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("pkexec error: {}", stderr))
    }
}

/// Windows: use PowerShell Start-Process with RunAs verb to trigger UAC
#[cfg(target_os = "windows")]
fn write_hosts_windows(content: &str) -> Result<(), String> {
    let tmp = std::env::temp_dir().join("laradoc-hosts.tmp");
    std::fs::write(&tmp, content).map_err(|e| format!("Write tmp failed: {}", e))?;

    let hosts_path = r"C:\Windows\System32\drivers\etc\hosts";
    let tmp_str = tmp.to_string_lossy();

    let ps_cmd = format!(
        "Start-Process -Verb RunAs -Wait -FilePath cmd.exe -ArgumentList '/c copy /Y \"{}\" \"{}\"'",
        tmp_str, hosts_path
    );

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
        .output()
        .map_err(|e| format!("PowerShell elevation failed: {}", e))?;

    let _ = std::fs::remove_file(&tmp);

    if output.status.success() {
        log::info!("Hosts file updated via PowerShell RunAs (Windows)");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("PowerShell UAC error: {}", stderr))
    }
}
