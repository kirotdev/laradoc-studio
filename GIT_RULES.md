# Git Rules & Branching Strategy for Laradoc Studio

This document defines the official Git governance, branching model, commit standards, and pull request workflow for the **Laradoc Studio** project.

---

## 1. Branching Architecture

Laradoc Studio employs a structured **Git Flow / Scaled Trunk Hybrid** optimized for cross-platform desktop application development.

```
  main (Production Releases, Tags v0.1.0, v0.2.0)
   ▲
   │ (Release PR / Hotfix PR)
   │
  develop (Active Integration Branch)
   ▲          ▲          ▲
   │          │          │
feat/*      fix/*      docs/*
```

### Branch Categories & Naming Rules

| Branch | Base | Target | Naming Pattern | Description |
|---|---|---|---|---|
| **`main`** | — | — | `main` | Production-ready releases. Never commit directly. Protected. |
| **`develop`** | `main` | `main` | `develop` | Primary integration branch for upcoming releases. Protected. |
| **Feature** | `develop` | `develop` | `feat/<scope>-<short-description>` | New features or capabilities. |
| **Bugfix** | `develop` | `develop` | `fix/<scope>-<short-description>` | Fixes for issues found in development. |
| **Release** | `develop` | `main` & `develop` | `release/v<SemVer>` | Release staging, version bumps, release notes. |
| **Hotfix** | `main` | `main` & `develop` | `hotfix/v<SemVer>` | Urgent production patches. |
| **Refactor** | `develop` | `develop` | `refactor/<scope>-<short-description>` | Code improvements without behavioral change. |
| **Docs** | `develop` | `develop` | `docs/<short-description>` | Documentation additions or updates. |

### Valid Branch Name Examples
- `feat/supervisor-job-objects`
- `feat/pki-root-ca-macos`
- `feat/hosts-elevation-polkit`
- `fix/port-conflict-sysinfo`
- `fix/tray-close-to-minimize`
- `release/v0.1.0`
- `hotfix/v0.1.1`

---

## 2. Commit Message Standards (Conventional Commits 1.0.0)

All commits must follow the **Conventional Commits** specification:

```
<type>(<scope>): <short imperative description>

[optional detailed body explaining WHY and WHAT changed]

[optional footer(s): Closes #123, Breaking Changes]
```

### Allowed Types
- **`feat`**: Introduces a new user-facing or architectural feature.
- **`fix`**: Patches a bug or addresses an unexpected error.
- **`docs`**: Changes documentation, readmes, or inline comments only.
- **`refactor`**: Code reorganization without changing external behavior or fixing bugs.
- **`perf`**: Performance optimization (memory, startup time, CPU).
- **`build`**: Changes affecting Tauri build, Cargo.toml, Vite configs, dependencies.
- **`ci`**: Updates to CI/CD workflows (`.github/workflows/`).
- **`chore`**: Maintenance, housecleaning, tool updates.
- **`test`**: Adding or correcting tests.

### Allowed Scopes
- `supervisor` (Windows Job Objects, Unix process groups, child process management)
- `pki` (Root CA generation, wildcard certs, OS trust stores)
- `hosts` (Hosts file parsing, delimiter safety, privilege elevation)
- `vhost` (Virtual host generator, directory watcher, Nginx templates)
- `tray` (System tray icon, menus, quick actions)
- `mail` (Local SMTP catcher, MIME parsing, inbox)
- `ui` (Desktop frontend, components, styling, themes)
- `ipc` (Tauri commands, events, state bridges)
- `config` (Environment, default settings, port configs)

### Examples
#### Good Commits
- `feat(supervisor): implement Windows Job Objects with kill-on-close limit`
- `feat(pki): auto-install local Root CA into macOS System.keychain via security`
- `fix(hosts): handle permission denied gracefully when user rejects osascript prompt`
- `fix(vhost): debounce directory events to 500ms preventing duplicate nginx reloads`
- `docs(git): define official branching rules and PR requirements`

#### Prohibited Anti-Patterns
- `update stuff` ❌ (Unclear type and purpose)
- `WIP` ❌ (Never commit WIP to integration branches)
- `fixed bug` ❌ (Missing scope and explanation)
- `feat: added everything` ❌ (Too broad, unatomic)

---

## 3. Branch Protection & PR Governance

1. **Direct Pushes Disallowed:**
   - Both `main` and `develop` MUST have direct push restrictions enabled in GitHub / GitLab.
2. **Pull Request Requirements:**
   - At least 1 code review approval required before merging.
   - All CI matrix checks (Ubuntu, macOS, Windows) must pass.
   - Branch must be rebased or cleanly up-to-date with target branch before merge.
   - PR description must follow the `.github/pull_request_template.md`.
3. **Merge Strategy:**
   - Feature branches into `develop`: **Squash and Merge** (to maintain clean linear history on `develop`) or **Rebase and Merge**.
   - `develop` into `main`: **Merge Commit** (preserving the release milestone history).
4. **Desktop Safety Checklist for PRs:**
   - [ ] No zombie processes left when application exits.
   - [ ] No private keys (`.key`), certificates (`.crt`), or `.env` files staged.
   - [ ] Tested on target operating system (Windows, macOS, or Linux).
   - [ ] Cross-platform compilation guards (`#[cfg(target_os = "...")]`) correctly verified.

---

## 4. Git Hooks Configuration

This repository provides pre-configured Git hooks in `.githooks/`:
- **`pre-commit`**: Prevents accidental commits of private keys, certificates, large binaries, or trailing whitespace.
- **`commit-msg`**: Validates commit message formatting against the Conventional Commits specification.

### Activating the Hooks (Run Once)
```bash
git config core.hooksPath .githooks
chmod +x .githooks/pre-commit .githooks/commit-msg
```

---

## 5. Developer Quick Reference

### Starting a New Feature
```bash
git checkout develop
git pull origin develop
git checkout -b feat/supervisor-unix-signals
```

### Staging and Committing Changes
```bash
git add src-tauri/src/supervisor/unix.rs
git commit -m "feat(supervisor): implement graceful SIGTERM and SIGKILL timeout escalation"
```

### Keeping Feature Branch Fresh
```bash
git fetch origin
git rebase origin/develop
```

### Preparing a Release
```bash
git checkout develop
git checkout -b release/v0.1.0
# Update version numbers in Cargo.toml and tauri.conf.json
git commit -am "chore(release): bump version to v0.1.0"
# Open PR to main, merge, then tag
git checkout main
git pull
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```
