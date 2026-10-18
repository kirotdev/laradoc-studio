import { CapturedEmail, invokeCommand } from '../lib/tauri';
import { iconTrash } from '../lib/icons';

export class MailViewer {
  private emails: CapturedEmail[] = [];
  private selectedEmailId: string | null = null;
  private onAction: () => void;

  constructor(onAction: () => void) {
    this.onAction = onAction;
  }

  public update(emails: CapturedEmail[]) {
    this.emails = emails;
    if (!this.selectedEmailId && emails.length > 0) {
      this.selectedEmailId = emails[0].id;
    }
  }

  public appendEmail(email: CapturedEmail) {
    this.emails.unshift(email);
    if (!this.selectedEmailId) {
      this.selectedEmailId = email.id;
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
      <div style="display: flex; justify-content: flex-end; margin-bottom: 12px; flex-shrink: 0;">
        <button class="btn btn-secondary btn-sm" id="btn-clear-inbox">
          ${iconTrash()} Clear Inbox
        </button>
      </div>

      <div class="mail-split">
        <!-- Inbox Sidebar -->
        <div class="mail-sidebar">
          <div class="mail-sidebar-header">
            <span style="font-size: 11.5px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-muted);">
              Inbox (${this.emails.length})
            </span>
          </div>
          <div class="mail-list-scroll" id="mail-list-container">
            ${
              this.emails.length === 0
                ? `<div style="color: var(--text-muted); text-align: center; padding: 48px 16px; font-size: 12px;">No captured emails.<br>Send from PHP/Laravel via port 1025.</div>`
                : ''
            }
          </div>
        </div>

        <!-- Detail Preview Pane -->
        <div class="mail-preview-pane" id="mail-detail-container">
          ${this.renderDetail()}
        </div>
      </div>
    `;

    const listContainer = container.querySelector('#mail-list-container')!;
    this.emails.forEach((email) => {
      const item = document.createElement('div');
      item.className = `mail-item ${email.id === this.selectedEmailId ? 'selected' : ''}`;

      item.innerHTML = `
        <div style="display: flex; justify-content: space-between; font-size: 11px; color: var(--text-muted); margin-bottom: 3px;">
          <span>${email.date}</span>
          <span>${(email.size_bytes / 1024).toFixed(1)} KB</span>
        </div>
        <div style="font-size: 13px; font-weight: 600; color: var(--text-primary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; margin-bottom: 2px;">
          ${email.subject || '(No Subject)'}
        </div>
        <div style="font-size: 11.5px; color: var(--text-secondary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">
          ${email.from}
        </div>
      `;

      item.addEventListener('click', () => {
        this.selectedEmailId = email.id;
        const detailContainer = container.querySelector('#mail-detail-container');
        if (detailContainer) {
          detailContainer.innerHTML = this.renderDetail();
        }
        listContainer.querySelectorAll('.mail-item').forEach((i) => i.classList.remove('selected'));
        item.classList.add('selected');
      });

      listContainer.appendChild(item);
    });

    const btnClear = container.querySelector('#btn-clear-inbox');
    if (btnClear) {
      btnClear.addEventListener('click', async () => {
        try {
          await invokeCommand('clear_emails');
          this.emails = [];
          this.selectedEmailId = null;
          this.onAction();
        } catch (err) {
          console.warn('Clear emails error:', err);
        }
      });
    }

    return container;
  }

  private renderDetail(): string {
    const email = this.emails.find((e) => e.id === this.selectedEmailId);
    if (!email) {
      return `
        <div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-muted); font-size: 13px;">
          Select a message to view
        </div>
      `;
    }

    return `
      <div class="mail-meta-header">
        <h3 style="font-size: 15px; font-weight: 600; color: var(--text-primary); margin-bottom: 8px;">
          ${email.subject || '(No Subject)'}
        </h3>
        <div style="display: flex; flex-direction: column; gap: 3px; font-size: 12px; color: var(--text-secondary);">
          <div><span style="color: var(--text-muted); width: 42px; display: inline-block;">From:</span> <strong>${email.from}</strong></div>
          <div><span style="color: var(--text-muted); width: 42px; display: inline-block;">To:</span> ${email.to.join(', ')}</div>
          <div><span style="color: var(--text-muted); width: 42px; display: inline-block;">Date:</span> ${email.date}</div>
        </div>
      </div>

      <div class="mail-body-render">
        ${
          email.body_html
            ? email.body_html
            : `<pre style="font-family: var(--font-mono); font-size: 12px; line-height: 1.5; white-space: pre-wrap; color: var(--text-primary);">${email.body_text}</pre>`
        }
      </div>
    `;
  }
}
