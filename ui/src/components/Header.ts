import { invokeCommand } from '../lib/tauri';

export class Header {
  private title: string = 'Dashboard';
  private isRunning: boolean = false;
  private activePhp: string = '8.3';
  private onStateChange: () => void;

  constructor(onStateChange: () => void) {
    this.onStateChange = onStateChange;
  }

  public update(title: string, isRunning: boolean, activePhp: string) {
    this.title = title;
    this.isRunning = isRunning;
    this.activePhp = activePhp;
  }

  public render(): HTMLElement {
    const header = document.createElement('header');
    header.className = 'top-header';

    header.innerHTML = `
      <div class="header-left">
        <h2 class="page-title">${this.title}</h2>
      </div>

      <div class="header-right">
        <!-- Global PHP Selector -->
        <div class="php-selector-container" title="Global CLI PHP & Composer Version">
          <span class="php-label">Active PHP:</span>
          <select class="php-select" id="global-php-select">
            <option value="8.2" ${this.activePhp === '8.2' ? 'selected' : ''}>PHP 8.2</option>
            <option value="8.3" ${this.activePhp === '8.3' ? 'selected' : ''}>PHP 8.3 (Default)</option>
            <option value="8.4" ${this.activePhp === '8.4' ? 'selected' : ''}>PHP 8.4</option>
            <option value="8.5" ${this.activePhp === '8.5' ? 'selected' : ''}>PHP 8.5 (Preview)</option>
          </select>
        </div>

        <!-- Open www/ Folder -->
        <button class="btn btn-secondary btn-sm" id="btn-open-www" title="Open www/ Directory in Finder/Explorer">
          📁 Open www/
        </button>

        <!-- Master Start/Stop All Button -->
        <button class="btn ${this.isRunning ? 'btn-danger' : 'btn-primary'}" id="btn-master-toggle">
          ${this.isRunning ? '⏹ Stop All' : '▶ Start All'}
        </button>
      </div>
    `;

    // PHP Selector Change Event
    const phpSelect = header.querySelector<HTMLSelectElement>('#global-php-select');
    if (phpSelect) {
      phpSelect.addEventListener('change', async (e) => {
        const val = (e.target as HTMLSelectElement).value;
        try {
          await invokeCommand('set_active_php', { version: val });
          this.activePhp = val;
          this.onStateChange();
        } catch (err) {
          alert('Failed to switch active PHP version: ' + err);
        }
      });
    }

    // Open www/ folder
    const btnOpenWww = header.querySelector('#btn-open-www');
    if (btnOpenWww) {
      btnOpenWww.addEventListener('click', async () => {
        try {
          await invokeCommand('open_site_folder', { path: '' });
        } catch (err) {
          console.warn('Open www error:', err);
        }
      });
    }

    // Master Toggle Event
    const masterBtn = header.querySelector('#btn-master-toggle');
    if (masterBtn) {
      masterBtn.addEventListener('click', async () => {
        masterBtn.setAttribute('disabled', 'true');
        try {
          if (this.isRunning) {
            await invokeCommand('stop_all');
          } else {
            await invokeCommand('start_all');
          }
          this.onStateChange();
        } catch (err) {
          alert('Action failed: ' + err);
        } finally {
          masterBtn.removeAttribute('disabled');
        }
      });
    }

    return header;
  }
}
