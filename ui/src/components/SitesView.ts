import { SiteInfo, invokeCommand } from '../lib/tauri';

export class SitesView {
  private sites: SiteInfo[] = [];
  private onAction: () => void;

  constructor(onAction: () => void) {
    this.onAction = onAction;
  }

  public update(sites: SiteInfo[]) {
    this.sites = sites;
  }

  public render(): HTMLElement {
    const container = document.createElement('div');
    container.className = 'view-content';

    container.innerHTML = `
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
        <div>
          <h3 style="font-size: 14px; font-weight: 700;">Managed Virtual Hosts</h3>
          <p style="font-size: 12px; color: var(--text-muted);">
            Folders in <code>www/</code> are automatically provisioned as <code>*.test</code> domains with trusted SSL.
          </p>
        </div>
        <button class="btn btn-secondary btn-sm" id="btn-rescan-sites">🔄 Rescan www/</button>
      </div>

      <div class="table-container">
        <table class="data-table">
          <thead>
            <tr>
              <th>Domain</th>
              <th>Document Root</th>
              <th>PHP Pool</th>
              <th>SSL</th>
              <th style="text-align: right;">Actions</th>
            </tr>
          </thead>
          <tbody id="sites-table-body">
            ${
              this.sites.length === 0
                ? `<tr><td colspan="5" style="text-align: center; color: var(--text-muted); padding: 32px;">No projects found in www/. Create or clone a Laravel project into www/ to begin.</td></tr>`
                : ''
            }
          </tbody>
        </table>
      </div>
    `;

    const tbody = container.querySelector('#sites-table-body')!;

    this.sites.forEach((site) => {
      const tr = document.createElement('tr');

      tr.innerHTML = `
        <td>
          <div class="site-domain">
            <span style="color: var(--accent-emerald); margin-right: 4px;">🔒</span>
            ${site.domain}
          </div>
          <div class="site-path">${site.name}</div>
        </td>
        <td>
          <div style="font-family: var(--font-mono); font-size: 11px; color: var(--text-secondary);">
            ${site.public_path.split('/').pop() || 'root'}
          </div>
        </td>
        <td>
          <select class="php-select site-php-select" data-site="${site.name}" style="background: var(--bg-elevated); padding: 4px 8px; border-radius: var(--radius-sm); border: 1px solid var(--border-color);">
            <option value="default" ${site.php_version === 'default' ? 'selected' : ''}>Inherit Global</option>
            <option value="8.2" ${site.php_version === '8.2' ? 'selected' : ''}>PHP 8.2</option>
            <option value="8.3" ${site.php_version === '8.3' ? 'selected' : ''}>PHP 8.3</option>
            <option value="8.4" ${site.php_version === '8.4' ? 'selected' : ''}>PHP 8.4</option>
            <option value="8.5" ${site.php_version === '8.5' ? 'selected' : ''}>PHP 8.5</option>
          </select>
        </td>
        <td>
          <span class="badge badge-running">Trusted CA</span>
        </td>
        <td style="text-align: right;">
          <button class="btn btn-secondary btn-sm btn-open-browser" data-domain="${site.domain}">Open Site</button>
          <button class="btn btn-secondary btn-sm btn-open-dir" data-path="${site.path}">Folder</button>
        </td>
      `;

      // Event handlers
      const selectPhp = tr.querySelector('.site-php-select');
      if (selectPhp) {
        selectPhp.addEventListener('change', async (e) => {
          const newVer = (e.target as HTMLSelectElement).value;
          try {
            await invokeCommand('set_site_php_version', { site: site.name, version: newVer });
            this.onAction();
          } catch (err) {
            alert('Failed to update site PHP version: ' + err);
          }
        });
      }

      const btnBrowser = tr.querySelector('.btn-open-browser');
      if (btnBrowser) {
        btnBrowser.addEventListener('click', async () => {
          try {
            await invokeCommand('open_site_in_browser', { domain: site.domain });
          } catch (err) {
            console.warn('Open browser error:', err);
          }
        });
      }

      const btnDir = tr.querySelector('.btn-open-dir');
      if (btnDir) {
        btnDir.addEventListener('click', async () => {
          try {
            await invokeCommand('open_site_folder', { path: site.path });
          } catch (err) {
            console.warn('Open folder error:', err);
          }
        });
      }

      tbody.appendChild(tr);
    });

    const btnRescan = container.querySelector('#btn-rescan-sites');
    if (btnRescan) {
      btnRescan.addEventListener('click', async () => {
        try {
          await invokeCommand('rescan_sites');
          this.onAction();
        } catch (err) {
          console.warn('Rescan error:', err);
        }
      });
    }

    return container;
  }
}
