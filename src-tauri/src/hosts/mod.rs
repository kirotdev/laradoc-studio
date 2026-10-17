pub mod elevation;

const BEGIN_MARKER: &str = "# BEGIN LARADOC-STUDIO";
const END_MARKER: &str = "# END LARADOC-STUDIO";

#[cfg(unix)]
fn hosts_path() -> &'static str {
    "/etc/hosts"
}

#[cfg(windows)]
fn hosts_path() -> &'static str {
    r"C:\Windows\System32\drivers\etc\hosts"
}

/// Read the current hosts file contents
pub fn read_hosts() -> Result<String, String> {
    std::fs::read_to_string(hosts_path())
        .map_err(|e| format!("Failed to read hosts file: {}", e))
}

/// Atomically write the hosts file with updated LARADOC-STUDIO block.
/// Only modifies the delimited block; all other entries are preserved.
pub fn sync_hosts(entries: &[(String, String)]) -> Result<(), String> {
    let current = read_hosts()?;
    let updated = rebuild_hosts(&current, entries)?;

    // Attempt direct write first (works if running as root / administrator)
    match atomic_write(hosts_path(), &updated) {
        Ok(_) => return Ok(()),
        Err(_) => {} // Fall through to elevation
    }

    // Elevation required
    elevation::write_hosts_elevated(&updated)
}

/// Add a single domain mapping (e.g. "my-app.test" -> "127.0.0.1")
pub fn add_domain(domain: &str) -> Result<(), String> {
    let current = read_hosts()?;
    let mut entries = parse_laradoc_entries(&current);
    entries.retain(|(d, _)| d != domain);
    entries.push((domain.to_string(), "127.0.0.1".to_string()));
    let updated = rebuild_hosts(&current, &entries)?;

    match atomic_write(hosts_path(), &updated) {
        Ok(_) => Ok(()),
        Err(_) => elevation::write_hosts_elevated(&updated),
    }
}

/// Remove a domain mapping
pub fn remove_domain(domain: &str) -> Result<(), String> {
    let current = read_hosts()?;
    let mut entries = parse_laradoc_entries(&current);
    entries.retain(|(d, _)| d != domain);
    let updated = rebuild_hosts(&current, &entries)?;

    match atomic_write(hosts_path(), &updated) {
        Ok(_) => Ok(()),
        Err(_) => elevation::write_hosts_elevated(&updated),
    }
}

/// Parse only the entries inside the LARADOC-STUDIO delimiters
fn parse_laradoc_entries(content: &str) -> Vec<(String, String)> {
    let mut inside = false;
    let mut entries = Vec::new();
    for line in content.lines() {
        if line.trim() == BEGIN_MARKER {
            inside = true;
            continue;
        }
        if line.trim() == END_MARKER {
            inside = false;
            continue;
        }
        if inside {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && !parts[0].starts_with('#') {
                entries.push((parts[1].to_string(), parts[0].to_string()));
            }
        }
    }
    entries
}

/// Rebuild the full hosts file, replacing only the delimited block
fn rebuild_hosts(original: &str, entries: &[(String, String)]) -> Result<String, String> {
    // Strip any existing LARADOC-STUDIO block
    let mut output = Vec::new();
    let mut inside = false;

    for line in original.lines() {
        if line.trim() == BEGIN_MARKER {
            inside = true;
            continue;
        }
        if line.trim() == END_MARKER {
            inside = false;
            continue;
        }
        if !inside {
            output.push(line.to_string());
        }
    }

    // Append fresh block
    if !entries.is_empty() {
        output.push(String::new());
        output.push(BEGIN_MARKER.to_string());
        for (domain, ip) in entries {
            output.push(format!("{}\t{}", ip, domain));
        }
        output.push(END_MARKER.to_string());
    }

    Ok(output.join("\n") + "\n")
}

/// Atomic write via temp file + rename
fn atomic_write(path: &str, content: &str) -> Result<(), String> {
    let tmp = format!("{}.laradoc.tmp", path);
    std::fs::write(&tmp, content).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())?;
    Ok(())
}
