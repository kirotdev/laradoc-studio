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
  RuntimeProgressPayload,
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
  private sidebarSlot!: HTMLElement;
  private headerSlot!: HTMLElement;
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

    // Sidebar Container
    this.sidebarSlot = document.createElement('div');
    this.sidebarSlot.style.display = 'contents';
    this.sidebarSlot.appendChild(this.sidebar.render());
    this.appRoot.appendChild(this.sidebarSlot);

    // Main Viewport
    const mainViewport = document.createElement('main');
    mainViewport.className = 'main-viewport';

    // Header Slot
    this.headerSlot = document.createElement('div');
    this.headerSlot.style.display = 'contents';
    this.headerSlot.appendChild(this.header.render());
    mainViewport.appendChild(this.headerSlot);

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
    this.sidebar.setActiveTab(tab);
    this.updateSidebar();
    this.updateHeader();
    this.renderCurrentView();
  }

  private updateSidebar() {
    const anyRunning = this.services.some((s) => s.state === 'running');
    this.sidebar.updateStatus(anyRunning, this.emails.length);
    this.sidebarSlot.innerHTML = '';
    this.sidebarSlot.appendChild(this.sidebar.render());
  }

  private updateHeader() {
    const titles: Record<TabId, string> = {
      dashboard: 'Services',
      sites: 'Sites',
      logs: 'Logs',
      mail: 'Mail',
      settings: 'Settings',
    };
    const anyRunning = this.services.some((s) => s.state === 'running');
    this.header.update(titles[this.activeTab], anyRunning);

    this.headerSlot.innerHTML = '';
    this.headerSlot.appendChild(this.header.render());
  }

  private renderCurrentView() {
    this.contentSlot.innerHTML = '';

    switch (this.activeTab) {
      case 'dashboard':
        this.dashboard.update(this.services, this.settings.active_php);
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

      this.updateSidebar();
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
        this.dashboard.update(this.services, this.settings.active_php);
        this.contentSlot.innerHTML = '';
        this.contentSlot.appendChild(this.dashboard.render());
      }
      const anyRunning = this.services.some((s) => s.state === 'running');
      this.sidebar.updateStatus(anyRunning, this.emails.length);
      const dot = document.getElementById('global-status-dot');
      const text = document.getElementById('global-status-text');
      if (dot && text) {
        dot.className = `status-dot ${anyRunning ? 'running' : ''}`;
        text.innerText = anyRunning ? 'Running' : 'Stopped';
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
        mailBadge.classList.add('active');
      }
    });

    // Service state changed event
    listenEvent<void>('service:state_changed', () => {
      this.refreshData();
    });

    // On-demand runtime provisioning progress event
    listenEvent<RuntimeProgressPayload>('runtime:progress', (payload) => {
      const svc = this.services.find((s) => s.kind === payload.service);
      if (svc) {
        svc.state = 'installing';
        svc.progress_percent = payload.percent;
      }
      if (this.activeTab === 'dashboard') {
        this.dashboard.update(this.services, this.settings.active_php);
        this.contentSlot.innerHTML = '';
        this.contentSlot.appendChild(this.dashboard.render());
      }
    });
  }
}


// Initialize on DOM load
window.addEventListener('DOMContentLoaded', () => {
  new App();
});
