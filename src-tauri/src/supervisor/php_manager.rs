/// Smart Composer shim path helper — routes to the correct PHP binary
/// This mirrors the bin/composer shim logic at the Rust level
pub fn composer_php_binary(bin_dir: &str, php_version: &str) -> String {
    format!("{}/php/{}/bin/php", bin_dir, php_version)
}

pub fn composer_phar_path(bin_dir: &str) -> String {
    format!("{}/composer/composer.phar", bin_dir)
}

/// Detect the PHP version constraint from a project's composer.json
pub fn detect_project_php_version(project_path: &str) -> Option<String> {
    let composer_json = std::path::Path::new(project_path).join("composer.json");
    if !composer_json.exists() {
        // Check .php-version file
        let php_version_file = std::path::Path::new(project_path).join(".php-version");
        if php_version_file.exists() {
            if let Ok(content) = std::fs::read_to_string(php_version_file) {
                let trimmed = content.trim();
                // Accept e.g. "8.3", "8.3.0", "php8.3"
                let version = trimmed
                    .trim_start_matches("php")
                    .split('.')
                    .take(2)
                    .collect::<Vec<_>>()
                    .join(".");
                return Some(version);
            }
        }
        return None;
    }

    let content = std::fs::read_to_string(composer_json).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;

    let php_constraint = json
        .get("require")
        .and_then(|r| r.get("php"))
        .and_then(|p| p.as_str())?;

    // Extract major.minor from constraint like "^8.2", ">=8.3", "~8.4.0"
    let version_str: String = php_constraint
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect();

    let parts: Vec<&str> = version_str.splitn(3, '.').collect();
    if parts.len() >= 2 {
        Some(format!("{}.{}", parts[0], parts[1]))
    } else {
        None
    }
}
