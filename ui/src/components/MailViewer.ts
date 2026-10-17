import { CapturedEmail, invokeCommand } from '../lib/tauri';

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

    container.innerHTML = `
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
        <div>
          <h3 style="font-size: 14px; font-weight: 700;">Local Mail Catcher (Port 1025)</h3>
          <p style="font-size: 12px; color: var(--text-muted);">
            All outbound PHP <code>mail()</code> and Laravel SMTP emails are intercepted locally.
          </p>
        </div>
        <button class="btn btn-secondary btn-sm" id="btn-clear-inbox">🗑️ Clear Inbox</button>
      </div>

      <div class="mail-split-pane">
        <!-- Inbox List -->
        <div class="mail-inbox">
          <div class="inbox-header">
            <span style="font-size: 11px; font-weight: 600; text-transform: uppercase; color: var(--text-muted);">
              Inbox (${this.emails.length})
            </span>
          </div>
          <div class="mail-list" id="mail-list-container">
            ${
              this.emails.length === 0
                ? `<div style="color: var(--text-muted); text-align: center; padding: 40px; font-size: 12px;">Inbox is empty.<br>Trigger an email from Laravel via port 1025.</div>`
                : ''
            }
          </div>
        </div>

        <!-- Detail Preview -->
        <div class="mail-detail" id="mail-detail-container">
          ${this.renderDetail()}
        </div>
      </div>
    `;

    const listContainer = container.querySelector('#mail-list-container')!;
    this.emails.forEach((email) => {
      const item = document.createElement('div');
      item.className = `mail-item ${email.id === this.selectedEmailId ? 'selected' : ''}`;

      item.innerHTML = `
        <div class="mail-item-header">
          <span>${email.date}</span>
          <span>${(email.size_bytes / 1024).toFixed(1)} KB</span>
        </div>
        <div class="mail-subject">${email.subject || '(No Subject)'}</div>
        <div class="mail-from">From: ${email.from}</div>
      `;

      item.addEventListener('click', () => {
        this.selectedEmailId = email.id;
        this.render();
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
          Select an email to view details
        </div>
      `;
    }

    return `
      <div class="mail-detail-header">
        <h3 style="font-size: 15px; font-weight: 700; color: var(--text-primary); margin-bottom: 6px;">
          ${email.subject || '(No Subject)'}
        </h3>
        <div style="font-size: 12px; color: var(--text-secondary); display: flex; flex-direction: column; gap: 2px;">
          <div><strong>From:</strong> ${email.from}</div>
          <div><strong>To:</strong> ${email.to.join(', ')}</div>
          <div><strong>Date:</strong> ${email.date}</div>
        </div>
      </div>

      <div class="mail-detail-body">
        ${
          email.body_html
            ? `<div class="mail-body-rendered">${email.body_html}</div>`
            : `<pre style="font-family: var(--font-mono); font-size: 12px; white-space: pre-wrap; color: var(--text-primary);">${email.body_text}</pre>`
        }
      </div>
    `;
  }
}
