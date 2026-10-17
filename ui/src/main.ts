import { Dashboard } from './components/Dashboard';
import { Header } from './components/Header';
import { LogViewer } from './components/LogViewer';
import { MailViewer } from './components/MailViewer';
import { SettingsView } from './components/SettingsView';
import { Sidebar, TabId } from './components/Sidebar';
import { SitesView } from './components/SitesView';
import {
  AppSettings,
  CapturedEmail,
  invokeCommand,
  listenEvent,
  LogEntry,
  ServiceStatus,
  SiteInfo,
} from './lib/tauri';

class App {
  private activeTab: TabId = 'dashboard';
  private services: ServiceStatus[] = [];
  private sites: SiteInfo[] = [];
  private emails: CapturedEmail[] = [];
  private logs: LogEntry[] = [];
  private settings: AppSettings = {
    active_php: '8.3',
    www_directory: 'www',
    root_ca_trusted: true,
    hosts_synced: true,
    auto_start_disabled: true,
  };

  private sidebar: Sidebar;
  private header: Header;
  private dashboard: Dashboard;
  private sitesView: SitesView;
  private logViewer: LogViewer;
  private mailViewer: MailViewer;
  private settingsView: SettingsView;

  private appRoot: HTMLElement;
  private contentSlot!: HTMLElement;
  private pollInterval: number | null = null;

  constructor() {
    this.appRoot = document.getElementById('app')!;

    this.sidebar = new Sidebar((tab) => this.switchTab(tab));
    this.header = new Header(() => this.refreshData());
    this.dashboard = new Dashboard(() => this.refreshData());
    this.sitesView = new SitesView(() => this.refreshData());
    this.logViewer = new LogViewer();
    this.mailViewer = new MailViewer(() => this.refreshData());
    this.settingsView = new SettingsView(() => this.refreshData());

    this.initLayout();
    this.initData();
    this.initEvents();
  }

  private initLayout() {
    this.appRoot.innerHTML = '';

    // Sidebar
    this.appRoot.appendChild(this.sidebar.render());

    // Main Viewport
    const mainViewport = document.createElement('main');
    mainViewport.className = 'main-viewport';

    // Header
    mainViewport.appendChild(this.header.render());

    // Content Slot
    this.contentSlot = document.createElement('div');
    this.contentSlot.style.flex = '1';
    this.contentSlot.style.display = 'flex';
    this.contentSlot.style.flexDirection = 'column';
    this.contentSlot.style.overflow = 'hidden';
    mainViewport.appendChild(this.contentSlot);

    this.appRoot.appendChild(mainViewport);

    this.renderCurrentView();
  }

  private switchTab(tab: TabId) {
    this.activeTab = tab;
    this.updateHeader();
    this.renderCurrentView();
  }

  private updateHeader() {
    const titles: Record<TabId, string> = {
      dashboard: 'Service Dashboard',
      sites: 'Sites & Virtual Hosts',
      logs: 'Real-Time Log Stream',
      mail: 'Local Mail Catcher',
      settings: 'Settings & Security',
    };
    const anyRunning = this.services.some((s) => s.state === 'running');
    this.header.update(titles[this.activeTab], anyRunning, this.settings.active_php);

    // Re-render header
    const mainViewport = this.appRoot.querySelector('.main-viewport');
    if (mainViewport) {
      const oldHeader = mainViewport.querySelector('.top-header');
      if (oldHeader) {
        mainViewport.replaceChild(this.header.render(), oldHeader);
      }
    }
  }

  private renderCurrentView() {
    this.contentSlot.innerHTML = '';

    switch (this.activeTab) {
      case 'dashboard':
        this.dashboard.update(this.services);
        this.contentSlot.appendChild(this.dashboard.render());
        break;
      case 'sites':
        this.sitesView.update(this.sites);
        this.contentSlot.appendChild(this.sitesView.render());
        break;
      case 'logs':
        this.logViewer.update(this.logs);
        this.contentSlot.appendChild(this.logViewer.render());
        break;
      case 'mail':
        this.mailViewer.update(this.emails);
        this.contentSlot.appendChild(this.mailViewer.render());
        break;
      case 'settings':
        this.settingsView.update(this.settings);
        this.contentSlot.appendChild(this.settingsView.render());
        break;
    }
  }

  private async initData() {
    await this.refreshData();

    // Regular polling every 2.5s for service statuses
    this.pollInterval = window.setInterval(() => {
      this.pollServices();
    }, 2500);
  }

  private async refreshData() {
    try {
      const [services, sites, emails, logs, settings] = await Promise.all([
        invokeCommand<ServiceStatus[]>('get_services_status'),
        invokeCommand<SiteInfo[]>('list_sites'),
        invokeCommand<CapturedEmail[]>('get_emails'),
        invokeCommand<LogEntry[]>('get_recent_logs'),
        invokeCommand<AppSettings>('get_settings'),
      ]);

      this.services = services;
      this.sites = sites;
      this.emails = emails;
      this.logs = logs;
      this.settings = settings;

      // Update badge counts and cold start indicator
      const mailBadge = document.getElementById('mail-count-badge');
      if (mailBadge) mailBadge.innerText = emails.length.toString();

      const globalDot = document.getElementById('global-status-dot');
      const globalText = document.getElementById('global-status-text');
      const anyRunning = this.services.some((s) => s.state === 'running');

      if (globalDot && globalText) {
        if (anyRunning) {
          globalDot.className = 'dot-indicator running';
          globalText.innerText = 'Active (Services Running)';
        } else {
          globalDot.className = 'dot-indicator stopped';
          globalText.innerText = 'Cold Start (Stopped)';
        }
      }

      this.updateHeader();
      this.renderCurrentView();
    } catch (err) {
      console.warn('Failed to refresh data:', err);
    }
  }

  private async pollServices() {
    try {
      const services = await invokeCommand<ServiceStatus[]>('get_services_status');
      this.services = services;
      if (this.activeTab === 'dashboard') {
        this.dashboard.update(this.services);
        this.contentSlot.innerHTML = '';
        this.contentSlot.appendChild(this.dashboard.render());
      }
    } catch (err) {
      console.warn('Poll error:', err);
    }
  }

  private initEvents() {
    // Real-time log streamer event
    listenEvent<LogEntry>('log:entry', (entry) => {
      this.logViewer.appendLog(entry);
    });

    // Real-time mail catcher event
    listenEvent<CapturedEmail>('mail:received', (email) => {
      this.mailViewer.appendEmail(email);
      const mailBadge = document.getElementById('mail-count-badge');
      if (mailBadge) {
        const count = parseInt(mailBadge.innerText || '0', 10) + 1;
        mailBadge.innerText = count.toString();
      }
    });

    // Service state changed event
    listenEvent<void>('service:state_changed', () => {
      this.refreshData();
    });
  }
}

// Initialize on DOM load
window.addEventListener('DOMContentLoaded', () => {
  new App();
});
