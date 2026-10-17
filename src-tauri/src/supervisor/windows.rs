use super::{LaradocConfig, ServiceKind};
use std::os::windows::io::AsRawHandle;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
use windows::Win32::System::Threading::OpenProcess;
use windows::Win32::System::Threading::{
    TerminateProcess, PROCESS_ACCESS_RIGHTS, PROCESS_TERMINATE,
};

/// Windows Job Object handle — when this process exits, all children are automatically killed
static mut JOB_HANDLE: Option<HANDLE> = None;

/// Initialize the global Job Object at startup
pub fn init_job_object() -> Result<(), String> {
    unsafe {
        let job = CreateJobObjectW(None, None)
            .map_err(|e| format!("CreateJobObjectW failed: {}", e))?;

        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &info as *const _ as *const _,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
        .map_err(|e| format!("SetInformationJobObject failed: {}", e))?;

        JOB_HANDLE = Some(job);
        Ok(())
    }
}

/// Spawn a service process on Windows and assign it to the Job Object
pub async fn spawn_service(
    kind: &ServiceKind,
    config: &LaradocConfig,
) -> Result<u32, String> {
    let binary = format!("{}\\{}.exe", config.bin_directory, kind.binary_name().replace('/', "\\"));
    let args = build_args(kind, config);

    let mut cmd = tokio::process::Command::new(&binary);
    cmd.args(&args);
    cmd.current_dir(&config.data_directory);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    // CREATE_SUSPENDED would allow assigning to job before first instruction, but tokio spawn
    // doesn't expose that directly; we assign immediately after spawn instead
    let child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn {:?}: {}", kind, e))?;

    let pid = child.id().ok_or("Process exited immediately")?;

    // Assign to global Job Object so kernel kills it when laradoc-studio exits
    unsafe {
        if let Some(job) = JOB_HANDLE {
            let process = OpenProcess(PROCESS_ACCESS_RIGHTS(PROCESS_TERMINATE.0), false, pid)
                .map_err(|e| format!("OpenProcess failed: {}", e))?;
            AssignProcessToJobObject(job, process)
                .map_err(|e| format!("AssignProcessToJobObject failed: {}", e))?;
        }
    }

    // Write PID file
    let pid_path = format!("{}\\pids\\{}.pid", config.data_directory, kind.as_str());
    let _ = std::fs::create_dir_all(format!("{}\\pids", config.data_directory));
    let _ = std::fs::write(&pid_path, pid.to_string());

    log::info!("Started {:?} on Windows (PID {})", kind, pid);
    Ok(pid)
}

/// Terminate a Windows process by PID
pub async fn kill_service(pid: u32) {
    unsafe {
        match OpenProcess(PROCESS_ACCESS_RIGHTS(PROCESS_TERMINATE.0), false, pid) {
            Ok(handle) => {
                if let Err(e) = TerminateProcess(handle, 0) {
                    log::error!("TerminateProcess({}) failed: {}", pid, e);
                } else {
                    log::info!("Terminated Windows process PID {}", pid);
                }
            }
            Err(e) => log::warn!("OpenProcess({}) failed: {}", pid, e),
        }
    }
}

fn build_args(kind: &ServiceKind, config: &LaradocConfig) -> Vec<String> {
    let etc = config.bin_directory.replace("\\bin", "\\etc");
    let data = &config.data_directory;

    match kind {
        ServiceKind::Nginx => vec![
            "-c".into(),
            format!("{}\\nginx\\nginx.conf", etc),
        ],
        ServiceKind::Php82 => vec![
            "--fpm-config".into(),
            format!("{}\\php\\8.2\\php-fpm.conf", etc),
        ],
        ServiceKind::Php83 => vec![
            "--fpm-config".into(),
            format!("{}\\php\\8.3\\php-fpm.conf", etc),
        ],
        ServiceKind::Php84 => vec![
            "--fpm-config".into(),
            format!("{}\\php\\8.4\\php-fpm.conf", etc),
        ],
        ServiceKind::Php85 => vec![
            "--fpm-config".into(),
            format!("{}\\php\\8.5\\php-fpm.conf", etc),
        ],
        ServiceKind::Mysql => vec![
            format!("--datadir={}\\mysql", data),
        ],
        ServiceKind::Redis => vec![
            format!("{}\\redis\\redis.conf", etc),
        ],
        ServiceKind::Mailpit => vec![
            "--smtp".into(), "127.0.0.1:1025".into(),
            "--listen".into(), "127.0.0.1:8025".into(),
        ],
    }
}
