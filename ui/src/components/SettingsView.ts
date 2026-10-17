import { AppSettings, invokeCommand } from '../lib/tauri';

export class SettingsView {
  private settings: AppSettings = {
    active_php: '8.3',
    www_directory: 'www',
    root_ca_trusted: true,
    hosts_synced: true,
    auto_start_disabled: true,
  };
  private onAction: () => void;

  constructor(onAction: () => void) {
    this.onAction = onAction;
  }

  public update(settings: AppSettings) {
    this.settings = settings;
  }

  public render(): HTMLElement {
    const container = document.createElement('div');
    container.className = 'view-content';

    container.innerHTML = `
      <div style="max-width: 760px; display: flex; flex-direction: column; gap: 20px;">
        <!-- Root CA & SSL -->
        <div class="service-card">
          <div class="card-header">
            <div>
              <div class="service-name">Local Root CA & Wildcard SSL</div>
              <p style="font-size: 12px; color: var(--text-muted); margin-top: 2px;">
                Pure-Rust X.509 Root CA generates trusted <code>*.test</code> wildcard certificates automatically.
              </p>
            </div>
            <span class="badge ${this.settings.root_ca_trusted ? 'badge-running' : 'badge-warning'}">
              ${this.settings.root_ca_trusted ? '🟢 Trusted in OS' : '🟠 Not Installed'}
            </span>
          </div>
          <div class="card-footer">
            <button class="btn btn-secondary btn-sm" id="btn-install-ca">
              ${this.settings.root_ca_trusted ? 'Re-install CA' : 'Install Root CA to Keychain / Trust Store'}
            </button>
          </div>
        </div>

        <!-- System Hosts Delimiter -->
        <div class="service-card">
          <div class="card-header">
            <div>
              <div class="service-name">Atomic Delimited Hosts File</div>
              <p style="font-size: 12px; color: var(--text-muted); margin-top: 2px;">
                Safely maps <code>*.test</code> domains inside <code># BEGIN LARADOC-STUDIO</code> and <code># END LARADOC-STUDIO</code> blocks using native OS elevation.
              </p>
            </div>
            <span class="badge badge-running">🟢 Synced</span>
          </div>
          <div class="card-footer">
            <button class="btn btn-secondary btn-sm" id="btn-sync-hosts">Force Re-sync Hosts</button>
          </div>
        </div>

        <!-- Composer & PHP Shims -->
        <div class="service-card">
          <div class="card-header">
            <div>
              <div class="service-name">Smart Composer Shim & Environment</div>
              <p style="font-size: 12px; color: var(--text-muted); margin-top: 2px;">
                <code>bin/composer</code> auto-detects PHP constraints from <code>composer.json</code> and routes directly to the matched PHP binary (8.2, 8.3, 8.4, 8.5).
              </p>
            </div>
          </div>
          <div style="background: var(--bg-elevated); padding: 10px; border-radius: var(--radius-sm); font-family: var(--font-mono); font-size: 11px;">
            <div>Global Active CLI: PHP ${this.settings.active_php}</div>
            <div style="color: var(--text-muted); margin-top: 4px;">Shim: /bin/composer -&gt; bin/php/current/bin/php</div>
          </div>
        </div>

        <!-- Port Allocations -->
        <div class="service-card">
          <div class="card-header">
            <div class="service-name">Pre-Allocated Service Ports</div>
          </div>
          <div class="card-metrics" style="grid-template-columns: repeat(3, 1fr);">
            <div class="metric-item"><span>Nginx HTTP</span><strong>:80</strong></div>
            <div class="metric-item"><span>Nginx HTTPS</span><strong>:443</strong></div>
            <div class="metric-item"><span>MySQL</span><strong>:3306</strong></div>
            <div class="metric-item"><span>PHP 8.2 Pool</span><strong>:9082</strong></div>
            <div class="metric-item"><span>PHP 8.3 Pool</span><strong>:9083</strong></div>
            <div class="metric-item"><span>PHP 8.4 Pool</span><strong>:9084</strong></div>
            <div class="metric-item"><span>PHP 8.5 Pool</span><strong>:9085</strong></div>
            <div class="metric-item"><span>Redis Cache</span><strong>:6379</strong></div>
            <div class="metric-item"><span>Mail Catcher (SMTP)</span><strong>:1025</strong></div>
          </div>
        </div>
      </div>
    `;

    const btnCa = container.querySelector('#btn-install-ca');
    if (btnCa) {
      btnCa.addEventListener('click', async () => {
        try {
          await invokeCommand('install_root_ca');
          alert('Root CA successfully installed into local trust store!');
          this.onAction();
        } catch (err) {
          alert('Root CA install failed: ' + err);
        }
      });
    }

    const btnSyncHosts = container.querySelector('#btn-sync-hosts');
    if (btnSyncHosts) {
      btnSyncHosts.addEventListener('click', async () => {
        try {
          await invokeCommand('sync_hosts');
          alert('Hosts file successfully synchronized!');
          this.onAction();
        } catch (err) {
          alert('Hosts synchronization failed: ' + err);
        }
      });
    }

    return container;
  }
}
