use std::path::PathBuf;

/// Ensure the internal phpMyAdmin virtual host and application directory exist
pub fn ensure_phpmyadmin_vhost() -> Result<(), String> {
    let app_data = dirs::data_local_dir()
        .unwrap_or_default()
        .join("laradoc-studio");

    let apps_dir = app_data.join("apps").join("phpmyadmin");
    let vhosts_dir = app_data.join("data").join("vhosts");
    let ssl_dir = app_data.join("ssl").join("certs");

    let _ = std::fs::create_dir_all(&apps_dir);
    let _ = std::fs::create_dir_all(&vhosts_dir);

    // 1. Generate config.inc.php for auto-login
    let config_inc = apps_dir.join("config.inc.php");
    if !config_inc.exists() {
        let config_content = r#"<?php
declare(strict_types=1);

$cfg['blowfish_secret'] = 'laradocstudio_secret_key_32bytes_min_123456';

$i = 0;
$i++;
$cfg['Servers'][$i]['auth_type'] = 'config';
$cfg['Servers'][$i]['host'] = '127.0.0.1';
$cfg['Servers'][$i]['port'] = '3306';
$cfg['Servers'][$i]['user'] = 'root';
$cfg['Servers'][$i]['password'] = '';
$cfg['Servers'][$i]['AllowNoPassword'] = true;
$cfg['Servers'][$i]['compress'] = false;
$cfg['Servers'][$i]['ssl'] = false;

$cfg['UploadDir'] = '';
$cfg['SaveDir'] = '';
"#;
        let _ = std::fs::write(&config_inc, config_content);
    }

    // 2. Generate embedded single-file DB manager index.php if not present
    let index_php = apps_dir.join("index.php");
    if !index_php.exists() {
        let manager_code = r#"<?php
/**
 * Laradoc Studio — Built-in Database Manager (phpmyadmin.test)
 * Automatically connects to local MySQL (127.0.0.1:3306) with root user
 */
$host = '127.0.0.1';
$port = 3306;
$user = 'root';
$pass = '';

$error = null;
$pdo = null;
try {
    $pdo = new PDO("mysql:host=$host;port=$port", $user, $pass, [
        PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
        PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
    ]);
} catch (PDOException $e) {
    $error = $e->getMessage();
}

$db = $_GET['db'] ?? null;
$table = $_GET['table'] ?? null;
$action = $_POST['action'] ?? null;

if ($pdo && $action === 'create_db') {
    $newDb = preg_replace('/[^a-zA-Z0-9_]/', '', $_POST['new_db'] ?? '');
    if ($newDb) {
        $pdo->exec("CREATE DATABASE `$newDb` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci");
        header("Location: ?db=$newDb");
        exit;
    }
}

if ($pdo && $action === 'query') {
    $sql = trim($_POST['sql'] ?? '');
    if ($sql) {
        try {
            if ($db) $pdo->exec("USE `$db`");
            $stmt = $pdo->query($sql);
            $queryResults = $stmt ? $stmt->fetchAll() : [];
            $querySuccess = "Query executed successfully.";
        } catch (PDOException $e) {
            $queryError = $e->getMessage();
        }
    }
}

$databases = [];
if ($pdo) {
    $databases = $pdo->query("SHOW DATABASES")->fetchAll(PDO::FETCH_COLUMN);
}

$tables = [];
if ($pdo && $db) {
    $pdo->exec("USE `$db`");
    $tables = $pdo->query("SHOW TABLES")->fetchAll(PDO::FETCH_COLUMN);
}

$tableRows = [];
$columns = [];
if ($pdo && $db && $table) {
    $pdo->exec("USE `$db`");
    $stmt = $pdo->query("SELECT * FROM `$table` LIMIT 100");
    $tableRows = $stmt->fetchAll();
    if (!empty($tableRows)) {
        $columns = array_keys($tableRows[0]);
    } else {
        $colStmt = $pdo->query("DESCRIBE `$table`");
        $columns = $colStmt->fetchAll(PDO::FETCH_COLUMN);
    }
}
?>
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Database Manager — phpmyadmin.test</title>
    <style>
        :root {
            --bg-canvas: #f8fafc;
            --bg-card: #ffffff;
            --border: #e2e8f0;
            --text-main: #0f172a;
            --text-sub: #64748b;
            --accent: #10b981;
        }
        * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; font-size: 13px; }
        body { display: flex; height: 100vh; background: var(--bg-canvas); color: var(--text-main); }
        .sidebar { width: 230px; background: #fff; border-right: 1px solid var(--border); display: flex; flex-direction: column; }
        .sidebar-brand { padding: 16px; border-bottom: 1px solid var(--border); font-weight: 700; font-size: 14px; display: flex; align-items: center; justify-content: space-between; }
        .db-list { flex: 1; overflow-y: auto; padding: 8px; }
        .db-item { display: block; padding: 7px 10px; border-radius: 6px; text-decoration: none; color: var(--text-sub); font-weight: 500; margin-bottom: 2px; }
        .db-item:hover, .db-item.active { background: #f1f5f9; color: var(--text-main); }
        .main { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
        .topbar { height: 50px; background: #fff; border-bottom: 1px solid var(--border); display: flex; align-items: center; justify-content: space-between; padding: 0 20px; }
        .content { flex: 1; overflow-y: auto; padding: 20px; }
        .card { background: #fff; border: 1px solid var(--border); border-radius: 10px; padding: 18px; margin-bottom: 16px; box-shadow: 0 1px 3px rgba(0,0,0,0.03); }
        .table { width: 100%; border-collapse: collapse; text-align: left; }
        .table th, .table td { padding: 9px 12px; border-bottom: 1px solid var(--border); }
        .table th { background: #f8fafc; font-size: 11.5px; text-transform: uppercase; color: var(--text-sub); }
        .badge { display: inline-block; padding: 2px 7px; border-radius: 9999px; font-size: 11px; font-weight: 600; background: #ecfdf5; color: #047857; }
        .btn { padding: 6px 12px; background: #0f172a; color: #fff; border: none; border-radius: 6px; cursor: pointer; font-weight: 500; }
        .btn:hover { background: #1e293b; }
        .input { padding: 6px 10px; border: 1px solid var(--border); border-radius: 6px; outline: none; }
    </style>
</head>
<body>
    <div class="sidebar">
        <div class="sidebar-brand">
            <span>MySQL Local</span>
            <span class="badge">:3306</span>
        </div>
        <div class="db-list">
            <div style="font-size: 11px; text-transform: uppercase; color: var(--text-sub); padding: 8px 10px; font-weight: 600;">Databases</div>
            <?php foreach ($databases as $d): ?>
                <a href="?db=<?= urlencode($d) ?>" class="db-item <?= $db === $d ? 'active' : '' ?>">
                    📁 <?= htmlspecialchars($d) ?>
                </a>
            <?php endforeach; ?>
        </div>
        <div style="padding: 12px; border-top: 1px solid var(--border);">
            <form method="POST" style="display: flex; gap: 6px;">
                <input type="hidden" name="action" value="create_db">
                <input type="text" name="new_db" placeholder="New DB name" class="input" style="flex: 1;" required>
                <button type="submit" class="btn">+</button>
            </form>
        </div>
    </div>

    <div class="main">
        <div class="topbar">
            <strong><?= $db ? htmlspecialchars($db) . ($table ? ' &rarr; ' . htmlspecialchars($table) : '') : 'Database Overview' ?></strong>
            <span style="color: var(--text-sub); font-size: 12px;">phpmyadmin.test (User: root)</span>
        </div>
        <div class="content">
            <?php if ($error): ?>
                <div class="card" style="border-color: #fecaca; background: #fef2f2; color: #991b1b;">
                    <strong>Database Connection Error:</strong> <?= htmlspecialchars($error) ?>
                </div>
            <?php endif; ?>

            <?php if ($db && !$table): ?>
                <div class="card">
                    <h3 style="margin-bottom: 12px; font-size: 14px;">Tables in <?= htmlspecialchars($db) ?></h3>
                    <?php if (empty($tables)): ?>
                        <p style="color: var(--text-sub);">No tables in this database.</p>
                    <?php else: ?>
                        <table class="table">
                            <thead><tr><th>Table Name</th><th>Action</th></tr></thead>
                            <tbody>
                                <?php foreach ($tables as $t): ?>
                                    <tr>
                                        <td><strong><?= htmlspecialchars($t) ?></strong></td>
                                        <td><a href="?db=<?= urlencode($db) ?>&table=<?= urlencode($t) ?>" style="color: #2563eb; text-decoration: none;">Browse Data &rarr;</a></td>
                                    </tr>
                                <?php endforeach; ?>
                            </tbody>
                        </table>
                    <?php endif; ?>
                </div>
            <?php elseif ($db && $table): ?>
                <div class="card">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                        <h3 style="font-size: 14px;">Table: <?= htmlspecialchars($table) ?> (First 100 rows)</h3>
                        <a href="?db=<?= urlencode($db) ?>" style="color: var(--text-sub); text-decoration: none;">&larr; Back to Tables</a>
                    </div>
                    <?php if (empty($tableRows)): ?>
                        <p style="color: var(--text-sub);">Table is empty.</p>
                    <?php else: ?>
                        <div style="overflow-x: auto;">
                            <table class="table">
                                <thead>
                                    <tr>
                                        <?php foreach ($columns as $c): ?>
                                            <th><?= htmlspecialchars($c) ?></th>
                                        <?php endforeach; ?>
                                    </tr>
                                </thead>
                                <tbody>
                                    <?php foreach ($tableRows as $row): ?>
                                        <tr>
                                            <?php foreach ($columns as $c): ?>
                                                <td><?= htmlspecialchars((string)($row[$c] ?? 'NULL')) ?></td>
                                            <?php endforeach; ?>
                                        </tr>
                                    <?php endforeach; ?>
                                </tbody>
                            </table>
                        </div>
                    <?php endif; ?>
                </div>
            <?php else: ?>
                <div class="card">
                    <h3 style="font-size: 14px; margin-bottom: 8px;">MySQL Database Service</h3>
                    <p style="color: var(--text-sub); line-height: 1.6;">
                        Connected to local MySQL database on <strong>127.0.0.1:3306</strong>.<br>
                        Select a database from the sidebar to inspect tables or execute queries.
                    </p>
                </div>
            <?php endif; ?>

            <div class="card">
                <h3 style="font-size: 14px; margin-bottom: 8px;">SQL Query Console</h3>
                <form method="POST">
                    <input type="hidden" name="action" value="query">
                    <textarea name="sql" style="width: 100%; height: 80px; font-family: monospace; padding: 10px; border: 1px solid var(--border); border-radius: 6px; margin-bottom: 8px;" placeholder="SELECT * FROM ..."></textarea>
                    <button type="submit" class="btn">Execute Query</button>
                </form>
            </div>
        </div>
    </div>
</body>
</html>
"#;
        let _ = std::fs::write(&index_php, manager_code);
    }

    // 3. Generate Nginx virtual host for phpmyadmin.test
    let vhost_conf = vhosts_dir.join("00-phpmyadmin.test.conf");
    let conf_content = format!(
        r#"# Auto-generated by Laradoc Studio for built-in database management
server {{
    listen 80;
    server_name phpmyadmin.test;
    return 301 https://$host$request_uri;
}}

server {{
    listen 443 ssl;
    http2 on;
    server_name phpmyadmin.test;

    root "{apps_dir}";
    index index.php index.html;

    ssl_certificate {ssl_dir}/wildcard.test.crt;
    ssl_certificate_key {ssl_dir}/wildcard.test.key;

    location / {{
        try_files $uri $uri/ /index.php?$query_string;
    }}

    location ~ \.php$ {{
        fastcgi_pass 127.0.0.1:9083;
        fastcgi_index index.php;
        fastcgi_param SCRIPT_FILENAME $realpath_root$fastcgi_script_name;
        include fastcgi_params;
    }}

    access_log off;
    error_log /dev/null crit;
}}
"#,
        apps_dir = apps_dir.to_string_lossy(),
        ssl_dir = ssl_dir.to_string_lossy()
    );

    let _ = std::fs::write(&vhost_conf, conf_content);

    // 4. Register in system hosts file
    let _ = crate::hosts::add_domain("phpmyadmin.test");

    log::info!("Configured phpmyadmin.test vhost and directory at {:?}", apps_dir);
    Ok(())
}
