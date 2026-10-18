import { invokeCommand } from '../lib/tauri';
import { iconFolder, iconPlay, iconStop } from '../lib/icons';

export class Header {
  private title: string = 'Services';
  private isRunning: boolean = false;
  private onStateChange: () => void;

  constructor(onStateChange: () => void) {
    this.onStateChange = onStateChange;
  }

  public update(title: string, isRunning: boolean) {
    this.title = title;
    this.isRunning = isRunning;
  }

  public render(): HTMLElement {
    const header = document.createElement('header');
    header.className = 'top-header';

    header.innerHTML = `
      <div class="header-left">
        <h2 class="header-title">${this.title}</h2>
      </div>

      <div class="header-actions">
        <!-- Open www/ Folder -->
        <button class="btn btn-secondary btn-sm" id="btn-open-www" title="Open Projects Directory">
          ${iconFolder()} Open Folder
        </button>

        <!-- Master Start All / Stop All Toggle -->
        <button class="btn ${this.isRunning ? 'btn-danger' : 'btn-primary'} btn-sm" id="btn-master-toggle">
          ${this.isRunning ? `${iconStop()} Stop All` : `${iconPlay()} Start All`}
        </button>
      </div>
    `;

    // Open www/ folder
    const btnOpenWww = header.querySelector<HTMLButtonElement>('#btn-open-www');
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
    const masterBtn = header.querySelector<HTMLButtonElement>('#btn-master-toggle');
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
