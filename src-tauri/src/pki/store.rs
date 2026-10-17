/// Install the Root CA into the OS native trust store

pub fn install_root_ca(ca_cert_path: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    return install_macos(ca_cert_path);

    #[cfg(target_os = "linux")]
    return install_linux(ca_cert_path);

    #[cfg(target_os = "windows")]
    return install_windows(ca_cert_path);

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    Err("Trust store installation not supported on this platform".into())
}

/// Check if the CA is already trusted in the OS
pub fn is_ca_trusted(_ca_cert_path: &str) -> bool {
    // A full implementation would verify the cert fingerprint in the system keychain.
    // For now, we check whether the cert file exists at the expected installed location.
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("security")
            .args(["find-certificate", "-c", "Laradoc Studio Local CA", "/Library/Keychains/System.keychain"])
            .output();
        return output.map(|o| o.status.success()).unwrap_or(false);
    }

    #[cfg(target_os = "linux")]
    {
        return std::path::Path::new("/usr/local/share/ca-certificates/laradoc-studio-ca.crt").exists();
    }

    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("certutil")
            .args(["-verifystore", "Root", "Laradoc Studio Local CA"])
            .output();
        return output.map(|o| o.status.success()).unwrap_or(false);
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    false
}

#[cfg(target_os = "macos")]
fn install_macos(ca_cert_path: &str) -> Result<(), String> {
    let script = format!(
        r#"do shell script "security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain '{}'" with administrator privileges"#,
        ca_cert_path
    );
    let output = std::process::Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("osascript failed: {}", e))?;

    if output.status.success() {
        log::info!("Root CA installed into macOS System.keychain");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("macOS CA install failed: {}", stderr))
    }
}

#[cfg(target_os = "linux")]
fn install_linux(ca_cert_path: &str) -> Result<(), String> {
    let dest = "/usr/local/share/ca-certificates/laradoc-studio-ca.crt";

    // Copy cert with pkexec
    let copy = std::process::Command::new("pkexec")
        .args(["cp", ca_cert_path, dest])
        .output()
        .map_err(|e| format!("pkexec cp failed: {}", e))?;

    if !copy.status.success() {
        return Err(format!("pkexec copy failed: {}", String::from_utf8_lossy(&copy.stderr)));
    }

    // Update CA certificates
    let update = std::process::Command::new("pkexec")
        .args(["update-ca-certificates"])
        .output()
        .map_err(|e| format!("update-ca-certificates failed: {}", e))?;

    if update.status.success() {
        log::info!("Root CA installed on Linux via update-ca-certificates");
        Ok(())
    } else {
        Err(format!("update-ca-certificates failed: {}", String::from_utf8_lossy(&update.stderr)))
    }
}

#[cfg(target_os = "windows")]
fn install_windows(ca_cert_path: &str) -> Result<(), String> {
    let ps_cmd = format!(
        "Start-Process -Verb RunAs -Wait -FilePath certutil.exe -ArgumentList '-addstore -f Root \"{}\"'",
        ca_cert_path
    );

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
        .output()
        .map_err(|e| format!("PowerShell certutil failed: {}", e))?;

    if output.status.success() {
        log::info!("Root CA installed via certutil on Windows");
        Ok(())
    } else {
        Err(format!("certutil failed: {}", String::from_utf8_lossy(&output.stderr)))
    }
}
