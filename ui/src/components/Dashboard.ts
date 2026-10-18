import { ServiceStatus, invokeCommand } from '../lib/tauri';
import { iconPlay, iconStop, iconRotate, iconChevronDown, iconDownload } from '../lib/icons';

export class Dashboard {
  private services: ServiceStatus[] = [];
  private activePhp: string = '8.3';
  private onAction: () => void;

  constructor(onAction: () => void) {
    this.onAction = onAction;
  }

  public update(services: ServiceStatus[], activePhp?: string) {
    this.services = services;
    if (activePhp) {
      this.activePhp = activePhp;
    }
  }

  public render(): HTMLElement {
    const container = document.createElement('div');
    container.className = 'view-content';

    const grid = document.createElement('div');
    grid.className = 'services-grid';
    grid.id = 'service-cards-container';
    container.appendChild(grid);

    // 1. Unified PHP Service Card with inline version dropdown
    grid.appendChild(this.renderPhpCard());

    // 2. MySQL Service Card
    const mysql = this.services.find((s) => s.kind === 'mysql');
    if (mysql) {
      grid.appendChild(this.renderStandardCard(mysql, 'MySQL Database'));
    }

    // 3. Nginx Web Server Card
    const nginx = this.services.find((s) => s.kind === 'nginx');
    if (nginx) {
      grid.appendChild(this.renderStandardCard(nginx, 'Nginx Web Server'));
    }

    // 4. Redis Cache Card
    const redis = this.services.find((s) => s.kind === 'redis');
    if (redis) {
      grid.appendChild(this.renderStandardCard(redis, 'Redis Cache'));
    }

    // 5. Mail Catcher Card
    const mail = this.services.find((s) => s.kind === 'mailpit');
    if (mail) {
      grid.appendChild(this.renderStandardCard(mail, 'Mail Catcher'));
    }

    return container;
  }

  private renderPhpCard(): HTMLElement {
    const card = document.createElement('div');
    card.className = 'service-card';

    // Find active PHP service state (php82, php83, php84, php85)
    const phpKind = `php${this.activePhp.replace('.', '')}` as ServiceStatus['kind'];
    const activeService = this.services.find((s) => s.kind === phpKind) || {
      name: `PHP ${this.activePhp} FPM`,
      kind: phpKind,
      state: 'not_installed',
      port: 9000 + parseInt(this.activePhp.replace('.', ''), 10),
      pid: null,
      uptime_seconds: 0,
      memory_bytes: 0,
      cpu_percent: 0,
    };

    const isRunning = activeService.state === 'running';
    const isError = activeService.state === 'error';
    const isConflict = activeService.state === 'conflict';
    const isNotInstalled = activeService.state === 'not_installed';
    const isInstalling = activeService.state === 'installing';

    let badgeClass = 'badge-stopped';
    let badgeText = 'Stopped';

    if (isRunning) {
      badgeClass = 'badge-running';
      badgeText = 'Running';
    } else if (isInstalling) {
      badgeClass = 'badge-warning';
      badgeText = `Installing (${activeService.progress_percent || 15}%)`;
    } else if (isNotInstalled) {
      badgeClass = 'badge-stopped';
      badgeText = 'Not Installed';
    } else if (isError) {
      badgeClass = 'badge-error';
      badgeText = 'Error';
    } else if (isConflict) {
      badgeClass = 'badge-warning';
      badgeText = 'Port Conflict';
    }

    card.innerHTML = `
      <div class="card-top">
        <div class="card-title-group">
          <span class="service-title">PHP</span>
          <div class="custom-select-wrap">
            <select class="custom-select" id="card-php-select" aria-label="Select Active PHP Version">
              <option value="8.2" ${this.activePhp === '8.2' ? 'selected' : ''}>8.2</option>
              <option value="8.3" ${this.activePhp === '8.3' ? 'selected' : ''}>8.3</option>
              <option value="8.4" ${this.activePhp === '8.4' ? 'selected' : ''}>8.4</option>
              <option value="8.5" ${this.activePhp === '8.5' ? 'selected' : ''}>8.5</option>
            </select>
            <span class="custom-select-arrow">${iconChevronDown()}</span>
          </div>
          <span class="port-pill">:${activeService.port}</span>
        </div>
        <span class="badge ${badgeClass}">${badgeText}</span>
      </div>

      ${
        isInstalling
          ? `
            <div class="progress-bar-wrap">
              <div class="progress-bar-fill" style="width: ${activeService.progress_percent || 15}%"></div>
            </div>
          `
          : ''
      }

      ${
        isConflict && activeService.conflict_process
          ? `<div class="conflict-notice">Port occupied by: ${activeService.conflict_process}</div>`
          : ''
      }
      ${
        isError && activeService.error_message
          ? `<div class="error-notice">${activeService.error_message}</div>`
          : ''
      }

      ${
        isRunning
          ? `
            <div class="card-stats">
              <span>Up <strong>${this.formatUptime(activeService.uptime_seconds)}</strong></span>
              <span>RAM <strong>${(activeService.memory_bytes / (1024 * 1024)).toFixed(1)} MB</strong></span>
              <span>CPU <strong>${activeService.cpu_percent.toFixed(1)}%</strong></span>
              ${activeService.pid ? `<span>PID <strong>${activeService.pid}</strong></span>` : ''}
            </div>
          `
          : ''
      }

      <div class="card-actions">
        ${
          isInstalling
            ? `<button class="btn btn-secondary btn-sm" disabled style="opacity: 0.65;">Installing...</button>`
            : isNotInstalled
            ? `
              <button class="btn btn-primary btn-sm btn-action-install" data-kind="${activeService.kind}" title="Download and provision PHP ${this.activePhp}">
                ${iconDownload()} Install Runtime
              </button>
            `
            : isRunning
            ? `
              <button class="btn btn-secondary btn-sm btn-action-restart" data-kind="${activeService.kind}" title="Restart PHP ${this.activePhp}">
                ${iconRotate()} Restart
              </button>
              <button class="btn btn-danger btn-sm btn-action-stop" data-kind="${activeService.kind}" title="Stop PHP ${this.activePhp}">
                ${iconStop()} Stop
              </button>
            `
            : `
              <button class="btn btn-primary btn-sm btn-action-start" data-kind="${activeService.kind}" title="Start PHP ${this.activePhp}">
                ${iconPlay()} Start
              </button>
            `
        }
      </div>
    `;

    // Dropdown change listener
    const select = card.querySelector<HTMLSelectElement>('#card-php-select');
    if (select) {
      select.addEventListener('change', async (e) => {
        const newVer = (e.target as HTMLSelectElement).value;
        try {
          await invokeCommand('set_active_php', { version: newVer });
          this.activePhp = newVer;
          this.onAction();
        } catch (err) {
          alert('Failed to switch PHP version: ' + err);
        }
      });
    }

    this.bindActionButtons(card, activeService.name);
    return card;
  }

  private renderStandardCard(service: ServiceStatus, displayTitle: string): HTMLElement {
    const card = document.createElement('div');
    card.className = 'service-card';

    const isRunning = service.state === 'running';
    const isError = service.state === 'error';
    const isConflict = service.state === 'conflict';
    const isNotInstalled = service.state === 'not_installed';
    const isInstalling = service.state === 'installing';

    let badgeClass = 'badge-stopped';
    let badgeText = 'Stopped';

    if (isRunning) {
      badgeClass = 'badge-running';
      badgeText = 'Running';
    } else if (isInstalling) {
      badgeClass = 'badge-warning';
      badgeText = `Installing (${service.progress_percent || 15}%)`;
    } else if (isNotInstalled) {
      badgeClass = 'badge-stopped';
      badgeText = 'Not Installed';
    } else if (isError) {
      badgeClass = 'badge-error';
      badgeText = 'Error';
    } else if (isConflict) {
      badgeClass = 'badge-warning';
      badgeText = 'Port Conflict';
    }

    card.innerHTML = `
      <div class="card-top">
        <div class="card-title-group">
          <span class="service-title">${displayTitle}</span>
          <span class="port-pill">:${service.port}</span>
        </div>
        <span class="badge ${badgeClass}">${badgeText}</span>
      </div>

      ${
        isInstalling
          ? `
            <div class="progress-bar-wrap">
              <div class="progress-bar-fill" style="width: ${service.progress_percent || 15}%"></div>
            </div>
          `
          : ''
      }

      ${
        isConflict && service.conflict_process
          ? `<div class="conflict-notice">Port occupied by: ${service.conflict_process}</div>`
          : ''
      }
      ${
        isError && service.error_message
          ? `<div class="error-notice">${service.error_message}</div>`
          : ''
      }

      ${
        isRunning
          ? `
            <div class="card-stats">
              <span>Up <strong>${this.formatUptime(service.uptime_seconds)}</strong></span>
              <span>RAM <strong>${(service.memory_bytes / (1024 * 1024)).toFixed(1)} MB</strong></span>
              <span>CPU <strong>${service.cpu_percent.toFixed(1)}%</strong></span>
              ${service.pid ? `<span>PID <strong>${service.pid}</strong></span>` : ''}
            </div>
          `
          : ''
      }

      <div class="card-actions">
        ${
          isInstalling
            ? `<button class="btn btn-secondary btn-sm" disabled style="opacity: 0.65;">Installing...</button>`
            : isNotInstalled
            ? `
              <button class="btn btn-primary btn-sm btn-action-install" data-kind="${service.kind}" title="Download and provision ${displayTitle}">
                ${iconDownload()} Install Runtime
              </button>
            `
            : isRunning
            ? `
              <button class="btn btn-secondary btn-sm btn-action-restart" data-kind="${service.kind}">
                ${iconRotate()} Restart
              </button>
              <button class="btn btn-danger btn-sm btn-action-stop" data-kind="${service.kind}">
                ${iconStop()} Stop
              </button>
            `
            : `
              <button class="btn btn-primary btn-sm btn-action-start" data-kind="${service.kind}">
                ${iconPlay()} Start
              </button>
            `
        }
      </div>
    `;

    this.bindActionButtons(card, displayTitle);
    return card;
  }

  private bindActionButtons(card: HTMLElement, serviceName: string) {
    const btnInstall = card.querySelector<HTMLButtonElement>('.btn-action-install');
    if (btnInstall) {
      btnInstall.addEventListener('click', async () => {
        btnInstall.setAttribute('disabled', 'true');
        btnInstall.innerText = 'Installing...';
        const kind = btnInstall.getAttribute('data-kind');
        try {
          await invokeCommand('install_service', { service: kind });
          this.onAction();
        } catch (err) {
          alert(`Failed to install ${serviceName}: ` + err);
        } finally {
          btnInstall.removeAttribute('disabled');
        }
      });
    }

    const btnStart = card.querySelector<HTMLButtonElement>('.btn-action-start');
    if (btnStart) {
      btnStart.addEventListener('click', async () => {
        btnStart.setAttribute('disabled', 'true');
        const kind = btnStart.getAttribute('data-kind');
        try {
          await invokeCommand('start_service', { service: kind });
          this.onAction();
        } catch (err) {
          alert(`Failed to start ${serviceName}: ` + err);
        } finally {
          btnStart.removeAttribute('disabled');
        }
      });
    }

    const btnStop = card.querySelector<HTMLButtonElement>('.btn-action-stop');
    if (btnStop) {
      btnStop.addEventListener('click', async () => {
        btnStop.setAttribute('disabled', 'true');
        const kind = btnStop.getAttribute('data-kind');
        try {
          await invokeCommand('stop_service', { service: kind });
          this.onAction();
        } catch (err) {
          alert(`Failed to stop ${serviceName}: ` + err);
        } finally {
          btnStop.removeAttribute('disabled');
        }
      });
    }

    const btnRestart = card.querySelector<HTMLButtonElement>('.btn-action-restart');
    if (btnRestart) {
      btnRestart.addEventListener('click', async () => {
        btnRestart.setAttribute('disabled', 'true');
        const kind = btnRestart.getAttribute('data-kind');
        try {
          await invokeCommand('restart_service', { service: kind });
          this.onAction();
        } catch (err) {
          alert(`Failed to restart ${serviceName}: ` + err);
        } finally {
          btnRestart.removeAttribute('disabled');
        }
      });
    }
  }

  private formatUptime(seconds: number): string {
    if (seconds < 60) return `${seconds}s`;
    const mins = Math.floor(seconds / 60);
    if (mins < 60) return `${mins}m ${seconds % 60}s`;
    const hrs = Math.floor(mins / 60);
    return `${hrs}h ${mins % 60}m`;
  }
}
