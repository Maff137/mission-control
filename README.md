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

These are approved product directions and requirements, not claims of completed study functionality. A backup/restore prototype is implemented as part of the foundation; see B4 below.

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
| Native integration | Rust | Versioned migrations, guarded persistence commands, backup validation and restart restore |
| Local database | SQLite via official Tauri SQL plugin | B2 persistence and B3 two-build schema upgrade validated on macOS ARM64 |

Broader platform support and the production study schema still require implementation and validation.

## Development Status

**Current phase: Project foundation and architectural validation.**

The project is being rebuilt from scratch while preserving previously approved functional requirements.

B1 is completed and integrated into `main`. B2 native SQLite persistence is implemented and validated on macOS ARM64, including separate development/production databases and record retention after closing and reopening both bundles. The user independently confirmed save/close/reopen persistence. B3 automated verification passed 9 Vitest tests and 4 Rust tests, including the B2-to-B3 SQLite upgrade. The native two-build upgrade from `b75d65f` to `52dadb7` also passed with a dedicated test identifier.

B5 adds isolated process-termination and filesystem-fault integration tests, plus a native B2→B3→B4→current upgrade lineage and packaged SIGKILL recovery. Its verified scope and remaining acceptance gaps are recorded in [B5 resilience evidence](docs/B5-RESILIENCE-REPORT.md). B5 is partially verified; Stage 2 remains in progress. Full device restart has not been tested by the user or verified in the recorded implementation checks. B4 implements consistent export and restart-based restore; native B4 validation is recorded below. Additional platforms remain pending architectural verification. Study workflows are not implemented.

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

The native welcome screen includes temporary SQLite persistence and B4 backup/restore controls. Study features are not implemented.

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

The official Rust `tauri-plugin-sql` (SQLite feature) owns the `sqlite:mission-control.db` pool and migration preload. React uses `@tauri-apps/api` commands rather than generic SQL plugin access. Rust registers migration versions 1 and 2 from `app/src-tauri/migrations/`. Version 2 (`0002_add_test_record_note.sql`) adds `note TEXT NOT NULL DEFAULT ''` without changing the existing persistence interface. The SQL plugin preloads the database and applies transactional SQLx migrations during native startup; a migration error aborts startup before database operations are available. This also avoids the plugin 2.6.0 load-retry path consuming migration registration after a failed migration. The plugin configuration requires a direct `serde_json` dependency for Tauri context generation; it was already present transitively. Ordinary initialization does not seed, delete, replace, or reset records; explicitly confirmed B4 restoration runs before it. Runtime load/query errors are displayed in the test interface; migration errors now fail native startup. SQL now runs behind narrow Rust read/insert commands with parameter binding and a shared operation lock; React uses an injectable typed IPC boundary. Confirmed restores block native writes until restart.

On macOS the plugin resolves the native application configuration directory under `~/Library/Application Support/`:

- Production: `com.missioncontrol.desktop/mission-control.db`.
- Development: `com.missioncontrol.desktop.dev/mission-control.db`.

`npm run desktop:dev` explicitly loads `src-tauri/tauri.dev.conf.json`, separating development storage without changing the production identifier. Plain `tauri dev` does not load that override; use the npm command. Production packaging uses the base configuration. The main-window capability grants no generic SQL, filesystem or dialog plugin permissions; narrow application commands own all operations, including native dialogs. There is no browser-storage fallback.

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

This is a temporary persistence prototype, not the final study schema. Device reboot, signing, native startup-failure presentation, and other platforms require later validation. B4 backup/restore is described below.

## Backup and restore prototype (B4)

Use **Export Backup** to choose a new `.mcbackup` filename in a native save dialog. The Rust service uses SQLite `VACUUM INTO`, which includes committed WAL data in a consistent snapshot, then validates the result. It publishes the file atomically without overwriting existing files. Choose a new filename for each export. Destinations anywhere beneath the parent of the application configuration directory are blocked to protect application storage, including development/production siblings. The destination filesystem must support hard links; unsupported filesystems fail safely. Exports require free space for the snapshot.

Use **Restore Backup** to choose a local file. Internal application storage and files with SQLite WAL/SHM/journal companions cannot be selected as backups. Rust copies at most 64 MiB into a private staging directory and validates that copy. **Cancel** leaves the current database unchanged. **Replace data on next startup** explicitly confirms replacement of all records, without merging. Quit the application completely (Command-Q on macOS), then reopen the same bundle. Writes remain blocked after confirmation. The application processes restoration before the SQL plugin opens or migrates the database; records refresh only in the new process after successful startup.

### Format and compatibility

Format 1 is a standalone SQLite snapshot with `application_id = 0x4d434250` (`MCBP`) and `user_version = 1`, set only on the exported snapshot. No live schema or applied migration files are modified by export. Record IDs, values, notes, timestamps and complete SQLx migration history are preserved; physical page layout is not part of the format. Backups contain plaintext data, without encryption, authentication or a signature: use trusted sources and protect exported files.

This initial format accepts exactly the current B3 schema and migration versions 1/2, including descriptions and SQLx checksums. Unknown/future versions, altered constraints, extra schema objects and unrelated files are rejected; no backup-supplied migrations run. Future schema support must deliberately extend this compatibility policy. Validation checks the SQLite header, application/version identity, exact schema, migration history, text-field types, `integrity_check` and `foreign_key_check`. Limits are 512 bytes–64 MiB per file and 1,000,000 combined UTF-8 bytes per record, plus SQLite allocation/SQL/expression limits, a 2 MiB page cache and a 10-second VM progress budget (not a guarantee against stalled filesystem I/O). Larger existing records are never reset; export/restore reports an error.

### Recovery and storage lifecycle

A process-held native file lock prevents concurrent B4 instances for the same application storage. Confirmed restores have a durable, atomically published marker. Before replacement, startup creates and validates `restore-<uuid>/original.db` using SQLite, then persists an installation marker. The original main database and any `-wal`, `-shm`, and `-journal` files are retained together in that recovery directory. The replacement is prepared on the same filesystem and moved into place only before SQL plugin setup. The pending marker is removed only after plugin preload/migrations succeed. If interrupted after installation begins, the next startup validates and reinstalls the original snapshot before opening SQL. Invalid recovery snapshots stop startup without displacing existing files. Keep all recovery files when troubleshooting; never delete the live database to fix an error.

Recovery copies are retained without automatic pruning or a recovery-management UI. Unconfirmed staging directories and tracked interrupted export temporary files are cleaned on a later startup; published exports remain untouched. Files are flushed, and directories are synced on Unix. Windows directory durability, filesystem-specific behavior, OS power loss and applications/tools that ignore the process lock remain outside verified guarantees. External modification of application storage while Mission Control runs is unsupported.

Dependencies added directly in Rust are `tauri-plugin-dialog` (native dialogs; `rfd` and `tauri-plugin-fs` transitively), `serde` (structured errors), `uuid` (private operation IDs), `tokio` (shared operation mutex), and `libsqlite3-sys` (limits on SQLx's guarded native handle); SQLx moves from test-only to runtime use. Except for the dialog dependency tree, these were already present in Cargo.lock. The JavaScript SQL package is replaced by the Tauri core API. No frontend filesystem access is granted, and no cloud or network service is introduced.

**Validation (macOS ARM64):** Native dialogs, cancellation, confirmed restore through full quit/reopen, exact restored records/history, retained original recovery data, another reopening, and malformed-file rejection passed using only the dedicated `com.missioncontrol.desktop.b4test20261024` bundle/storage. Production/development user databases were not opened. The locked desktop bundle build passed.

**Automated verification:** 13 Vitest tests and 16 Rust tests passed, including multiple/empty records, exact records/history, WAL snapshots, restore/reopen, repeated restore, cancellation boundaries, incompatible/corrupt/truncated/oversized files, migration/checksum rejection, failed staging, interrupted export cleanup, replacement rollback, invalid recovery preservation and dev/prod isolation. Tests use unique temporary directories only. TypeScript/Vite, locked Rust check/test, rustfmt and Clippy with all targets passed. Native packaging and UI validation status are recorded in [Project State](docs/STATE.md#stage-b4--consistent-backup-export-and-safe-restore). Stage 2 remains in progress.

## Resilience verification (B5)

The Rust suite now passes **22 scenario tests**, including six B5 tests, with one subprocess helper ignored in the parent run and explicitly invoked by the harness. It reuses isolated B4 fixtures and compares complete records and migration metadata. B5 covers killed processes with committed WAL/uncommitted transactions, exclusive process locking, distinct restore/rollback data, durable confirmation boundaries, retained originals, invalid markers/formats, SQLite FULL/READONLY/BUSY errors, actual temporary-directory permission failures, and deterministic copy/flush/publication/rename failures. Fault hooks compile only in test builds; there is no production fault-injection API or new dependency.

Run B5 alone from `app/`:

```sh
cargo test --manifest-path src-tauri/Cargo.toml --locked b5_ -- --nocapture
cargo test --manifest-path src-tauri/Cargo.toml --locked b5_ -- --nocapture --test-threads=1
```

Both commands passed, along with 13 Vitest tests and all documented frontend/native, formatting, Clippy and release-packaging checks. Isolated macOS bundles verified the historical upgrade lineage, committed-record recovery after SIGKILL, concurrent-instance rejection, fail-closed startup with an invalid recovery marker and subsequent reopen. Native database integrity and foreign-key checks passed; exact snapshots retained original IDs, values, notes, timestamps and migration history. Normal development/production databases were not opened.

**B5 remains partially verified.** SIGKILL does not establish power-loss durability. Full device reboot, native UI handling of real disk/permission faults, every interior restore interruption in packaged startup, startup-error UX, WebView-data clearing, signed updater delivery and other platforms remain unverified. Stage 2 stays open. See the [B5 report](docs/B5-RESILIENCE-REPORT.md) for exact commands, per-scenario results, reproducible native procedures and limitations.

## Documentation

For current decisions, approved requirements, development priorities, and unresolved technical questions, refer to:

- [Project State](docs/STATE.md)

The project state document is the authoritative reference for the current development phase. After every development task, agents must update and synchronize both `README.md` and `docs/STATE.md` before the final commit and push, review their consistency, and confirm both updates in the completion report. See [Agent Instructions](AGENTS.md) for the mandatory workflow.

---

**Mission Control — Track your time. Build consistency. Understand your progress.**
