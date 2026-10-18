import { LogEntry, invokeCommand } from '../lib/tauri';
import { iconTrash, iconChevronDown } from '../lib/icons';

export class LogViewer {
  private logs: LogEntry[] = [];
  private filterService: string = 'all';
  private autoScroll: boolean = true;

  public update(logs: LogEntry[]) {
    this.logs = logs;
  }

  public appendLog(entry: LogEntry) {
    this.logs.push(entry);
    if (this.logs.length > 500) {
      this.logs.shift();
    }
  }

  public render(): HTMLElement {
    const container = document.createElement('div');
    container.className = 'view-content';
    container.style.display = 'flex';
    container.style.flexDirection = 'column';
    container.style.height = '100%';
    container.style.overflow = 'hidden';

    container.innerHTML = `
      <div class="log-card">
        <div class="log-bar">
          <div style="display: flex; align-items: center; gap: 10px;">
            <div class="custom-select-wrap">
              <select class="custom-select" id="log-filter-service" style="background: #1e293b; color: #f8fafc; border-color: #334155;">
                <option value="all">All Services</option>
                <option value="Nginx">Nginx</option>
                <option value="PHP">PHP Pools</option>
                <option value="MySQL">MySQL</option>
                <option value="Redis">Redis</option>
                <option value="Mailpit">Mail Catcher</option>
                <option value="System">System</option>
              </select>
              <span class="custom-select-arrow" style="color: #94a3b8;">${iconChevronDown()}</span>
            </div>
          </div>

          <div style="display: flex; align-items: center; gap: 12px;">
            <label style="font-size: 11.5px; color: #94a3b8; display: flex; align-items: center; gap: 5px; cursor: pointer; user-select: none;">
              <input type="checkbox" id="log-autoscroll" ${this.autoScroll ? 'checked' : ''} style="accent-color: #0f172a;" />
              Auto-scroll
            </label>
            <button class="btn btn-secondary btn-sm" id="btn-clear-logs" style="background: #1e293b; color: #f8fafc; border-color: #334155;">
              ${iconTrash()} Clear
            </button>
          </div>
        </div>

        <div class="log-scroll-area" id="terminal-output-body"></div>
      </div>
    `;

    const outputBody = container.querySelector<HTMLElement>('#terminal-output-body')!;
    this.renderLogs(outputBody);

    const filterSelect = container.querySelector<HTMLSelectElement>('#log-filter-service');
    if (filterSelect) {
      filterSelect.addEventListener('change', (e) => {
        this.filterService = (e.target as HTMLSelectElement).value;
        this.renderLogs(outputBody);
      });
    }

    const autoScrollCheck = container.querySelector<HTMLInputElement>('#log-autoscroll');
    if (autoScrollCheck) {
      autoScrollCheck.addEventListener('change', (e) => {
        this.autoScroll = (e.target as HTMLInputElement).checked;
      });
    }

    const btnClear = container.querySelector('#btn-clear-logs');
    if (btnClear) {
      btnClear.addEventListener('click', async () => {
        try {
          await invokeCommand('clear_logs');
          this.logs = [];
          this.renderLogs(outputBody);
        } catch (err) {
          console.warn('Clear logs error:', err);
        }
      });
    }

    return container;
  }

  private renderLogs(target: HTMLElement) {
    target.innerHTML = '';
    const filtered = this.filterService === 'all'
      ? this.logs
      : this.logs.filter((l) => l.service.toLowerCase().includes(this.filterService.toLowerCase()));

    if (filtered.length === 0) {
      target.innerHTML = `<div style="color: #64748b; text-align: center; padding: 48px;">No logs recorded.</div>`;
      return;
    }

    filtered.forEach((log) => {
      const line = document.createElement('div');
      line.className = 'log-line';

      let tagColor = '#94a3b8';
      if (log.level === 'error') tagColor = '#f87171';
      else if (log.level === 'warn') tagColor = '#fbbf24';
      else if (log.level === 'info') tagColor = '#38bdf8';

      line.innerHTML = `
        <span class="log-time">${log.timestamp}</span>
        <span class="log-tag" style="color: ${tagColor};">[${log.service}]</span>
        <span class="log-msg">${this.escapeHtml(log.message)}</span>
      `;
      target.appendChild(line);
    });

    if (this.autoScroll) {
      target.scrollTop = target.scrollHeight;
    }
  }

  private escapeHtml(str: string): string {
    return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }
}
