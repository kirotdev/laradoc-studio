use super::{LaradocConfig, ServiceKind};
use tokio::process::Command;

/// Spawn a service as a new process group on Unix (macOS & Linux).
/// Using setpgid(0, 0) so we can killpg() the entire group cleanly.
pub async fn spawn_service(
    kind: &ServiceKind,
    config: &LaradocConfig,
    binary: &str,
) -> Result<u32, String> {
    let args = build_args(kind, config);

    let mut cmd = Command::new(binary);
    cmd.args(&args);
    cmd.current_dir(&config.data_directory);

    // Place in new process group so killpg works
    unsafe {
        cmd.pre_exec(|| {
            libc::setpgid(0, 0);
            Ok(())
        });
    }

    // Silence stdout/stderr — logs are tailed separately
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn {:?}: {}", kind, e))?;

    let pid = child
        .id()
        .ok_or_else(|| "Process exited immediately".to_string())?;

    // Write PID file for crash recovery on next startup
    let pid_path = format!("{}/pids/{}.pid", config.data_directory, kind.as_str());
    let _ = std::fs::create_dir_all(format!("{}/pids", config.data_directory));
    let _ = std::fs::write(&pid_path, pid.to_string());

    Ok(pid)
}

/// Gracefully terminate a service: SIGTERM first, SIGKILL after 5s timeout
pub async fn kill_service(pid: u32) {
    use nix::unistd::Pid;
    use nix::sys::signal::{killpg, Signal};

    let pgid = Pid::from_raw(pid as i32);

    // SIGTERM to the entire process group
    if let Err(e) = killpg(pgid, Signal::SIGTERM) {
        log::warn!("SIGTERM to pgid {} failed: {}", pid, e);
    }

    // Wait up to 5 seconds for graceful shutdown
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if tokio::time::Instant::now() >= deadline {
            break;
        }
        // Check if process is still alive
        if unsafe { libc::kill(pid as i32, 0) } != 0 {
            log::info!("Process {} exited gracefully", pid);
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    // SIGKILL escalation
    log::warn!("Process {} did not exit gracefully, sending SIGKILL", pid);
    if let Err(e) = killpg(pgid, Signal::SIGKILL) {
        log::error!("SIGKILL to pgid {} failed: {}", pid, e);
    }
}

/// Verify PID files from a previous crash and clean them up
pub async fn cleanup_stale_pids(data_dir: &str) {
    let pid_dir = format!("{}/pids", data_dir);
    let Ok(entries) = std::fs::read_dir(&pid_dir) else { return };

    for entry in entries.flatten() {
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            if let Ok(pid) = content.trim().parse::<u32>() {
                // Check if still alive with signal 0
                if unsafe { libc::kill(pid as i32, 0) } == 0 {
                    log::info!("Cleaning up stale PID {} from previous session", pid);
                    kill_service(pid).await;
                }
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
}

fn build_args(kind: &ServiceKind, config: &LaradocConfig) -> Vec<String> {
    let etc = format!("{}/etc", config.bin_directory.replace("/bin", ""));
    let data = &config.data_directory;

    match kind {
        ServiceKind::Nginx => vec![
            "-c".into(),
            format!("{}/nginx/nginx.conf", etc),
            "-g".into(),
            "daemon off;".into(),
        ],
        ServiceKind::Php82 => vec![
            "--fpm-config".into(),
            format!("{}/php/8.2/php-fpm.conf", etc),
            "--nodaemonize".into(),
        ],
        ServiceKind::Php83 => vec![
            "--fpm-config".into(),
            format!("{}/php/8.3/php-fpm.conf", etc),
            "--nodaemonize".into(),
        ],
        ServiceKind::Php84 => vec![
            "--fpm-config".into(),
            format!("{}/php/8.4/php-fpm.conf", etc),
            "--nodaemonize".into(),
        ],
        ServiceKind::Php85 => vec![
            "--fpm-config".into(),
            format!("{}/php/8.5/php-fpm.conf", etc),
            "--nodaemonize".into(),
        ],
        ServiceKind::Mysql => vec![
            format!("--datadir={}/mysql", data),
            format!("--socket={}/sockets/mysql.sock", data),
            "--user=root".into(),
        ],
        ServiceKind::Redis => vec![
            format!("{}/redis/redis.conf", etc),
            "--daemonize".into(),
            "no".into(),
            format!("--dir={}/redis", data),
        ],
        ServiceKind::Mailpit => vec![
            "--smtp".into(),
            "127.0.0.1:1025".into(),
            "--listen".into(),
            "127.0.0.1:8025".into(),
            "--storage".into(),
            format!("{}/mail", data),
        ],
    }
}
