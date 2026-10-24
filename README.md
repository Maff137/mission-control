# Mission Control

**A local-first application for tracking study time, building consistency, and visualizing long-term progress.**

## Overview

Mission Control is a personal productivity and study-tracking application designed to help users understand how they spend their study time, maintain consistent habits, and monitor their progress.

The project prioritizes accurate time tracking, meaningful statistics, a clean user experience, and full control over personal data.

## Planned Features

- **Study Sessions:** Record study time manually or through a built-in timer with pause tracking.
- **Study Areas:** Organize activities into customizable areas, with support for multiple topics and areas within a session.
- **Daily Streaks:** Track consistency using a minimum of 10 minutes of effective study per calendar day.
- **Progress Dashboard:** Monitor total study hours, current streaks, personal records, and accumulated progress.
- **Activity Heatmap:** Visualize daily study activity through a GitHub-style contribution calendar.
- **Study History:** Review, edit, and organize historical study records.
- **Backup and Restore:** Export and restore personal data through portable backups.

These are approved product directions and requirements, not claims of completed functionality.

## Design Principles

Mission Control follows five core principles:

1. **Local-first:** Core features must work without an internet connection.
2. **Data ownership:** Study records belong to the user and are stored in native application-managed storage.
3. **Simplicity:** A focused, intuitive interface without unnecessary complexity.
4. **Reliability:** Accurate calculations, durable persistence, and safe data recovery.
5. **Maintainability:** Modular architecture, clear responsibilities, and testable components.

## Platform and Technology

Mission Control is being developed as an installable application using **Tauri 2**.

The intended platforms are macOS, Windows, Linux, Android, and iOS, subject to compatibility testing and platform-specific requirements.

The implemented foundation uses:

| Component | Technology | Status |
|---|---|---|
| Application framework | Tauri 2 | Approved |
| Frontend | React, TypeScript, Vite | Implemented; macOS foundation validated |
| Native integration | Rust | Tauri entry point and versioned SQLite migration registration implemented |
| Local database | SQLite via official Tauri SQL plugin | B2 persistence and B3 two-build schema upgrade validated on macOS ARM64 |

Broader platform support and the production study schema still require implementation and validation.

## Development Status

**Current phase: Project foundation and architectural validation.**

The project is being rebuilt from scratch while preserving previously approved functional requirements.

B1 is completed and integrated into `main`. B2 native SQLite persistence is implemented and validated on macOS ARM64, including separate development/production databases and record retention after closing and reopening both bundles. The user independently confirmed save/close/reopen persistence. B3 automated verification passed 9 Vitest tests and 4 Rust tests, including the B2-to-B3 SQLite upgrade. The native two-build upgrade from `b75d65f` to `52dadb7` also passed with a dedicated test identifier.

Stage 2 remains in progress. Full device restart has not been tested by the user or verified in the recorded implementation checks. Backup/restore and additional platforms remain pending architectural verification. Study workflows are not implemented.

No production-ready release or installation package is currently available.

## Repository Structure

```text
mission-control/
├── app/          # React/Vite frontend, persistence tests, and src-tauri/ native SQLite integration
├── docs/         # Project state and specifications
├── tests/        # Reserved for future repository-level tests
├── AGENTS.md     # Coding agent instructions
├── .gitignore
└── README.md
```

The structure will evolve as implementation progresses.

## Getting Started

Stage B2 extends the native welcome screen with a temporary SQLite persistence test. Study features are not implemented.

### Prerequisites

- Node.js 24 LTS and npm (validated with Node 24.21.0 and npm 11.19.0).
- Stable Rust with Cargo; `rustup component add rustfmt clippy` for native quality checks (validated with Rust 1.99.1).
- Platform dependencies from the [official Tauri prerequisites](https://v2.tauri.app/start/prerequisites/). On macOS desktop, install Xcode Command Line Tools with `xcode-select --install`. Full Xcode is needed for iOS work, which remains outside the validated B2 scope.

### Development and validation

Run from `app/`:

```sh
npm ci
npm run desktop:dev
```

The Tauri CLI starts and stops the Vite development server automatically. Development uses localhost; the packaged application embeds the built frontend and does not require Node, a user-managed server, an account, or network access during normal use.

```sh
npm run check          # TypeScript and Vitest tests
npm run build          # Frontend production assets
npm run rust:check     # Cargo check using Cargo.lock
npm run rust:test      # Native SQLite migration/durability tests
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
npm run desktop:build -- -- --locked  # Production macOS .app bundle
# Isolated development .app bundle for native persistence checks:
npm run tauri -- build --debug --config src-tauri/tauri.dev.conf.json --bundles app -- --locked
```

`npm run dev` and `npm run preview` are frontend-only helpers. `npm run tauri -- <command>` exposes the native CLI. The desktop build command currently targets the validated macOS `.app` format; other platform packaging remains unverified. Lockfiles are committed for reproducible dependency resolution.

The macOS bundle is generated at `app/src-tauri/target/release/bundle/macos/Mission Control.app`. Open it through Finder, or copy it to Applications for local use. This is a local development bundle, not a signed/notarized distribution release. Template icons remain placeholders.

Automated tests cover the TypeScript persistence boundary, native SQLite migration rollback and file reopen, and server-rendered application identity and the bundled-assets/window configuration. Native launch and packaged rendering require a desktop smoke check; these tests are not end-to-end WebView automation. See [Project State](docs/STATE.md) for actual results and limitations.

## SQLite persistence prototype

The official `tauri-plugin-sql` (SQLite feature) and `@tauri-apps/plugin-sql` use `sqlite:mission-control.db`. Rust registers migration versions 1 and 2 from `app/src-tauri/migrations/`. Version 2 (`0002_add_test_record_note.sql`) adds `note TEXT NOT NULL DEFAULT ''` without changing the existing persistence interface. The SQL plugin preloads the database and applies transactional SQLx migrations during native startup; a migration error aborts startup before database operations are available. This also avoids the plugin 2.6.0 load-retry path consuming migration registration after a failed migration. The plugin configuration requires a direct `serde_json` dependency for Tauri context generation; it was already present transitively. Initialization does not seed, delete, replace, or reset records. Runtime load/query errors are displayed in the test interface; migration errors now fail native startup. SQL lives in a typed persistence module, with parameter binding and a concurrent-write guard.

On macOS the plugin resolves the native application configuration directory under `~/Library/Application Support/`:

- Production: `com.missioncontrol.desktop/mission-control.db`.
- Development: `com.missioncontrol.desktop.dev/mission-control.db`.

`npm run desktop:dev` explicitly loads `src-tauri/tauri.dev.conf.json`, separating development storage without changing the production identifier. Plain `tauri dev` does not load that override; use the npm command. Production packaging uses the base configuration. Permissions enable only SQL load, select, and execute. There is no browser-storage fallback.

### Schema migration safety (B3)

Migrations are forward-only and versioned. Never edit an applied migration's SQL, description, or version; add the next numbered file and register it in Rust. Migration 1 remains byte-for-byte unchanged. SQLx's `_sqlx_migrations` is the only migration history; unknown applied versions and checksum mismatches fail safely without reset or downgrade.

Four Rust tests use isolated temporary SQLite files and the exact registered SQL, migration kinds, and SQLx engine. They verify v1-to-v2 preservation of multiple IDs, values, timestamps and counts, empty note defaults, new inserts, fresh schema/history, unchanged history on reopen, transactional DDL/data rollback after intentional failure, recovery, future-version/checksum rejection, integrity, and constraints. Frontend checks cover startup failure blocking dependent operations and native preload configuration. No development or production user databases are opened by these tests.

B3 is complete for the validated macOS ARM64 scope. Automated validation passed `npm run check` (9 Vitest tests), `npm run build`, `npm run rust:check`, `npm run rust:test` (4 Rust tests), formatting, Clippy with all targets, and the locked desktop bundle build.

The native two-build upgrade passed on macOS 27.0.1: detached worktrees at B2 `b75d65f` and B3 `52dadb7` produced separate debug `.app` bundles using the same dedicated identifier, `com.missioncontrol.desktop.b3upgrade20261024`. Three records were saved through the B2 interface, then B2 was fully quit. B3 loaded all three, saved a fourth, and retained all four after a full quit/reopen. Closed-database snapshots confirmed exact original IDs, values and timestamps, successful migration versions 1/2, empty note defaults, unchanged migration history on reopen, and integrity `ok`. Bundle identifiers and observed open SQLite paths confirmed test storage isolation; production/development databases were not opened by the validation. No applied migrations or application source were changed. See [Project State](docs/STATE.md#stage-b3--safe-database-schema-migrations) for build commands, evidence and remaining architectural checks.

For future native upgrade checks, use a fresh dedicated identifier shared by both builds, never the production/development identifiers or an already upgraded test database with the older build.

### Manual macOS durability check

1. Run `npm run desktop:build -- -- --locked` from `app/`.
2. With the development server stopped, open `app/src-tauri/target/release/bundle/macos/Mission Control.app` through Finder.
3. Enter a recognizable test value, click **Save**, and confirm the success message and saved record. Blank input must keep Save disabled.
4. Quit with Command-Q, reopen the same bundle, and confirm the record appears automatically.
5. In Finder, use **Go → Go to Folder** for `~/Library/Application Support/com.missioncontrol.desktop/`. Confirm `mission-control.db` is outside the repository. Do not delete it to recover from errors.
6. Repeat using `npm run desktop:dev` with a different value; production and development records should be separate. Stop development, then reopen the production bundle to confirm isolation.

This is a temporary persistence prototype, not the final study schema. Device reboot, backup/restore, signing, native startup-failure presentation, and other platforms require later validation.

## Documentation

For current decisions, approved requirements, development priorities, and unresolved technical questions, refer to:

- [Project State](docs/STATE.md)

The project state document is the authoritative reference for the current development phase. After every development task, agents must update and synchronize both `README.md` and `docs/STATE.md` before the final commit and push, review their consistency, and confirm both updates in the completion report. See [Agent Instructions](AGENTS.md) for the mandatory workflow.

---

**Mission Control — Track your time. Build consistency. Understand your progress.**
