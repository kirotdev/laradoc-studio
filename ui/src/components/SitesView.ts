import { SiteInfo, invokeCommand } from '../lib/tauri';
import {
  iconLock,
  iconRotate,
  iconExternalLink,
  iconFolder,
  iconChevronDown,
} from '../lib/icons';

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
        <span style="font-size: 13px; font-weight: 600; color: var(--text-secondary);">
          ${this.sites.length} ${this.sites.length === 1 ? 'Site' : 'Sites'} Detected
        </span>
        <button class="btn btn-secondary btn-sm" id="btn-rescan-sites">
          ${iconRotate()} Rescan
        </button>
      </div>

      <div class="table-wrapper">
        <table class="clean-table">
          <thead>
            <tr>
              <th>Domain</th>
              <th>Document Root</th>
              <th>PHP Version</th>
              <th>SSL</th>
              <th style="text-align: right;">Actions</th>
            </tr>
          </thead>
          <tbody id="sites-table-body">
            ${
              this.sites.length === 0
                ? `<tr><td colspan="5" style="text-align: center; color: var(--text-muted); padding: 48px;">No virtual hosts found in www/. Add a folder to get started.</td></tr>`
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
          <a class="domain-link btn-open-browser" data-domain="${site.domain}">
            <span style="color: var(--status-running); display: flex;">${iconLock()}</span>
            <span>${site.domain}</span>
          </a>
        </td>
        <td>
          <span style="font-family: var(--font-mono); font-size: 11.5px; color: var(--text-secondary);">
            ${site.public_path.split('/').pop() || 'public'}
          </span>
        </td>
        <td>
          <div class="custom-select-wrap">
            <select class="custom-select site-php-select" data-site="${site.name}">
              <option value="default" ${site.php_version === 'default' ? 'selected' : ''}>Inherit</option>
              <option value="8.2" ${site.php_version === '8.2' ? 'selected' : ''}>8.2</option>
              <option value="8.3" ${site.php_version === '8.3' ? 'selected' : ''}>8.3</option>
              <option value="8.4" ${site.php_version === '8.4' ? 'selected' : ''}>8.4</option>
              <option value="8.5" ${site.php_version === '8.5' ? 'selected' : ''}>8.5</option>
            </select>
            <span class="custom-select-arrow">${iconChevronDown()}</span>
          </div>
        </td>
        <td>
          <span class="badge badge-running">Trusted</span>
        </td>
        <td style="text-align: right;">
          <div style="display: inline-flex; gap: 6px;">
            <button class="btn btn-secondary btn-sm btn-open-browser" data-domain="${site.domain}" title="Open in browser">
              ${iconExternalLink()} Open
            </button>
            <button class="btn btn-secondary btn-sm btn-open-dir" data-path="${site.path}" title="Open directory in Finder">
              ${iconFolder()} Folder
            </button>
          </div>
        </td>
      `;

      // Event handlers
      const selectPhp = tr.querySelector<HTMLSelectElement>('.site-php-select');
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

      const openBrowserBtns = tr.querySelectorAll('.btn-open-browser');
      openBrowserBtns.forEach((btn) => {
        btn.addEventListener('click', async () => {
          try {
            await invokeCommand('open_site_in_browser', { domain: site.domain });
          } catch (err) {
            console.warn('Open browser error:', err);
          }
        });
      });

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
