# Laragon-RS: Desktop Application Implementation Plan & Master Blueprint Prompt

> **Target Platforms:** Windows 10/11, macOS (Intel & Apple Silicon), Linux (Ubuntu, Debian, Fedora, Arch)  
> **Framework:** Tauri v2 (Rust Backend + Native Webview Frontend) + Native System Tray  
> **Architecture Reference:** Based strictly on [`AI APP BUILDING BLUEPRINT.md`](file:///Users/myatkyawthu/Downloads/vox/Prompt-BP/AI%20APP%20BUILDING%20BLUEPRINT.md)

---

## PART 1: Desktop Application Implementation Roadmap

```
Phase 1: Tauri v2 Desktop Skeleton & System Tray
                     │
Phase 2: Cross-Platform Process Supervisor (Windows JobObjects / Unix Process Groups)
                     │
Phase 3: Elevated System Hosts Manager (UAC / Polkit / osascript)
                     │
Phase 4: Pure-Rust Root CA & Native Trust Store Provisioning
                     │
Phase 5: Auto-VHost Watcher & Nginx/Caddy Dynamic Router
                     │
Phase 6: Desktop GUI (Service Dashboard, Live Log Streamer, Site Manager, Mail Catcher)
                     │
Phase 7: Packaging & Release Validation (.msi/nsis, .dmg, .AppImage/.deb)
```

### Phase 1: Tauri v2 Desktop Foundation & Tray Lifecycle
- **Objective:** Scaffold a cross-platform desktop application that runs in the background and controls services via a native System Tray menu.
- **Key Modules:**
  - `src-tauri/src/tray.rs`: Native system tray setup using `tauri::tray::TrayIconBuilder`.
  - Left-click toggles the main window visibility; right-click displays quick action context menu:
    - Status Indicators: Nginx (🟢), PHP-FPM (🟢), MySQL (🟢), Redis (⚪)
    - Actions: `Start All`, `Stop All`, `Restart All`
    - Quick Links: `Open www/ Folder`, `Open Localhost`, `Settings`, `Quit`
  - Window behavior: Intercept close event (`on_window_event`) to minimize/hide to system tray instead of terminating the background daemons.
  - Single-instance enforcement: `@tauri-apps/plugin-single-instance` to prevent multiple daemon supervisors from colliding.

---

### Phase 2: Cross-Platform Process Supervisor Engine
- **Objective:** Supervise child processes (`nginx`, `php-fpm`, `mysql`, `redis`, `mailpit`) reliably across Windows 10/11, macOS, and Linux without leaving orphan zombie processes on crash or exit.
- **Platform-Specific Subsystems:**
  - **Windows 10/11 (`supervisor/windows.rs`):**
    - Assign all spawned worker processes to a Windows **Job Object** configured with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`. When `laragon-rs` exits, the Windows kernel forcefully and cleanly terminates all child processes automatically.
  - **macOS & Linux (`supervisor/unix.rs`):**
    - Spawn processes with `setpgid(0, 0)` to place them in dedicated process groups.
    - Signal shutdowns gracefully with `killpg(pgid, SIGTERM)` followed by `SIGKILL` timeout fallback.
  - **Pre-flight Port Conflict Check:**
    - Test port availability via `std::net::TcpListener::bind` on ports `80`, `443`, `3306`, `5432`, `6379`, `1025`.
    - If occupied, use `sysinfo` to identify the existing process name and PID and report it to the desktop GUI.

---

### Phase 3: Elevated Hosts File Manager
- **Objective:** Safely manage domain mappings (`<project>.test -> 127.0.0.1`) requiring administrator/root privileges without crashing the unprivileged GUI.
- **Platform-Specific Elevation Handlers:**
  - **Windows 10/11:** Use `ShellExecuteExW` with verb `"runas"` or `powershell Start-Process -Verb RunAs` targeting an internal helper command `laragon-rs.exe --sync-hosts`.
  - **macOS:** Execute `osascript -e 'do shell script "..." with administrator privileges'` to prompt the native macOS TouchID/password dialog.
  - **Linux:** Invoke `pkexec` to trigger the native Polkit authentication dialog.
- **Atomic Safety:** Read `hosts`, update only within `# BEGIN LARAGON-RS` and `# END LARAGON-RS` delimiters, and atomically write via a temporary file.

---

### Phase 4: Pure-Rust Local Root CA & Trust Store Installer
- **Objective:** Generate a dedicated local Root CA and sign `*.test` wildcard certificates so local sites load over HTTPS with a trusted green lock.
- **Implementation:**
  - Use `rcgen` to generate an unconstrained X.509 Root CA keypair and self-signed certificate.
  - **Windows:** Auto-install via `certutil -addstore -f "Root" ca.crt`.
  - **macOS:** Auto-install via `security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain ca.crt`.
  - **Linux:** Copy to `/usr/local/share/ca-certificates/laragon-rs-ca.crt` and run `update-ca-certificates`.
  - Sign `*.test` certificates on demand and save to `etc/ssl/wildcard.test.crt` and `etc/ssl/wildcard.test.key`.

---

### Phase 5: Auto-VirtualHost Generator & Directory Watcher
- **Objective:** Zero-configuration site provisioning when developers create directories in `www/`.
- **Implementation:**
  - Async directory monitoring on `www/` using `notify-debouncer-mini` (500ms debounce).
  - For each directory `www/<project>`, automatically generate an Nginx virtual host configuration (`etc/nginx/sites-enabled/<project>.test.conf`) with:
    - Automatic root folder detection: checks if `<project>/public` or `<project>/web` exists; falls back to `<project>/`.
    - FastCGI proxy pass to active PHP-FPM socket or TCP port `127.0.0.1:9000`.
    - TLS certificates pointing to `wildcard.test.crt`.
  - Trigger graceful reload (`nginx -s reload`).
  - Automatically invoke the Hosts Manager to register `<project>.test`.

---

### Phase 6: Desktop GUI (Tauri Frontend)
- **Objective:** A polished, modern, distraction-free desktop dashboard.
- **Design System:** Minimalist dark mode with graphite backgrounds, subtle borders, and vivid status indicators.
- **Key Tabs / Views:**
  1. **Dashboard:**
     - Master "Start All" / "Stop All" primary action button.
     - Service status cards (Nginx, PHP, MySQL, Redis, Mailpit) with uptime, CPU/RAM usage, and individual restart buttons.
  2. **Sites / VirtualHosts:**
     - Filterable list of all detected projects.
     - Direct clickable links: `https://app.test`, root folder launcher, and custom PHP version selector.
  3. **Live Logs:**
     - Real-time streaming log viewer (Tauri event emitter streaming stdout/stderr of Nginx access/error logs and PHP errors).
  4. **Mail Catcher:**
     - Embedded inbox viewer for emails caught by the local SMTP server (`127.0.0.1:1025`).

---

### Phase 7: Packaging & Cross-Platform Distribution
- **Windows:** NSIS installer and standalone portable `.zip` with WebView2 bootstrapper.
- **macOS:** Universal `.dmg` (Apple Silicon & Intel) with signed `.app` bundle.
- **Linux:** Standalone `.AppImage` and `.deb` packages with desktop entry and tray indicators.

---

## PART 2: Master AI Application-Building Prompt

> **Instructions:** Copy the complete prompt below into your AI builder or coding assistant to execute the implementation.

```markdown
You are a Principal Systems Engineer, Desktop Application Architect, and Rust Specialist.

Build a complete, production-grade cross-platform desktop application in Rust using **Tauri v2** called **"Laragon-RS"**, an open-source, lightweight, cross-platform successor to Laragon supporting **Windows 10/11, macOS, and Linux**.

---

### 1. PROJECT IDENTITY
- **Project Name:** Laragon-RS (Desktop App: `laragon-rs`)
- **Application Type:** Cross-Platform Native Desktop Application (System Tray + Modern GUI + Background Process Supervisor)
- **Target Platforms:**
  - Windows 10 & 11 (64-bit, ARM64)
  - macOS 12+ (Apple Silicon & Intel x86_64)
  - Linux (Ubuntu, Debian, Fedora, Arch via X11 & Wayland)
- **Target Users:** Full-stack, PHP, Laravel, WordPress, Python, and Node.js developers who need an instant, isolated, non-Docker local development environment with automatic virtual hosts (`*.test`) and trusted SSL.
- **Core Value Proposition:** Single-executable portable desktop application with native system tray control, automatic virtual hosts, zero-warning local SSL certificates, instant version switching, and real-time process monitoring consuming <30MB idle RAM.

---

### 2. CORE FUNCTION
USER INPUT → CORE PROCESS → EXPECTED OUTPUT
- **User Input:**
  1. User launches `Laragon-RS` (resides in system tray with desktop GUI).
  2. User clicks "Start All" or drops a project folder into `www/<project-name>`.
- **Core Process:**
  1. The Rust backend supervises local runtime binaries (`nginx`, `php-fpm`, `mysql`, `redis`, `mailpit`) using platform-native process groups (Windows Job Objects on Win10/11, process groups on macOS/Linux).
  2. Detects folder creation in `www/` via filesystem events.
  3. Requests elevated privilege once to register `<project-name>.test` inside the OS `hosts` file using safe delimiter markers.
  4. Provisions an X.509 wildcard certificate signed by an auto-installed local Root CA.
  5. Generates the Nginx vhost config (auto-detecting `public/` directories) and reloads Nginx.
- **Expected Output:**
  - The project is immediately live and reachable in any browser at `https://<project-name>.test` with trusted TLS.
  - The desktop GUI displays green health indicators, active port listings, and live streaming logs.
- **Definition of Success:** All services run stably, browsers display a valid trusted lock icon with zero security warnings, system hosts remain uncorrupted, and closing the app cleanly cleans up all child processes without orphan PIDs.

---

### 3. PRODUCT BOUNDARIES
- **Essential Functionality:**
  - Native System Tray with service toggles and live status indicators.
  - Desktop GUI with Service Cards, Site Manager, Live Logs Viewer, and Mail Catcher.
  - Windows Job Objects implementation to guarantee zero orphan processes on Windows 10/11.
  - Unix process group supervision (`killpg`) with graceful `SIGTERM` and `SIGKILL` escalation on macOS/Linux.
  - Safe, atomic delimited hosts file updater (`# BEGIN / # END LARAGON-RS`).
  - Native privilege elevation: UAC (`ShellExecuteExW` / `runas`) on Windows, `osascript` on macOS, and `pkexec` on Linux.
  - Pure-Rust X.509 local Root CA generator and OS trust store installer (`rcgen`).
  - Async filesystem watcher on `www/` with 500ms debounce.
- **Explicit Exclusions (Must NOT be implemented):**
  - Do NOT implement Docker, Kubernetes, or container runtimes.
  - Do NOT use heavy Electron or Chromium browser processes; use Tauri v2 (native OS webviews: WebView2 on Windows, WKWebView on macOS, WebKitGTK on Linux).
  - Do NOT permanently modify global system `PATH` or Windows Registry keys.
  - Do NOT require cloud accounts, telemetry, or internet access for core operations.

---

### 4. VISUAL DESIGN SYSTEM (DESKTOP GUI)
- **Aesthetic:** Minimalist, high-performance developer tool aesthetic. Dark graphite backgrounds, crisp typography, subtle borders, and purposeful accent colors.
- **Color Palette:**
  - Primary Background: Dark Graphite (`#121316`)
  - Secondary / Card Background: Deep Charcoal (`#1A1C20`)
  - Elevated Surfaces & Modals: (`#22252B`)
  - Borders & Dividers: Subtle Slate (`#2E323B`)
  - Primary Accent / Action: Vivid Emerald Green (`#10B981`) for Running states and primary actions.
  - Danger / Error: Coral Red (`#EF4444`)
  - Warning / Port Conflict: Amber (`#F59E0B`)
  - Text Primary: Creamy White (`#F3F4F6`)
  - Text Secondary: Muted Gray (`#9CA3AF`)
- **Typography:**
  - UI Font: System sans-serif (`Inter`, `system-ui`, `-apple-system`, `Segoe UI`, `Roboto`)
  - Monospace (Logs, Ports, Paths): `JetBrains Mono`, `Fira Code`, `Consolas`, `monospace`
- **Layout & Interaction:**
  - Compact window (900x600 default size, resizable with minimum 750x500).
  - Left navigation sidebar: Dashboard, Sites, Logs, Mail, Settings.
  - Smooth hover states, micro-animations on service state toggles, and instant tab transitions.
  - Close-to-Tray: Clicking the window `X` minimizes to the System Tray rather than quitting the background services.

---

### 5. USER WORKFLOW
1. **Launch:** Application opens into the system tray with a native desktop window. If run on first start, it prompts to install the local Root CA once.
2. **One-Click Start:** User clicks "Start All". Nginx, PHP-FPM, and MySQL launch. Status indicators switch from Stopped (⚪) to Running (🟢).
3. **Project Creation:** User clones or creates a project in `www/my-api`.
4. **Auto-Discovery:** Within 500ms, the desktop app detects the folder, requests elevation (if domain not yet in hosts), writes the virtual host, reloads Nginx, and pushes an OS desktop notification: `"my-api.test is ready at https://my-api.test"`.
5. **Inspection:** User switches to the "Logs" tab in the GUI to view live streaming PHP errors or Nginx access logs.
6. **Email Testing:** Emails sent via PHP `mail()` appear instantly in the "Mail" tab without reaching the internet.
7. **Tray Control:** User closes the window. Background processes continue. User right-clicks tray icon to stop services or quit.

---

### 6. TECHNICAL ARCHITECTURE & STACK
- **Core Desktop Framework:** Tauri v2 (`tauri = { version = "2", features = ["tray-icon"] }`)
- **Async Runtime:** `tokio = { version = "1", features = ["full"] }`
- **Filesystem Watcher:** `notify = "6"`, `notify-debouncer-mini = "0.4"`
- **Template Engine:** `minijinja = "2"`
- **Cryptography / PKI:** `rcgen = "0.13"`, `rustls-pki-types = "1"`
- **System & Process Auditing:** `sysinfo = "0.31"`
- **Cross-Platform Elevation:**
  - Windows: `windows = { version = "0.58", features = ["Win32_UI_Shell", "Win32_System_JobObjects", "Win32_System_Threading"] }`
  - macOS: `osascript` standard command integration
  - Linux: `pkexec` Polkit wrapper
- **Frontend:** Lightweight TypeScript + Tailwind CSS / Vanilla CSS (compiled with Vite).

#### Module Organization:
```text
src-tauri/
├── src/
│   ├── main.rs                  # Tauri entrypoint & initialization
│   ├── lib.rs                   # Tauri application setup, plugins, and handlers
│   ├── tray.rs                  # System tray setup, context menus, and event handlers
│   ├── supervisor/              # Multi-process supervisor
│   │   ├── mod.rs
│   │   ├── service.rs           # Generic Service trait & state tracking
│   │   ├── windows.rs           # Windows Job Object implementation
│   │   └── unix.rs              # Unix process group & signal handler
│   ├── hosts/                   # Cross-platform hosts file manager
│   │   ├── mod.rs
│   │   └── elevation.rs         # UAC / osascript / pkexec elevation wrappers
│   ├── pki/                     # Local Root CA & Trust Store provisioning
│   │   ├── ca.rs                # Pure-Rust X.509 generation via rcgen
│   │   └── store.rs             # Windows certutil, macOS Keychain, Linux ca-certificates
│   ├── vhost/                   # Dynamic virtual host manager
│   │   ├── watcher.rs           # notify-debouncer on www/
│   │   └── template.rs          # minijinja Nginx/Caddy template renderer
│   ├── mail/                    # Embedded local SMTP catcher (port 1025)
│   │   └── smtp.rs
│   └── commands/                # Tauri IPC command handlers
│       ├── services.rs          # start_service, stop_service, get_status
│       ├── sites.rs             # list_sites, open_in_browser, open_folder
│       └── logs.rs              # stream_logs via Tauri events
└── tauri.conf.json              # Tauri v2 desktop & tray configuration
ui/                              # Clean, modern desktop webview interface
├── index.html
├── src/
│   ├── App.svelte / App.tsx     # Main dashboard layout & tabs
│   ├── components/              # ServiceCard, SiteTable, LogViewer, MailViewer
│   └── lib/tauri.ts             # Typed IPC wrappers around invoke and listen
```

---

### 7. EDGE CASES & RECOVERY BEHAVIOR
1. **Port Occupied by Existing App (e.g. Apache, IIS, or Skype on Port 80/443):**
   - The app must NOT crash.
   - Detect the occupied port before spawning.
   - Inspect the owning process PID and executable name via `sysinfo`.
   - Show a clear warning badge on the service card: `"Port 80 occupied by IIS (PID: 4120). Click to change port or stop conflicting app."`
2. **User Cancels or Rejects Privilege Elevation:**
   - If UAC / TouchID / Polkit is dismissed, do NOT crash.
   - Flag the domain as `"Hosts update pending elevation"`.
   - Provide a 1-click button: `"Retry Elevation"` or `"Copy Hosts Lines Manually"`.
3. **Application Crash / Force Quit:**
   - On Windows: The kernel-backed Job Object automatically terminates all children when the desktop handle is closed.
   - On Unix: On startup, inspect recorded PID files in `data/pids/`. If processes are still alive, verify identity and cleanly terminate them before spawning fresh daemons.
4. **Laptop Sleep & Wake:**
   - Detect system wake events and verify that supervised TCP ports are still responding; trigger health-check reconnects without restarting healthy processes.

---

### 8. DEFINITION OF DONE
1. Compiles with `cargo check` and `tauri build` on target platforms without errors.
2. The System Tray icon renders natively with live status indicators and working context menus.
3. Closing the desktop window minimizes to tray; quitting from tray terminates all child processes cleanly.
4. Windows Job Objects verify that terminating `laragon-rs` leaves zero orphaned Nginx or PHP background processes in Task Manager.
5. Creating a folder in `www/` automatically provisions hosts, SSL, and vhost routing.
6. Browsing to `https://<folder>.test` in Chrome, Edge, and Firefox displays a verified trusted TLS connection.
7. Memory usage idles below 30 MB when minimized to tray.

---

### 9. IMPLEMENTATION INSTRUCTIONS
1. Initialize the Tauri v2 desktop structure with `cargo` and Vite.
2. Implement backend modules in strict logical order:
   `supervisor` (with JobObjects/Unix groups) → `pki` (Root CA) → `hosts` (with elevation) → `vhost` (watcher) → `tray` → `ui`.
3. Provide complete, fully compilable Rust code without missing imports or placeholder pseudo-functions.
4. Ensure cross-platform conditional compilation attributes (`#[cfg(target_os = "...")]`) are cleanly separated.
```
