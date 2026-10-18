import {
  iconServer,
  iconGlobe,
  iconTerminal,
  iconMail,
  iconSettings,
} from '../lib/icons';

export type TabId = 'dashboard' | 'sites' | 'logs' | 'mail' | 'settings';

export class Sidebar {
  private activeTab: TabId = 'dashboard';
  private anyRunning: boolean = false;
  private mailCount: number = 0;
  private onTabChange: (tab: TabId) => void;

  constructor(onTabChange: (tab: TabId) => void) {
    this.onTabChange = onTabChange;
  }

  public setActiveTab(tab: TabId) {
    this.activeTab = tab;
  }

  public updateStatus(anyRunning: boolean, mailCount: number) {
    this.anyRunning = anyRunning;
    this.mailCount = mailCount;
  }

  public render(): HTMLElement {
    const aside = document.createElement('aside');
    aside.className = 'sidebar';

    aside.innerHTML = `
      <div class="brand">
        <div class="brand-icon">L</div>
        <div class="brand-text">
          <span class="brand-title">Laradoc</span>
          <span class="brand-sub">Studio v0.1.0</span>
        </div>
      </div>

      <nav class="nav-links">
        <a class="nav-item ${this.activeTab === 'dashboard' ? 'active' : ''}" data-tab="dashboard">
          <span class="nav-icon">${iconServer()}</span>
          <span>Services</span>
        </a>
        <a class="nav-item ${this.activeTab === 'sites' ? 'active' : ''}" data-tab="sites">
          <span class="nav-icon">${iconGlobe()}</span>
          <span>Sites</span>
        </a>
        <a class="nav-item ${this.activeTab === 'logs' ? 'active' : ''}" data-tab="logs">
          <span class="nav-icon">${iconTerminal()}</span>
          <span>Logs</span>
        </a>
        <a class="nav-item ${this.activeTab === 'mail' ? 'active' : ''}" data-tab="mail">
          <span class="nav-icon">${iconMail()}</span>
          <span>Mail</span>
          <span class="nav-badge ${this.mailCount > 0 ? 'active' : ''}" id="mail-count-badge">${this.mailCount}</span>
        </a>
        <a class="nav-item ${this.activeTab === 'settings' ? 'active' : ''}" data-tab="settings">
          <span class="nav-icon">${iconSettings()}</span>
          <span>Settings</span>
        </a>
      </nav>

      <div class="sidebar-footer">
        <span class="status-dot ${this.anyRunning ? 'running' : ''}" id="global-status-dot"></span>
        <span id="global-status-text">${this.anyRunning ? 'Running' : 'Stopped'}</span>
      </div>
    `;

    aside.querySelectorAll<HTMLElement>('.nav-item').forEach((item) => {
      item.addEventListener('click', () => {
        const tab = item.getAttribute('data-tab') as TabId;
        if (tab && tab !== this.activeTab) {
          this.setActiveTab(tab);
          this.onTabChange(tab);
        }
      });
    });

    return aside;
  }
}
