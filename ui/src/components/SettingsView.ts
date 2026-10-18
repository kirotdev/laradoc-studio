import { AppSettings, invokeCommand } from '../lib/tauri';
import { iconCheck, iconRotate } from '../lib/icons';

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
      <div style="max-width: 680px; display: flex; flex-direction: column; gap: 16px;">
        <!-- Local Root CA -->
        <div class="service-card">
          <div class="card-top">
            <span class="service-title">Local Root CA & SSL</span>
            <span class="badge ${this.settings.root_ca_trusted ? 'badge-running' : 'badge-warning'}">
              ${this.settings.root_ca_trusted ? 'Trusted' : 'Not Installed'}
            </span>
          </div>
          <div class="card-actions">
            <button class="btn btn-secondary btn-sm" id="btn-install-ca">
              ${this.settings.root_ca_trusted ? `${iconRotate()} Re-install Root CA` : `${iconCheck()} Install Root CA`}
            </button>
          </div>
        </div>

        <!-- Hosts File Sync -->
        <div class="service-card">
          <div class="card-top">
            <span class="service-title">System Hosts (*.test)</span>
            <span class="badge ${this.settings.hosts_synced ? 'badge-running' : 'badge-warning'}">
              ${this.settings.hosts_synced ? 'Synced' : 'Pending Sync'}
            </span>
          </div>
          <div class="card-actions">
            <button class="btn btn-secondary btn-sm" id="btn-sync-hosts">
              ${iconRotate()} Sync Hosts
            </button>
          </div>
        </div>

        <!-- Service Port Allocations -->
        <div class="service-card">
          <div class="card-top" style="margin-bottom: 4px;">
            <span class="service-title">Port Allocations</span>
          </div>
          <div class="card-stats" style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; border: none; padding: 4px 0;">
            <div>Nginx HTTP: <strong>:80</strong></div>
            <div>Nginx HTTPS: <strong>:443</strong></div>
            <div>MySQL: <strong>:3306</strong></div>
            <div>PHP 8.2: <strong>:9082</strong></div>
            <div>PHP 8.3: <strong>:9083</strong></div>
            <div>PHP 8.4: <strong>:9084</strong></div>
            <div>PHP 8.5: <strong>:9085</strong></div>
            <div>Redis: <strong>:6379</strong></div>
            <div>Mail Catcher: <strong>:1025</strong></div>
          </div>
        </div>
      </div>
    `;

    const btnCa = container.querySelector<HTMLButtonElement>('#btn-install-ca');
    if (btnCa) {
      btnCa.addEventListener('click', async () => {
        btnCa.setAttribute('disabled', 'true');
        try {
          await invokeCommand('install_root_ca');
          alert('Root CA installed successfully.');
          this.onAction();
        } catch (err) {
          alert('Root CA installation failed: ' + err);
        } finally {
          btnCa.removeAttribute('disabled');
        }
      });
    }

    const btnSyncHosts = container.querySelector<HTMLButtonElement>('#btn-sync-hosts');
    if (btnSyncHosts) {
      btnSyncHosts.addEventListener('click', async () => {
        btnSyncHosts.setAttribute('disabled', 'true');
        try {
          await invokeCommand('sync_hosts');
          alert('Hosts file synchronized.');
          this.onAction();
        } catch (err) {
          alert('Hosts sync failed: ' + err);
        } finally {
          btnSyncHosts.removeAttribute('disabled');
        }
      });
    }

    return container;
  }
}
