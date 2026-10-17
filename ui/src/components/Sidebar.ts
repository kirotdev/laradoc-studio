export type TabId = 'dashboard' | 'sites' | 'logs' | 'mail' | 'settings';

export class Sidebar {
  private activeTab: TabId = 'dashboard';
  private onTabChange: (tab: TabId) => void;

  constructor(onTabChange: (tab: TabId) => void) {
    this.onTabChange = onTabChange;
  }

  public setActiveTab(tab: TabId) {
    this.activeTab = tab;
    this.render();
  }

  public render(): HTMLElement {
    const aside = document.createElement('aside');
    aside.className = 'sidebar';

    aside.innerHTML = `
      <div class="brand">
        <div class="brand-icon">L</div>
        <div class="brand-text">
          <h1>Laradoc Studio</h1>
          <span class="brand-badge">v0.1.0-alpha</span>
        </div>
      </div>

      <nav class="nav-links">
        <a class="nav-item ${this.activeTab === 'dashboard' ? 'active' : ''}" data-tab="dashboard">
          <span class="nav-icon">⚡</span>
          <span>Dashboard</span>
        </a>
        <a class="nav-item ${this.activeTab === 'sites' ? 'active' : ''}" data-tab="sites">
          <span class="nav-icon">🌐</span>
          <span>Sites & VHosts</span>
        </a>
        <a class="nav-item ${this.activeTab === 'logs' ? 'active' : ''}" data-tab="logs">
          <span class="nav-icon">📋</span>
          <span>Live Logs</span>
        </a>
        <a class="nav-item ${this.activeTab === 'mail' ? 'active' : ''}" data-tab="mail">
          <span class="nav-icon">📬</span>
          <span>Mail Catcher</span>
          <span class="nav-badge" id="mail-count-badge">0</span>
        </a>
        <a class="nav-item ${this.activeTab === 'settings' ? 'active' : ''}" data-tab="settings">
          <span class="nav-icon">⚙️</span>
          <span>Settings & PKI</span>
        </a>
      </nav>

      <div class="sidebar-footer">
        <div class="cold-start-indicator">
          <span class="dot-indicator stopped" id="global-status-dot"></span>
          <span id="global-status-text">Cold Start (Stopped)</span>
        </div>
        <span>Port 1025 Mail / Port 6379 Redis</span>
      </div>
    `;

    aside.querySelectorAll<HTMLElement>('.nav-item').forEach((item) => {
      item.addEventListener('click', () => {
        const tab = item.getAttribute('data-tab') as TabId;
        if (tab) {
          this.setActiveTab(tab);
          this.onTabChange(tab);
        }
      });
    });

    return aside;
  }
}
