import { ServiceStatus, invokeCommand } from '../lib/tauri';

export class Dashboard {
  private services: ServiceStatus[] = [];
  private onAction: () => void;

  constructor(onAction: () => void) {
    this.onAction = onAction;
  }

  public update(services: ServiceStatus[]) {
    this.services = services;
  }

  public render(): HTMLElement {
    const container = document.createElement('div');
    container.className = 'view-content';

    const runningCount = this.services.filter((s) => s.state === 'running').length;

    container.innerHTML = `
      <div class="banner-notice">
        <div class="banner-content">
          <h4>🛡️ Strict Cold-Start Policy Active</h4>
          <p>Zero OS background daemons or LaunchAgents. Services spawn cleanly inside isolated process groups and run only when commanded.</p>
        </div>
        <div style="font-size: 12px; font-weight: 600; color: var(--text-secondary);">
          ${runningCount} of ${this.services.length} Services Running
        </div>
      </div>

      <div class="cards-grid" id="service-cards-container"></div>
    `;

    const grid = container.querySelector('#service-cards-container')!;

    this.services.forEach((service) => {
      const card = document.createElement('div');
      card.className = 'service-card';

      const isRunning = service.state === 'running';
      const isError = service.state === 'error';
      const isConflict = service.state === 'conflict';

      let badgeClass = 'badge-stopped';
      let badgeLabel = '⚪ Stopped';

      if (isRunning) {
        badgeClass = 'badge-running';
        badgeLabel = '🟢 Running';
      } else if (isError) {
        badgeClass = 'badge-error';
        badgeLabel = '🔴 Error';
      } else if (isConflict) {
        badgeClass = 'badge-warning';
        badgeLabel = '🟠 Port Conflict';
      }

      const memMb = (service.memory_bytes / (1024 * 1024)).toFixed(1);
      const uptimeStr = isRunning ? this.formatUptime(service.uptime_seconds) : '—';

      card.innerHTML = `
        <div class="card-header">
          <div class="service-info">
            <div>
              <div class="service-name">${service.name}</div>
              <span class="service-port">:${service.port}</span>
            </div>
          </div>
          <span class="badge ${badgeClass}">${badgeLabel}</span>
        </div>

        ${
          isConflict && service.conflict_process
            ? `<div style="font-size: 11px; color: var(--accent-amber); background: var(--accent-amber-subtle); padding: 6px 10px; border-radius: var(--radius-sm);">
                Port occupied by: ${service.conflict_process}
               </div>`
            : ''
        }

        <div class="card-metrics">
          <div class="metric-item">
            <span>Uptime</span>
            <strong>${uptimeStr}</strong>
          </div>
          <div class="metric-item">
            <span>RAM</span>
            <strong>${isRunning ? memMb + ' MB' : '0 MB'}</strong>
          </div>
          <div class="metric-item">
            <span>CPU</span>
            <strong>${isRunning ? service.cpu_percent.toFixed(1) + '%' : '0.0%'}</strong>
          </div>
          <div class="metric-item">
            <span>PID</span>
            <strong>${service.pid ? service.pid : '—'}</strong>
          </div>
        </div>

        <div class="card-footer">
          ${
            isRunning
              ? `
                <button class="btn btn-secondary btn-sm btn-restart" data-service="${service.kind}">Restart</button>
                <button class="btn btn-danger btn-sm btn-stop" data-service="${service.kind}">Stop</button>
              `
              : `
                <button class="btn btn-primary btn-sm btn-start" data-service="${service.kind}">Start</button>
              `
          }
        </div>
      `;

      // Event listeners
      const btnStart = card.querySelector('.btn-start');
      if (btnStart) {
        btnStart.addEventListener('click', async () => {
          try {
            await invokeCommand('start_service', { service: service.kind });
            this.onAction();
          } catch (err) {
            alert(`Failed to start ${service.name}: ` + err);
          }
        });
      }

      const btnStop = card.querySelector('.btn-stop');
      if (btnStop) {
        btnStop.addEventListener('click', async () => {
          try {
            await invokeCommand('stop_service', { service: service.kind });
            this.onAction();
          } catch (err) {
            alert(`Failed to stop ${service.name}: ` + err);
          }
        });
      }

      const btnRestart = card.querySelector('.btn-restart');
      if (btnRestart) {
        btnRestart.addEventListener('click', async () => {
          try {
            await invokeCommand('restart_service', { service: service.kind });
            this.onAction();
          } catch (err) {
            alert(`Failed to restart ${service.name}: ` + err);
          }
        });
      }

      grid.appendChild(card);
    });

    return container;
  }

  private formatUptime(seconds: number): string {
    if (seconds < 60) return `${seconds}s`;
    const mins = Math.floor(seconds / 60);
    if (mins < 60) return `${mins}m ${seconds % 60}s`;
    const hrs = Math.floor(mins / 60);
    return `${hrs}h ${mins % 60}m`;
  }
}
