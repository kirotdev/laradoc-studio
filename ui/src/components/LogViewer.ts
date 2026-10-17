import { LogEntry, invokeCommand } from '../lib/tauri';

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

    container.innerHTML = `
      <div class="log-terminal">
        <div class="log-toolbar">
          <div style="display: flex; align-items: center; gap: 12px;">
            <span style="font-weight: 600; font-size: 11px; text-transform: uppercase; color: var(--text-muted);">Log Stream:</span>
            <select class="php-select" id="log-filter-service" style="background: var(--bg-elevated); padding: 4px 8px; border-radius: var(--radius-sm); border: 1px solid var(--border-color);">
              <option value="all">All Services</option>
              <option value="Nginx">Nginx</option>
              <option value="PHP-Manager">PHP Pools</option>
              <option value="MySQL">MySQL</option>
              <option value="Redis">Redis</option>
              <option value="Mailpit">Mail Catcher</option>
              <option value="System">System</option>
            </select>
          </div>

          <div style="display: flex; align-items: center; gap: 12px;">
            <label style="font-size: 11px; color: var(--text-secondary); display: flex; align-items: center; gap: 4px; cursor: pointer;">
              <input type="checkbox" id="log-autoscroll" ${this.autoScroll ? 'checked' : ''} />
              Auto-scroll
            </label>
            <button class="btn btn-secondary btn-sm" id="btn-clear-logs">Clear Output</button>
          </div>
        </div>

        <div class="log-output" id="terminal-output-body"></div>
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
      target.innerHTML = `<div style="color: var(--text-muted); text-align: center; padding: 40px;">No logs recorded yet.</div>`;
      return;
    }

    filtered.forEach((log) => {
      const line = document.createElement('div');
      line.className = 'log-line';

      let levelClass = 'log-level-info';
      if (log.level === 'error') levelClass = 'log-level-error';
      else if (log.level === 'warn') levelClass = 'log-level-warn';

      line.innerHTML = `
        <span class="log-timestamp">[${log.timestamp}]</span>
        <span class="log-service">[${log.service}]</span>
        <span class="${levelClass}">${this.escapeHtml(log.message)}</span>
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
