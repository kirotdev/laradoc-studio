// Typed IPC wrapper for Tauri v2 commands with graceful web-browser fallback
import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export { isTauri };

export interface ServiceStatus {
  name: string;
  kind: 'nginx' | 'php82' | 'php83' | 'php84' | 'php85' | 'mysql' | 'redis' | 'mailpit';
  state: 'stopped' | 'starting' | 'running' | 'error' | 'conflict';
  port: number;
  pid: number | null;
  uptime_seconds: number;
  memory_bytes: number;
  cpu_percent: number;
  error_message?: string;
  conflict_process?: string;
}

export interface SiteInfo {
  name: string;
  domain: string;
  path: string;
  public_path: string;
  php_version: '8.2' | '8.3' | '8.4' | '8.5' | 'default';
  has_ssl: boolean;
  is_secured: boolean;
}

export interface CapturedEmail {
  id: string;
  from: string;
  to: string[];
  subject: string;
  date: string;
  body_text: string;
  body_html: string;
  headers: Record<string, string>;
  size_bytes: number;
}

export interface LogEntry {
  id: string;
  timestamp: string;
  service: string;
  level: 'info' | 'warn' | 'error' | 'debug';
  message: string;
}

export interface AppSettings {
  active_php: '8.2' | '8.3' | '8.4' | '8.5';
  www_directory: string;
  root_ca_trusted: boolean;
  hosts_synced: boolean;
  auto_start_disabled: boolean;
}

// Safe invoke wrapper with static import and graceful mock fallback
export async function invokeCommand<T>(
  cmd: string,
  args: Record<string, unknown> = {}
): Promise<T> {
  if (isTauri()) {
    try {
      return await invoke<T>(cmd, args);
    } catch (err) {
      console.error(`[IPC ERROR] invoke("${cmd}") failed:`, err);
      // In development mode, fallback to mock data if IPC is unavailable
      if (import.meta.env?.DEV) {
        console.warn(`[IPC Fallback] Serving mock data for "${cmd}"`);
        return getMockData<T>(cmd, args);
      }
      throw err;
    }
  }

  // Standalone / Browser Mock Data for UI previews
  return getMockData<T>(cmd, args);
}

// Safe event listener wrapper returning a guaranteed cleanup function
export async function listenEvent<T>(
  event: string,
  handler: (payload: T) => void
): Promise<() => void> {
  if (isTauri()) {
    try {
      const unlisten: UnlistenFn = await listen<T>(event, (e) => handler(e.payload));
      return unlisten;
    } catch (err) {
      console.warn(`[IPC] listen("${event}") failed, falling back to no-op:`, err);
      return () => {};
    }
  }

  // No-op for browser preview
  return () => {};
}

// Mock data generator for browser testing & verification
function getMockData<T>(cmd: string, _args: Record<string, unknown>): T {
  console.log(`[Mock IPC] Invoked "${cmd}" with args:`, _args);

  switch (cmd) {
    case 'get_services_status':
      return [
        {
          name: 'Nginx Web Server',
          kind: 'nginx',
          state: 'stopped',
          port: 80,
          pid: null,
          uptime_seconds: 0,
          memory_bytes: 0,
          cpu_percent: 0,
        },
        {
          name: 'PHP 8.3 FPM (Active)',
          kind: 'php83',
          state: 'stopped',
          port: 9083,
          pid: null,
          uptime_seconds: 0,
          memory_bytes: 0,
          cpu_percent: 0,
        },
        {
          name: 'PHP 8.2 FPM',
          kind: 'php82',
          state: 'stopped',
          port: 9082,
          pid: null,
          uptime_seconds: 0,
          memory_bytes: 0,
          cpu_percent: 0,
        },
        {
          name: 'PHP 8.4 FPM',
          kind: 'php84',
          state: 'stopped',
          port: 9084,
          pid: null,
          uptime_seconds: 0,
          memory_bytes: 0,
          cpu_percent: 0,
        },
        {
          name: 'PHP 8.5 FPM',
          kind: 'php85',
          state: 'stopped',
          port: 9085,
          pid: null,
          uptime_seconds: 0,
          memory_bytes: 0,
          cpu_percent: 0,
        },
        {
          name: 'MySQL Database',
          kind: 'mysql',
          state: 'stopped',
          port: 3306,
          pid: null,
          uptime_seconds: 0,
          memory_bytes: 0,
          cpu_percent: 0,
        },
        {
          name: 'Redis Cache',
          kind: 'redis',
          state: 'stopped',
          port: 6379,
          pid: null,
          uptime_seconds: 0,
          memory_bytes: 0,
          cpu_percent: 0,
        },
        {
          name: 'Mail Catcher (SMTP)',
          kind: 'mailpit',
          state: 'stopped',
          port: 1025,
          pid: null,
          uptime_seconds: 0,
          memory_bytes: 0,
          cpu_percent: 0,
        },
      ] as unknown as T;

    case 'list_sites':
      return [
        {
          name: 'laradoc-portal',
          domain: 'laradoc-portal.test',
          path: '/Users/myatkyawthu/Documents/laradoc-studio/www/laradoc-portal',
          public_path: '/Users/myatkyawthu/Documents/laradoc-studio/www/laradoc-portal/public',
          php_version: '8.3',
          has_ssl: true,
          is_secured: true,
        },
        {
          name: 'api-service',
          domain: 'api-service.test',
          path: '/Users/myatkyawthu/Documents/laradoc-studio/www/api-service',
          public_path: '/Users/myatkyawthu/Documents/laradoc-studio/www/api-service/public',
          php_version: '8.4',
          has_ssl: true,
          is_secured: true,
        },
      ] as unknown as T;

    case 'get_emails':
      return [
        {
          id: 'mock-1',
          from: 'system@laradoc-portal.test',
          to: ['developer@example.com'],
          subject: 'Welcome to Laradoc Studio Local Mail Catcher',
          date: new Date().toLocaleTimeString(),
          body_text: 'Your local SMTP server is running on 127.0.0.1:1025. All PHP mail() and Laravel emails will appear here.',
          body_html: '<p>Welcome! Your local SMTP server is active on <strong>127.0.0.1:1025</strong>.<br>Emails sent from Laravel via <code>MAIL_PORT=1025</code> are captured with zero internet transmission.</p>',
          headers: {
            'From': 'system@laradoc-portal.test',
            'To': 'developer@example.com',
            'Subject': 'Welcome to Laradoc Studio Local Mail Catcher',
            'Content-Type': 'text/html; charset=UTF-8'
          },
          size_bytes: 1420
        }
      ] as unknown as T;

    case 'get_recent_logs':
      return [
        {
          id: 'log-1',
          timestamp: new Date().toLocaleTimeString(),
          service: 'System',
          level: 'info',
          message: 'Laradoc Studio initialized with cold-start policy. All services stopped by default.'
        },
        {
          id: 'log-2',
          timestamp: new Date().toLocaleTimeString(),
          service: 'PHP-Manager',
          level: 'info',
          message: 'FastCGI pools ready: 8.2 (9082), 8.3 (9083), 8.4 (9084), 8.5 (9085).'
        }
      ] as unknown as T;

    case 'get_settings':
      return {
        active_php: '8.3',
        www_directory: '/Users/myatkyawthu/Documents/laradoc-studio/www',
        root_ca_trusted: true,
        hosts_synced: true,
        auto_start_disabled: true,
      } as unknown as T;

    default:
      return {} as T;
  }
}
