# Laradoc Studio: Desktop Application Implementation Plan & Master Blueprint

> **Target Platforms:** Windows 10/11, macOS (Intel & Apple Silicon), Linux (Ubuntu, Debian, Fedora, Arch)  
> **Framework:** Tauri v2 (Rust Backend + Native Webview Frontend) + Native System Tray  
> **Application Name:** Laradoc Studio (`laradoc-studio`)  
> **Architecture Reference:** Inspired by Laravel Herd & Laragon cross-platform developer environments with zero background bloat.

---

## Executive Summary & Core Requirements

1. **Strictly No Auto-Starting (Cold Start by Default):**
   - Zero background services, systemd daemons, Windows services, or macOS `launchd` LaunchAgents registered on OS boot.
   - When the user launches Laradoc Studio, all managed services initialize in the **Stopped** (⚪) state.
   - Services run **only** when the user explicitly triggers "Start All" or starts an individual service card.
   - Closing or quitting terminates all child processes cleanly without leaving background orphan processes.

2. **Multi-PHP Version Switcher (PHP 8.2, 8.3, 8.4, 8.5):**
   - Simultaneous, isolated FastCGI pools:
     - PHP 8.2: `127.0.0.1:9082` (or socket `data/sockets/php82.sock`)
     - PHP 8.3: `127.0.0.1:9083` (or socket `data/sockets/php83.sock`)
     - PHP 8.4: `127.0.0.1:9084` (or socket `data/sockets/php84.sock`)
     - PHP 8.5: `127.0.0.1:9085` (or socket `data/sockets/php85.sock`)
   - **Global Version Switching:** 1-click global CLI PHP version switcher in the UI header and System Tray context menu.
   - **Per-Project Routing:** Each `<project>.test` Nginx virtual host can target a specific PHP version pool, with auto-detection from `composer.json` (`"require": { "php": "^8.x" }`) or `.php-version`.

3. **Composer Auto-Switching Based on Active PHP:**
   - Smart CLI shims (`bin/composer` for Unix and `bin/composer.bat` for Windows) dynamically route execution of `composer.phar` to the matching PHP binary (project-specific or global active).
   - In-app terminal integration pre-configured with the active PHP version in `PATH`.

4. **Redis Service:**
   - Supervised on `127.0.0.1:6379`.
   - Isolated data directory at `data/redis/` (no root permissions required).
   - Dashboard card with real-time memory usage, client connection counts, and 1-click cache flush / restart.

5. **Mail Catcher & Mailport (Local SMTP Testing):**
   - Embedded SMTP listener on port `127.0.0.1:1025` (standard local test mailport).
   - Optional HTTP API / Web UI on `127.0.0.1:8025`.
   - `php.ini` across all PHP versions configured to forward outgoing `mail()` to port 1025.
   - Laravel `.env` compatibility: `MAIL_MAILER=smtp`, `MAIL_HOST=127.0.0.1`, `MAIL_PORT=1025`.
   - Integrated Desktop Inbox Viewer in the Tauri UI: split-pane layout with subject, sender, recipient, rich HTML preview, plain text, and raw MIME headers.

---

## PART 1: Implementation Roadmap

```
Phase 1: Tauri v2 Desktop Skeleton & System Tray
                     │
Phase 2: Cross-Platform Process Supervisor (Windows JobObjects / Unix Process Groups)
                     │
Phase 3: Multi-PHP FastCGI Pool Manager (8.2, 8.3, 8.4, 8.5) & Smart Composer Shim
                     │
Phase 4: Redis Service & Local Mail Catcher (SMTP Port 1025)
                     │
Phase 5: Elevated System Hosts Manager (UAC / Polkit / osascript)
                     │
Phase 6: Pure-Rust Root CA & Native Trust Store Provisioning (rcgen)
                     │
Phase 7: Auto-VHost Watcher & Nginx Dynamic Router (notify-debouncer-mini)
                     │
Phase 8: Desktop GUI (Dashboard, Multi-PHP Selector, Sites Manager, Live Logs, Mail Catcher)
                     │
Phase 9: Packaging & Release Validation (.msi/nsis, .dmg, .AppImage/.deb)
```

---

### Phase 1: Tauri v2 Desktop Foundation & Tray Lifecycle
- **Objective:** Scaffold a cross-platform desktop application that runs in the system tray and manages service lifecycles without auto-starting daemons.
- **Key Modules:**
  - `src-tauri/src/tray.rs`: Native system tray setup using `tauri::tray::TrayIconBuilder`.
  - Context menu layout:
    - Status Indicators: Nginx (⚪/🟢), Active PHP (8.2/8.3/8.4/8.5), MySQL (⚪/🟢), Redis (⚪/🟢), Mail Catcher (⚪/🟢)
    - PHP Version Submenu: `[✓] PHP 8.3`, `PHP 8.2`, `PHP 8.4`, `PHP 8.5`
    - Actions: `Start All`, `Stop All`, `Restart All`
    - Quick Links: `Open www/ Folder`, `Open Dashboard`, `Settings`, `Quit`
  - Window behavior: Intercept close event (`on_window_event`) to minimize/hide to system tray instead of killing background processes when services are running.
  - Single-instance enforcement: `@tauri-apps/plugin-single-instance` to prevent duplicate supervisors.

---

### Phase 2: Cross-Platform Process Supervisor Engine
- **Objective:** Supervise child processes (`nginx`, `php-fpm`, `mysql`, `redis`, `mailpit`) reliably across Windows 10/11, macOS, and Linux without leaving orphan zombie processes.
- **Strict Cold Start Policy:** No child processes spawn until explicitly commanded by the user.
- **Platform-Specific Subsystems:**
  - **Windows 10/11 (`supervisor/windows.rs`):**
    - Assign all spawned worker processes to a Windows **Job Object** configured with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`. When `laradoc-studio` exits, the Windows kernel forcefully and cleanly terminates all child processes automatically.
  - **macOS & Linux (`supervisor/unix.rs`):**
    - Spawn processes with `setpgid(0, 0)` to place them in dedicated process groups.
    - Signal shutdowns gracefully with `killpg(pgid, SIGTERM)` followed by `SIGKILL` timeout fallback.
    - Verify recorded PID files in `data/pids/` on startup and clean any stale processes.
  - **Pre-flight Port Conflict Check:**
    - Test port availability via `std::net::TcpListener::bind` on ports `80`, `443`, `3306`, `6379`, `1025`, `9082`, `9083`, `9084`, `9085`.
    - If occupied, use `sysinfo` to identify the existing process name and PID and report it to the desktop GUI.

---

### Phase 3: Multi-PHP Version Manager & Composer Alignment
- **Objective:** Support running and switching between PHP 8.2, 8.3, 8.4, and 8.5 concurrently, with automatic Composer alignment.
- **FastCGI Pool Orchestrator (`supervisor/php_manager.rs`):**
  - Manages isolated PHP-FPM pools for:
    - PHP 8.2 on port `9082`
    - PHP 8.3 on port `9083`
    - PHP 8.4 on port `9084`
    - PHP 8.5 on port `9085`
  - Maintains the global active version pointer (`bin/php/current` symlink or internal state).
- **Composer Smart Shim:**
  - `bin/composer` (Unix) and `bin/composer.bat` (Windows).
  - Inspects current working directory for `.php-version` or `composer.json` requirements.
  - Invokes `composer.phar` with the matched PHP binary.
  - In-app terminal action: "Open Terminal with Active PHP & Composer".

---

### Phase 4: Redis Service & Local Mail Catcher
- **Redis (`supervisor/service.rs`):**
  - Supervised child process running on `127.0.0.1:6379`.
  - Configured via `etc/redis/redis.conf` with isolated storage in `data/redis/`.
  - Reports memory consumption, connected client count, and uptime to GUI.
- **Mail Catcher (`mail/smtp.rs`):**
  - Embedded local SMTP listener on `127.0.0.1:1025`.
  - Parses incoming RFC 5322 MIME messages.
  - Stores captured emails in an in-memory ring buffer (up to 500 emails) and `data/mail/`.
  - Streams incoming emails directly to the desktop UI via Tauri events (`mail:received`).

---

### Phase 5: Elevated Hosts File Manager
- **Objective:** Safely manage domain mappings (`<project>.test -> 127.0.0.1`) requiring administrator/root privileges without crashing the unprivileged GUI.
- **Platform-Specific Elevation Handlers:**
  - **Windows 10/11:** PowerShell `Start-Process -Verb RunAs` or `ShellExecuteExW` targeting `laradoc-studio.exe --sync-hosts`.
  - **macOS:** `osascript -e 'do shell script "..." with administrator privileges'`.
  - **Linux:** `pkexec` to trigger the native Polkit authentication dialog.
- **Atomic Safety:** Read `hosts`, update only within `# BEGIN LARADOC-STUDIO` and `# END LARADOC-STUDIO` delimiters, and atomically write via a temporary file.

---

### Phase 6: Pure-Rust Local Root CA & Trust Store Installer
- **Objective:** Generate a dedicated local Root CA and sign `*.test` wildcard certificates so local sites load over HTTPS with a trusted green lock.
- **Implementation:**
  - Pure-Rust X.509 Root CA keypair and self-signed certificate generation using `rcgen`.
  - **Windows:** Auto-install via `certutil -addstore -f "Root" ca.crt`.
  - **macOS:** Auto-install via `security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain ca.crt`.
  - **Linux:** Copy to `/usr/local/share/ca-certificates/laradoc-studio-ca.crt` and run `update-ca-certificates`.
  - Wildcard certificate generation for `*.test` and `localhost` saved to `etc/ssl/wildcard.test.crt` and `etc/ssl/wildcard.test.key`.

---

### Phase 7: Auto-VirtualHost Generator & Directory Watcher
- **Objective:** Zero-configuration site provisioning when developers create directories in `www/`.
- **Implementation:**
  - Async directory monitoring on `www/` using `notify-debouncer-mini` (500ms debounce).
  - For each directory `www/<project>`, automatically generate an Nginx virtual host configuration (`etc/nginx/sites-enabled/<project>.test.conf`):
    - Document root auto-detection: checks for `<project>/public`, `<project>/web`, `<project>/dist`, `<project>/html`, falling back to `<project>/`.
    - FastCGI proxy pass to the selected PHP-FPM pool (8.2, 8.3, 8.4, or 8.5).
    - TLS certificates pointing to `wildcard.test.crt`.
  - Trigger graceful reload (`nginx -s reload`).
  - Automatically invoke the Hosts Manager to register `<project>.test`.

---

### Phase 8: Desktop GUI (Tauri Frontend)
- **Objective:** Modern, high-performance developer dashboard.
- **Design System:** Minimalist dark graphite aesthetic (`#121316` background, `#1A1C20` cards, `#22252B` elevated, `#10B981` emerald running state, `#EF4444` stopped/error state, `#F59E0B` warning state).
- **Core Views:**
  1. **Dashboard:**
     - Master "Start All" / "Stop All" primary action button.
     - Global PHP Version Switcher dropdown (`PHP 8.2`, `PHP 8.3`, `PHP 8.4`, `PHP 8.5`).
     - Service status cards (Nginx, PHP-FPM pools, MySQL, Redis, Mail Catcher) with uptime, CPU/RAM usage, and individual start/stop/restart toggles.
  2. **Sites / VirtualHosts:**
     - Filterable list of all detected projects in `www/`.
     - Per-site PHP version selector dropdown (instantly updates Nginx FastCGI routing).
     - Direct clickable links: `https://<site>.test`, root folder launcher, and configuration viewer.
  3. **Live Logs:**
     - Real-time streaming log viewer (Tauri event emitter streaming stdout/stderr of Nginx access/error logs and PHP errors).
     - Filter by log level and service; auto-scroll toggle and search.
  4. **Mail Catcher:**
     - Embedded inbox viewer for emails caught by the local SMTP server (`127.0.0.1:1025`).
     - Split view: email list on left, rich HTML preview / plain text / MIME headers on right.
  5. **Settings:**
     - Root CA installation status and 1-click install button.
     - Hosts file status and manual sync trigger.
     - Port configurations and binary path overrides.

---

### Phase 9: Packaging & Release Validation
- **Windows:** NSIS installer and standalone portable `.zip` with WebView2 bootstrapper.
- **macOS:** Universal `.dmg` (Apple Silicon & Intel) with signed `.app` bundle.
- **Linux:** Standalone `.AppImage` and `.deb` packages with desktop entry and tray indicators.

---

## PART 2: Project Identity & Technical Stack

- **Project Name:** Laradoc Studio
- **Binary / Package Name:** `laradoc-studio`
- **Application Type:** Cross-Platform Native Desktop Application (System Tray + Modern GUI + Background Process Supervisor)
- **Target Platforms:** Windows 10 & 11, macOS 12+ (Intel & Apple Silicon), Linux (X11 & Wayland)
- **Core Rust Crates:**
  - `tauri = { version = "2", features = ["tray-icon"] }`
  - `tauri-plugin-single-instance = "2"`
  - `tauri-plugin-opener = "2"`
  - `tokio = { version = "1", features = ["full"] }`
  - `notify = "6"`, `notify-debouncer-mini = "0.4"`
  - `minijinja = "2"`
  - `rcgen = "0.13"`, `rustls-pki-types = "1"`
  - `sysinfo = "0.31"`
  - `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`
  - `chrono = "0.4"`
  - Windows: `windows = { version = "0.58", features = ["Win32_UI_Shell", "Win32_System_JobObjects", "Win32_System_Threading"] }`
  - Unix: `nix = { version = "0.29", features = ["process", "signal"] }`, `libc = "0.2"`
- **Frontend Stack:** Vite + TypeScript + Vanilla CSS (No heavy framework dependencies, sub-millisecond tab switching, <30MB idle RAM).
