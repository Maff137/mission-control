# Mission Control — Project State

**Last updated:** 2026-10-25\
**Phase:** B6 compatibility verification; B5/B6 remain PARTIALLY VERIFIED and Stage 2 remains IN PROGRESS.
**Immediate delivery priority (2026-10-25):** Prepare study-data safety, then deliver a usable personal study workflow and stable macOS ARM64 installation before implementing the mandatory MVP updater. MVP target: November 2, 2026; targets are not completion guarantees. Stage 2 remains IN PROGRESS; B5/B6 remain PARTIALLY VERIFIED.
**Repository:** `Maff137/mission-control`  
**Historical reference:** `definicoes-mission-control.md` (October 20, 2026 compilation).

## Status conventions

- **APPROVED:** Product behavior or technical direction explicitly accepted for the rebuild.
- **RECOMMENDED:** Engineering proposal selected as the preferred direction, pending implementation and/or validation.
- **PLANNED:** Intended future work, not necessarily included in the first release.
- **IMPLEMENTED:** Present in the codebase; does not by itself establish acceptance.
- **VERIFIED:** Supported by recorded checks within their stated scope; not a universal guarantee.
- **OPEN:** Decision not finalized.
- **HISTORICAL:** Old implementation detail, not a requirement for the rebuild.

**Guiding rule:** Preserve approved product behavior and the newly approved platform decision. Do not automatically reuse the old architecture, technologies, codebase, roadmap, or teaching methodology. A specification or decision is not proof of implementation.

## 1. Current state

- **APPROVED:** Rebuild Mission Control from scratch because the previous technical foundation was unsatisfactory.
- **APPROVED:** Focus on delivering a professional, stable, maintainable product rather than using the project as a programming lesson.
- **APPROVED:** Keep project code and documentation entirely in English; conversations may be in Brazilian Portuguese.
- **APPROVED (2026-10-20):** Use **Tauri 2** as the primary platform for an installable, web-technology-based application; do not build a browser-only PWA as the primary product.
- **APPROVED:** Operate offline with data stored in native application-managed files, independently of browser history, caches, site storage and IndexedDB.
- **APPROVED:** No user-managed server, terminal command, localhost URL, cloud account or internet connection is required for normal use after installation.
- **APPROVED:** Target desktop and mobile devices, accepting practical minimum OS/WebView versions and platform-specific packaging; simultaneous cross-device synchronization is not a current priority.
- **RECOMMENDED:** React + TypeScript + Vite for the interface, Tauri's Rust layer for native operations, and SQLite for local persistence. Validate compatibility and durable storage before treating this implementation as established.
- **OPEN:** Exact supported OS versions, installers, final study schema and future migrations, future backup compatibility, cross-platform test matrix and release process.
- **PARTIALLY VERIFIED (2026-10-24):** B5 adds process-death, deterministic filesystem-fault and compatibility integration evidence: 24 Rust tests and 13 Vitest tests passed in the native acceptance follow-up; isolated native B2→B3→B4→current upgrade, SIGKILL recovery, concurrent-startup rejection and invalid-marker startup preservation passed. Native acceptance subsequently verified APFS/permission errors, eight packaged restore interruption boundaries, startup-error UX and WebView independence. M1 device reboot is PASS on macOS ARM64 (2026-10-25); power loss and remaining cross-platform/distribution acceptance remain pending; see [B5 report](B5-RESILIENCE-REPORT.md).
- **VERIFIED (2026-10-24):** Stage B4 adds consistent portable snapshots, bounded native validation and confirmed restart-only restoration with retained originals and interruption recovery. Automated checks and an isolated macOS ARM64 export/restore/reopen passed; see B4 below. Stage 2 remains open.
- **VERIFIED (2026-10-24):** Stage B2 adds native SQLite test-record persistence, transactional initialization, isolated dev storage, and packaged-app quit/reopen verification in both environments. See B2 results below; the wider foundation gate is still open.
- **HISTORICAL B1 VALIDATION (2026-10-24):** Stage B1 initializes the official Tauri 2 React/TypeScript/Vite template in `app/`, reduced to a static welcome screen and minimal Rust entry point. macOS ARM64 frontend checks, native checks, and `.app` packaging passed. The packaged window was visually inspected with the development server stopped. At the B1 milestone, no persistence or study features were implemented; B2 adds persistence as described below.
- **OBSERVED (2026-10-20):** The new GitHub repository had an empty `README.md` and no `STATE.md` on `main` before this document was drafted. No working features were verified in the new repository.

## 2. Preserved functional requirements

### 2.1 Study streak

- **APPROVED:** A calendar day qualifies after **10 minutes of effective study**.
- Combine multiple sessions and different study areas on the same day; count the qualifying day only once.
- Use the user's configured local calendar day, **not** a rolling 24-hour window.
- Do not transfer minutes between dates to meet the daily threshold.
- Exclude paused time, unreviewed sessions, discarded sessions, and sessions in trash from calculations.
- Recalculate affected daily totals, current streak, best streak and graphs after historical inserts, edits, deletions and restorations. Valid retroactive entries may repair a broken streak.
- **OPEN:** How the current streak is displayed before today's minimum is reached.

### 2.2 Dashboard and visualizations

- **APPROVED:** Display **current streak (days)**, **best streak (days)**, **study hours during the streak**, and **lifetime study hours** as distinct values.
- **APPROVED:** A cumulative study-hours line chart; its progression is flat on days without study and cannot decline as time advances. Historical corrections recompute the series.
- **APPROVED:** A GitHub-style study activity heatmap with intensity reflecting effective study time assigned to each day.
- **APPROVED:** Views and statistics by study area, and overall daily progress.
- **APPROVED:** Prioritize study hours over raw session count in primary indicators.
- **APPROVED:** Default accent color is **yellow**; UI should be readable and not overloaded.
- **OPEN:** Exact visual design, chart periods, heatmap thresholds, typography and layout.

### 2.3 Study areas

- **APPROVED:** Configurable study areas, not limited to programming (Python, German, English are examples, not fixed options).
- Areas have separate identities and editable **names and colors**.
- **APPROVED:** Allow duplicate area names with a warning, retaining separate IDs and associations.
- **APPROVED:** Archive areas without losing their historical sessions or totals.
- **OPEN:** Permanent deletion rules for areas with attached study records.

### 2.4 Manual study sessions

- **APPROVED:** Required: **duration, study area and study date**. Suggest today's date as the default.
- **APPROVED:** Optional: topic/title, start time, end time and free-form general notes.
- **APPROVED:** Duration is the authoritative required input. Starting time may suggest an ending time, which remains editable.
- **APPROVED:** Show warnings rather than prevent saving when duration and times disagree.
- **APPROVED:** Allow manually entered overlapping sessions with a prominent warning; do not silently deduplicate their time.
- **APPROVED:** Allow retroactive study dates, distinct from record creation timestamps; recalculate history as necessary.
- **APPROVED:** Do **not** add a dedicated `difficulty` field (explicitly removed from the earlier specification); such remarks may go in notes.

### 2.5 Timer and session review

- **APPROVED:** Start the timer **with one click** and without requiring an area, topic or target duration first.
- **APPROVED:** Only **one timed session** may occupy `running` or `paused` at a time. Manual entries remain possible.
- **APPROVED:** Support start, pause, resume and finish. Track **individual pause intervals**; paused minutes are excluded from effective study time.
- **APPROVED:** Persist timer state and timestamps in durable application storage so app closure, suspension or interruption does not silently erase session history.
- **APPROVED:** After **24 consecutive hours** left unresolved in `running`/`paused`, interrupt the timer and move the session to `needs_review`.
- **APPROVED:** Functional states include `running`, `paused`, `finished`, `needs_review` and `discarded`. Discarding an unfinished timer is different from deleting a saved session.
- **APPROVED:** `needs_review` sessions contribute **nothing** to streak, totals, line graphs, heatmap or study-area analytics until resolved.
- **APPROVED BEHAVIOR / IMPLEMENTATION OPEN:** Recovery flow should let the user accept calculated time, accept time until the last trustworthy activity, manually edit duration, or discard. Any inferred suggestion must be editable and confirmed.
- **OPEN:** Durable timer-write strategy, inactivity detection, last trustworthy activity, exact 24-hour transition mechanism and automated suggestion logic.

### 2.6 Multiple areas and topics within one session

- **APPROVED:** One session can allocate time to **multiple study areas and multiple topics**, including multiple topics within the same area.
- **APPROVED:** Allow manually dividing duration among allocations, including on timer completion. Preserve **one parent session**, not separate sessions per allocation.
- **APPROVED:** Warn about inconsistent allocation totals; offer correction without unnecessary blocking.
- **PLANNED:** Option to assign remaining time to the last selected area.
- **OPEN:** Rounding, allocation edge cases and precise pause handling across multiple areas.

### 2.7 Midnight and time zones

- **APPROVED:** A timed session crossing midnight remains **one session**, with effective minutes distributed across the relevant days.
- Example: 23:40–00:30 without pauses => 20 minutes on day one and 30 on day two.
- Pauses are excluded from each affected day's effective time; selecting either day opens the same original session.
- **APPROVED:** Use the user's time zone (initially inferred from the device, configurable by the user), never blindly assume UTC or any fixed time zone, for calendar and streak calculations.
- **OPEN:** Historical time-zone changes and daylight-saving transitions.

### 2.8 History and calendar

- **APPROVED:** Day-based study history with details, times, topic/area allocations and notes, connected with the calendar and heatmap.
- **APPROVED ORDERING:** Manual entries **without times first**, newest creation first; then timed entries in **reverse chronological order**.
- **APPROVED:** Never invent timestamps for entries without times merely to sort them.
- **PLANNED:** Gmail-inspired compact daily summaries, duration bars, study-area colors, expandable dates and filters.

### 2.9 Editing, warnings, archiving and trash

- **APPROVED:** Editing relevant dates, durations, times, pause intervals, areas, topics and notes must recompute derived statistics.
- **APPROVED:** Warnings rather than unnecessary hard blocks for retroactive records, duplicate area names, manually overlapping time ranges, inconsistent times/durations and imbalanced allocations.
- **APPROVED:** Prefer recoverable soft deletion. Trashed sessions stop contributing to all statistics; restored sessions contribute again after recalculation.
- **APPROVED:** Archiving areas must not destroy associated study history.
- **PLANNED:** Full trash-management UI backed by the selected local persistence layer.
- **OPEN:** Trash retention, permanent purge and audit behavior.

## 3. Planned features — not requirements of the first release

- **PLANNED:** Goals and objectives (potential properties include title, area, description, deadline, status and progress); definitive data model and calculation rules remain open.
- **PLANNED:** Additional weekly/monthly analytics, comparisons across areas, study times and trends.
- **PLANNED:** Responsive layouts and native mobile packaging, with desktop development first and early mobile compatibility checks.
- **PLANNED:** Optional **manual** transfer of portable backups between devices using OS sharing, AirDrop, Quick Share, USB or similar methods; full restoration and record merging are distinct operations. Automatic or concurrent sync is not required.
- **PLANNED/HISTORICAL:** Possible private alpha or hosted/SaaS service later; no obligation to implement accounts, payments, cloud services or multi-user support now.
- **NOT APPROVED AS MVP REQUIREMENTS:** AI, notifications, payments or additional gamification.

## 4. Platform and technical direction

**Decision date:** 2026-10-20. Tauri 2 is an approved platform decision; a proposed stack is not an implemented or validated stack.

| Area | Status |
| --- | --- |
| Primary platform | **APPROVED** — Tauri 2, installable application with a shared web-based UI and native platform integration. A browser-hosted PWA is not the primary target. |
| Target systems | **APPROVED DIRECTION** — macOS, Windows, Linux, Android and iOS, subject to platform-specific build, OS/WebView compatibility and validation. Exact minimum versions remain **OPEN**. |
| User experience | **APPROVED** — launch from an application icon; no terminal, manual server startup, localhost address, login or mandatory cloud account. |
| Runtime | **APPROVED** — fully functional offline after installation; no required backend service or remote database for core functionality. |
| Data ownership | **APPROVED** — user records reside in native application-controlled files on the user's device, not in browser-managed site data, localStorage, IndexedDB or a WebView cache. |
| Database | **APPROVED / IMPLEMENTED (B2)** — native SQLite through the official Tauri SQL plugin; versioned transactional migrations, verified on macOS. Other platforms remain unverified. |
| UI stack | **RECOMMENDED** — React, TypeScript and Vite; native operations through Tauri/Rust. Final dependency choices and module boundaries are **OPEN**. |
| Data durability | **APPROVED** — data should survive app closure, device restart, inactivity, browser cache/history clearing and correctly implemented app updates. Uninstallation may remove private app data; backups mitigate this and other loss scenarios. |
| Backup and restore | **APPROVED DIRECTION** — portable, user-controlled export/import with validation and consistent snapshots. B4 implements format 1 and retained recovery copies for the prototype; future compatibility, retention UI and automatic backup policy remain **OPEN**. |
| Cross-device synchronization | **NOT REQUIRED FOR MVP** — no concurrent profile sharing or automatic sync. **PLANNED** — manual backup transfer/import later; merging divergent device histories requires separate rules. |
| Authentication and hosting | **NOT REQUIRED FOR MVP** — no accounts, remote server or hosting dependency. Optional online services are not approved requirements. |
| Architecture and quality | **OPEN IMPLEMENTATION** — database schema, migrations, native storage permissions, error recovery, tests, packaging and cross-platform release pipeline. |

### 4.1. Storage and portability invariants

- **APPROVED:** Persist source records, session allocations, study areas, timer state and settings in a device-local durable store managed by the installed application.
- **APPROVED:** Do not make browser storage or WebView cache the authoritative record store; clearing browser data must not erase Mission Control records.
- **APPROVED:** Preserve user data across normal application updates through compatible migrations and robust failure handling; never silently reset a database after a schema or startup error.
- **APPROVED DIRECTION:** Provide portable backup export and validated restore, keeping stored data independent of a single installation when the user explicitly exports it.
- **PLANNED:** Design stable record IDs and versioned export metadata so future manual transfer or merging remains feasible without implementing sync now.
- **IMPLEMENTED (B2):** Official Tauri SQL plugin, native app configuration storage and versioned transactional migrations; macOS physical paths verified.
- **OPEN:** Other OS storage paths, final study schema, future backup compatibility, import conflict strategy, backup encryption and sensitive-data protection.
- **LIMITATION:** Native application storage is independent of browser eviction but not immune to disk failure, corruption, user deletion, OS cleanup or mobile app uninstallation. Backups remain necessary.

### 4.1.1. Personal-use data protection policy (approved 2026-10-25)

- **APPROVED:** Preserve the production application identifier `com.missioncontrol.desktop` and its existing native SQLite storage. Never delete, reset, overwrite, or repurpose the production database to resolve development or migration problems.
- **APPROVED:** Development, automated tests, experimental builds, and destructive acceptance tests must use separate application identifiers and isolated databases. Never execute destructive acceptance scenarios against personal study data.
- **APPROVED:** Previously applied migrations are immutable; add forward-only migrations. Updates must preserve existing records, identifiers, timestamps, and migration history.
- **APPROVED / PLANNED WORK:** Extend B4's existing schema validation and backup compatibility implementation before relying on backups for the new study schema. Current B4 support remains limited to the documented B3 prototype schema.
- **APPROVED:** Require compatible, verified backups stored outside application-managed storage before risky migrations or production upgrades. Retained internal recovery copies do not replace these external backups.
- **APPROVED:** Failed migrations or startup errors must preserve existing data and report an error rather than silently reset storage.
- **APPROVED:** Every future production release must pass regression and migration checks using representative disposable data.
- **LIMITATION:** No implementation can claim absolute immunity to data loss. Preserve recovery procedures and documented limitations, including the existing B4/B5/B6 risks and unresolved acceptance requirements.

### 4.2. Cross-platform implementation policy

- **APPROVED:** Maintain a shared codebase rather than independently building five unrelated applications.
- **RECOMMENDED:** Develop and validate on macOS first; check Windows and Android compatibility early; validate Linux and iOS before claiming official support. Packaging and signing remain platform-specific.
- **OPEN:** Minimum supported operating-system versions, mobile distribution channels, code-signing setup and exact release order.
- **NOT APPROVED:** A browser-only edition with weaker data persistence guarantees as a substitute for the native application.

### 4.3. Architectural verification gate

Before investing in the final dashboard, build a minimal Tauri prototype that:

1. Opens as an installed application without starting a server manually.
2. Creates and reads a native SQLite database outside browser-managed storage.
3. Preserves records after closing and reopening the app and restarting the device.
4. Exercises a schema migration and an application update without losing records.
5. Exports a consistent backup and successfully restores it; rejects invalid imports safely.
6. Verifies platform-specific persistence and build viability within the approved Stage 2 scope below; mobile acceptance is deferred to a future milestone.

**Status:** PARTIALLY VERIFIED on macOS ARM64 through B1/B2: packaged launch, native SQLite creation/read, and app quit/reopen persistence passed. Initial migration, repeated initialization, and failed-migration rollback were tested. B3 subsequently verified later schema migration safety and the native B2-to-B3 two-build upgrade. B4 subsequently verified consistent backup/restore and invalid-import rejection using isolated native data. B5 subsequently verified process interruption, deterministic filesystem failures, native historical upgrade lineage and fail-closed startup using disposable data; see the B5 evidence report. M1 device restart subsequently passed on macOS ARM64 (2026-10-25); other platforms and remaining acceptance requirements remain pending; the full gate has not passed.

**Historical, not binding:** The previous implementation was associated with the `backend365days` repository, `projects/backend365-mission-control`, branch `feature/mission-control-foundation`, Python 3.14.6 and FastAPI 0.141.1. The old FastAPI server, localhost workflow, HTML templates and former SQLite setup must not be reused automatically. This is not evidence of currently implemented functionality.

## 5. Former roadmap (historical; subject to redesign)

- **MVP 1.0:** Study areas, manual entry, persistence, basic history, total hours, streak, initial dashboard and recoverable deletion foundation.
- **MVP 1.1:** Visual history/calendar, heatmap, cumulative time chart and study-area statistics.
- **MVP 1.2:** Timer, tracked pauses, durable session state, recovery and review, multi-area/topic allocations.
- **MVP 1.5:** Goals.
- **2.0:** Advanced analytics.
- **2.2:** Mobile/responsive/PWA enhancements (historical plan; the new primary distribution target is installable Tauri applications).

The old **Backend365 / Mode 42** curriculum was an independent learning plan, **not** the mandated implementation sequence for this execution-focused rebuild.

## 6. Open product decisions

1. Today's in-progress streak display behavior.
2. Exact allocation, pause distribution, rounding and validation tolerances.
3. Reliable timer recovery and inactivity semantics.
4. Time-zone changes and daylight-saving edge cases.
5. Trash retention and permanent deletion rules.
6. Goals model and progress logic.
7. Navigation, final layout, color palette beyond yellow and exact heatmap scale.
8. Text constraints and more detailed note/topic modeling.
9. Compatibility of the selected Tauri SQL plugin on platforms beyond macOS.
10. Minimum supported OS versions, release order, installers and signing/distribution.
11. Future backup schema compatibility, recovery-retention UI and optional encryption beyond the B4 prototype.
12. Detailed study-feature scope and scheduling within the approved November 2, 2026 MVP target; implementation and acceptance remain pending. The approved sequence below supersedes updater-first delivery.

## 7. Immediate next steps

### Stage 1 — Minimum repository preparation

**Status:** IN PROGRESS — agent instructions and README completed; canonical specification and final `.gitignore` review pending.

**Objective:** Establish the minimum documentation and repository configuration required for consistent development.

1. **COMPLETED:** Populate `AGENTS.md` with coding agent instructions, including project conventions, documentation precedence, architectural constraints, and testing requirements.
2. **COMPLETED:** Create a concise `README.md` describing the project's purpose, approved platform, current development status, and repository organization.
3. Consolidate approved functional requirements into a canonical English specification under `docs/`, preserving historical references without treating obsolete technical decisions as binding.
4. Review and finalize `.gitignore` rules for the intended Tauri application structure, including generated files, build artifacts, local databases, and sensitive material.

**Completion criteria:** The repository contains sufficient documentation and configuration for a developer or coding agent to begin implementation without relying on previous conversations.

### Stage 2 — Application foundation and persistence validation

**Status:** IN PROGRESS — B1 completed and integrated into `main`; B2 native SQLite persistence, B3 safe schema migrations and B4 backup/restore completed and validated on macOS ARM64. The remaining architectural verification gate is pending; Stage 2 is not complete.

**Objective:** Establish a functional, durable, offline-first application foundation before implementing study features.

1. **COMPLETED (B1, macOS ARM64):** Initialize Tauri 2 within `app/` using React, TypeScript, and Vite. See verified scope below.
2. **COMPLETED (B2, macOS ARM64):** Implement native SQLite persistence outside browser-managed storage, with separate development and production databases.
3. **PARTIALLY VERIFIED:** Both bundles preserved records across closure/reopening; the user independently confirmed save/close/reopen persistence. M1 subsequently verified full device restart on macOS ARM64 (2026-10-25).
4. **COMPLETED (B3, macOS ARM64):** Versioned migrations preserve records; rollback/recovery, repeated initialization and incompatibility rejection passed automated tests. A native two-build B2-to-B3 upgrade preserved existing records and new writes through quit/reopen using isolated storage.
5. **COMPLETED (B4, macOS ARM64):** Consistent backup export, validated restart restoration, invalid-backup rejection and retained recovery data passed automated and isolated native validation.
6. **PARTIALLY VERIFIED (B5, macOS ARM64):** Integration/resilience tests and isolated native upgrade/process/startup checks passed. Native filesystem and SQLite write-failure handling (M3), eight packaged restore interruption boundaries (M4), and startup-error presentation and WebView storage independence (M6) passed within the documented scope. Full device reboot (M1) is PASS on macOS ARM64 (2026-10-25); real power loss (M2) and cross-platform/filesystem/distribution compatibility (M5) remain pending. Interruptions inside individual filesystem operations and midway through SQL plugin initialization remain unverified; see B5 below.
7. **PARTIALLY VERIFIED (B6):** Platform packaging overlays, Windows/Linux CI and early mobile static/toolchain inspection implemented. macOS final checks/native scenarios passed; see B6 report for remote results and exact blockers. Native Windows/Linux/mobile execution, reboot on other platforms, power-loss and distribution acceptance remain pending.
8. **ONGOING:** Record actual test results, known limitations, and remaining platform-specific requirements; B2 results are recorded below.

**Completion criteria:** The application launches without a manually managed server, operates offline, persists data reliably, and demonstrates safe migration and backup/restore behavior.

### Stage B1 — Minimal application foundation

**Status:** COMPLETED AND INTEGRATED INTO `main`; implemented and build-verified on macOS ARM64 (2026-10-24). Other platforms remain unverified.

- Generated using the official `create-tauri-app` React/TypeScript template, then removed demo commands, unused frontend assets, opener integration, and unnecessary runtime dependencies. Source icons remain template placeholders.
- Added a single native `main` window titled **Mission Control**, a static English welcome screen with a yellow accent, a local-assets content security policy, and no frontend native-command permissions.
- Added npm scripts for development, TypeScript checks, smoke tests, frontend builds, macOS packaging, and locked Rust checks/tests. Committed npm and Cargo lockfiles.
- Reviewed ignore rules: the existing root rules and generated native `.gitignore` exclude dependencies, frontend output, Cargo targets, and Tauri-generated schemas. No root ignore changes were needed.
- Updated README with prerequisites, exact commands, local bundle location, and the distinction between the CLI-managed development server and embedded production assets.

**Actual validation:**

- Environment: macOS 27.0.1, ARM64, Xcode Command Line Tools; Node 24.21.0, npm 11.19.0, Rust/Cargo 1.99.1. Rust, rustfmt, and Clippy were installed for this validation.
- `npm ci`: passed; audit reported zero vulnerabilities.
- `npm run check`: passed TypeScript and two smoke tests (server-rendered identity and native window/bundled-assets configuration).
- `npm run build`: passed; Vite production assets generated.
- `npm run rust:check`: passed with `--locked`.
- `npm run rust:test`: passed; the B1 Rust test harness contained **zero native tests** (B2 adds two).
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`: passed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --locked -- -D warnings`: passed.
- `npm run desktop:build -- -- --locked`: passed; generated `Mission Control.app` for macOS ARM64.
- `npm run desktop:dev`: Vite started and Cargo compiled/launched the native executable. The UI automation tool could not select the unbundled development process; its rendering was not visually verified.
- Packaged application: opened the `.app` with the Vite process stopped; accessibility inspection showed the Mission Control heading at `tauri://localhost`, and a screenshot confirmed the styled welcome screen. Closed the package after verification. This verifies embedded assets without a development server, not a disconnected-network test.

**Limitations:** Local development bundle only; distribution signing/notarization, custom icons, end-to-end WebView automation, Windows/Linux/mobile builds, and supported OS minimums remain unverified. B1 added no SQLite, schemas, migrations, backup/restore, durable user data, or study workflows; B2 supersedes the persistence limitation. The wider Stage 2 architectural gate remains open.

### Stage B2 — Native SQLite persistence prototype

**Status:** IMPLEMENTED AND VERIFIED on macOS ARM64 (2026-10-24); no production study schema or workflows.

- **APPROVED / IMPLEMENTED:** Official `tauri-plugin-sql` 2.6.0 with SQLite and `@tauri-apps/plugin-sql`, connection `sqlite:mission-control.db`. No external server, account, browser storage, or runtime network dependency.
- Rust registers native migration version 1 from `app/src-tauri/migrations/0001_persistence_test_records.sql`. The plugin applies transactional SQLx migrations on load and tracks versions in `_sqlx_migrations`. Schema creation stays outside React. No automatic reset, replacement, deletion, or seeding is implemented.
- Minimal table: UUID text primary key, required non-empty text with a database constraint, UTC creation timestamp generated by SQLite. Stable read order is creation timestamp then ID.
- Typed TypeScript module shares initialization, binds insert parameters, validates trimmed values, propagates errors and blocks concurrent writes. Temporary welcome-screen form loads records automatically and shows loading, empty, success and error states, with a load retry action. A read failure after a successful save retains the success message and clears the input to discourage a duplicate write.
- Only `sql:allow-load`, `sql:allow-select`, and `sql:allow-execute` are enabled for the main window. These plugin permissions permit those SQL operations generally; they are not table-level restrictions.
- **Physical paths verified by read-only SQLite inspection:** `/Users/maff137/Library/Application Support/com.missioncontrol.desktop/mission-control.db` (production) and `/Users/maff137/Library/Application Support/com.missioncontrol.desktop.dev/mission-control.db` (development). Portable form: `~/Library/Application Support/<identifier>/mission-control.db`. The plugin source resolves `app_config_dir()`. Both files are outside the repository and browser-managed storage.
- Production identifier remains `com.missioncontrol.desktop`. `npm run desktop:dev` explicitly uses `tauri.dev.conf.json` with identifier `com.missioncontrol.desktop.dev`; production builds use the base config. Direct `tauri dev` without the override would use production storage.

**Automated validation (actual results):**

- `npm run check`: TypeScript and 8 Vitest tests passed. Covers initialization sharing/retry, parameter binding, whitespace rejection, concurrent-write guard, write/read errors, identity and capability configuration.
- `npm run build`: passed, including TypeScript and Vite production build.
- `npm run rust:check`: passed with locked dependencies after downloading crates outside the network-restricted sandbox.
- `npm run rust:test`: 2 Rust tests passed. Real SQLite test creates a temporary file, applies the migration, writes, rejects invalid SQL/blank values, closes/reopens, reapplies the migrator and confirms exactly one unchanged record and timestamp. Another test confirms a failed migration rolls back partial schema changes while preserving existing data. Tests use the same SQLx migration engine and SQL file as the plugin; they do not exercise WebView IPC.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`: passed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --locked -- -D warnings` and the additional `--all-targets` check: passed.
- `npm run desktop:build -- -- --locked`: passed; production macOS ARM64 `.app` generated.
- `npm run tauri -- build --debug --config src-tauri/tauri.dev.conf.json --bundles app -- --locked`: passed; isolated development `.app` generated.

**Manual/native checks (actual results):**

- `npm run desktop:dev` compiled and launched with the dev override; Vite was stopped before bundled checks. The automation tool could not select the unbundled dev process, so an independently packaged debug bundle with the same override was used for UI checks.
- Both packaged applications automatically created their databases and initially displayed the empty state. Save was disabled for an empty input. Entered distinct synthetic test values, clicked Save, observed success and immediate record display in each environment.
- Actually quit each bundle with Command-Q, reopened it, and observed its own saved record loaded automatically with the same creation timestamp. Production and dev records remained isolated. No development server was running during either bundled check.
- **USER-CONFIRMED:** The user independently saved a record, closed the application, reopened it, and confirmed the record remained available. This was an application restart, not a full device restart; the user has not tested persistence across a device restart.
- Read-only inspection of both physical SQLite files confirmed the separate records, successful migration version 1 and `PRAGMA integrity_check = ok`. Synthetic records remain in native application storage and are not committed.

**Limitations / next work at the B2 milestone (upgrade validation superseded by B3 below):** Device reboot, application upgrades and later migration versions, backup/restore, disk corruption/permission fault injection in the UI, clearing WebView data, signed distribution and non-macOS platforms remain unverified. Error propagation is tested automatically, but native initialization failures have not been manually injected. The interface is temporary and duplicate text is allowed intentionally; the guard prevents simultaneous submissions, not repeated intentional saves. The broader Stage 2 gate remains open. Next foundation work should validate compatible upgrades, backups/restores and platform viability before production study workflows; no next-stage implementation is included in B2.

### Stage B3 — Safe database schema migrations

**Status:** COMPLETED AND VERIFIED on macOS ARM64 (2026-10-24), including automated acceptance scenarios and a genuine native two-build upgrade. The wider Stage 2 gate remains open.

- Migration 1 is byte-for-byte unchanged. Registered migration 2, `0002_add_test_record_note.sql`, adds `note TEXT NOT NULL DEFAULT ''`. Existing frontend inserts/reads remain compatible; database URL and production/development identifiers are unchanged. No study-session schema or backup/restore implementation was added.
- Forward-only policy: never change applied SQL, descriptions or versions; append numbered migrations and register them in Rust. The official SQL plugin and SQLx remain the sole engine/history (`_sqlx_migrations`); no reset, downgrade, replacement or secondary tracker was introduced.
- Safety finding: plugin 2.6.0 removes migration registration before a lazy load attempt, so retry after migration failure could skip it. Native SQL preload now runs migrations during plugin setup; an error aborts startup before dependent operations can run. Existing frontend errors still propagate. The preload configuration requires direct `serde_json` for `generate_context!`; the crate was already locked transitively, with no new package versions.
- Four Rust tests use unique temporary directories and real SQLite files, the same Rust registration/SQL and the plugin's SQLx conversion (Up/ReversibleUp, transactional). They cover: v1 records closed/reopened and upgraded with exact ID/value/timestamp/count preservation and default notes; fresh schema and versions 1/2; new inserts and persisted non-empty notes; repeated startup with unchanged migration history; failing v2 after ALTER/CREATE/UPDATE with full rollback and successful recovery after reopen; unknown future version and checksum rejection without data/history changes. Integrity and foreign-key checks plus primary-key/null/non-empty/note constraints are checked where applicable. No user database is accessed.

**Actual validation:**

- `npm run check`: passed TypeScript and **9 Vitest tests** (including initialization failure blocking dependent operations and preload configuration).
- `npm run build`: passed TypeScript/Vite.
- `npm run rust:check`: passed locked Cargo check.
- `npm run rust:test`: passed **4 Rust tests**; zero failures.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`: passed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings`: passed.
- `npm run desktop:build -- -- --locked`: passed; generated the macOS ARM64 `Mission Control.app` bundle. That production bundle was not launched; the follow-up below launched only isolated test bundles.
- Initial native compilation exposed the required `serde_json` direct dependency for plugin configuration; resolved and checks rerun successfully. Two npm invocations from the repository root failed because package.json is in `app/`; the actual checks above ran successfully from `app/`.

**Native two-build upgrade follow-up (2026-10-24): PASSED.**

- Environment: macOS 27.0.1 (26A434), ARM64. Detached worktrees `/private/tmp/mc-b3-native-b2` at `b75d65f` and `/private/tmp/mc-b3-native-b3` at `52dadb7`; application source, lockfiles and applied migrations remained unchanged. Each worktree added only a local JSON override: `{"identifier":"com.missioncontrol.desktop.b3upgrade20261024","productName":"Mission Control Upgrade Test"}`. Existing installed npm dependencies and the Cargo target cache were reused; each resulting bundle was copied to its own worktree before the next build.
- In each worktree's `app/`, the exact build command was `CARGO_TARGET_DIR=/Users/maff137/dev/projects/mission-control/app/src-tauri/target npm run tauri -- build --debug --config src-tauri/tauri.upgrade-test.conf.json --bundles app -- --locked`. **Both passed**, including frontend production builds. Bundles embedded assets at `tauri://localhost`; no development server was started.
- Verified both bundle `Info.plist` identifiers before launch. The dedicated test directory did not exist before B2 launch. Both bundles used `~/Library/Application Support/com.missioncontrol.desktop.b3upgrade20261024/mission-control.db`. Production and development apps were not launched; validation scripts never opened either existing database. `lsof -c mission-control -Fn` confirmed the B2/B3 executable paths and only the dedicated test database/WAL/SHM paths, including after B3 reopen.
- Launched B2 through native app automation. Saved three synthetic records through the form: alpha, beta (including an ASCII quote and a Unicode rocket), and gamma. Each save displayed success. Quit with Command-Q; confirmed PID 21786 no longer existed before inspecting the closed database. Snapshot contained exactly three records, migration history only `(1, success)`, and integrity `ok`.
- Launched the separate B3 bundle. All three original records and their original timestamps appeared automatically. Saved the new delta record through the interface and observed success and a fourth row. Quit completely (PID 22188 exited). Closed-database comparison verified all original IDs, values and creation timestamps unchanged, count increased from 3 to 4, successful migration versions 1/2, `note TEXT NOT NULL DEFAULT ''`, all four notes empty, and integrity `ok`.
- Reopened the same B3 bundle. All four values and original timestamps appeared automatically. Observed the isolated database handles again, quit fully (PID 22221 exited), and compared another closed snapshot: all four complete records and the full `_sqlx_migrations` history were identical, proving no migration reapplication. Integrity remained `ok`; `foreign_key_check` returned no rows.
- SQLite could not open the native test path directly in read-only mode (including an escalated attempt: `unable to open database file`). Verification therefore read file bytes into temporary snapshots **only after confirmed process exit and no WAL file**, then queried those copies. No schema/data writes or repairs were performed on the native database by the inspection scripts. Snapshots and test records remain outside Git; no user data was removed. The worktrees, test bundles and dedicated native test data were retained.
- Follow-up checks from the existing B3 branch's `app/`: `npm run check` **passed (9 Vitest tests)** and `npm run rust:test` **passed (4 Rust tests)**. Both isolated native build commands above passed. The follow-up changes only README/state documentation; prior B3 formatting, Clippy, Rust check and release packaging results remain recorded above, not claimed as rerun here. Documentation consistency review and Git whitespace checks passed before commit.

**Remaining work at the B3 milestone (backup/restore superseded by B4 below):** Native startup-failure UI verification, device restart, backup/restore, corruption/permissions fault injection, WebView-data clearing, signing/distribution and non-macOS/mobile viability remain open. This validates a debug-bundle application/schema transition, not a signed installer or updater delivery mechanism. Existing Stage 1/product questions remain unchanged.

### Stage B4 — Consistent backup export and safe restore

**Status:** COMPLETED AND VERIFIED for the macOS ARM64 prototype scope (2026-10-24). Stage 2 remains in progress; device restart and other platforms remain unverified.

**Implementation and compatibility decisions:**

- Native Rust service uses SQLite `VACUUM INTO` for consistent snapshots, including committed WAL data. Export sets snapshot-only `application_id = 0x4d434250` (MCBP) and `user_version = 1`. `.mcbackup` is plaintext standalone SQLite, without encryption/authentication. Original record fields and all SQLx history fields are preserved; physical page layout is not a compatibility guarantee. Applied migrations 1/2 remain unchanged.
- Format 1 accepts exactly the current version-2 schema (including exact schema SQL/constraints/indexes), successful migration versions 1/2, original descriptions and checksums. No unknown schema objects or backup-supplied migrations are accepted. Unsupported older/future formats/schemas require a future deliberate compatibility change.
- Validation operates on an isolated bounded copy: regular readable file, SQLite header/identity/version, exact schema before table queries, compatible history, record field types/size, integrity and foreign keys. Limits: 512 bytes–64 MiB per file; 1,000,000 combined UTF-8 bytes per record; 1 MiB SQLite value/row allocation limit, bounded SQL/columns/expressions/attachments, trusted schema disabled, defensive mode, 2 MiB page cache and 10-second VM progress budget. The budget does not interrupt stalled OS I/O. Existing oversized data is preserved, with export/restore rejected.
- Native save/open dialogs expose no frontend paths or general filesystem API. Restore rejects internal application storage, non-.mcbackup files and sources with SQLite WAL/SHM/journal companions, so active databases cannot be imported as standalone snapshots. Export never overwrites an existing destination. A temporary validated snapshot is atomically published by a no-clobber hard link, with file sync and Unix directory sync; filesystems without hard-link support fail safely. Application storage and its siblings under the app-config parent are protected destinations. A tracked temporary export is cleaned after failure or on next startup; final exports are retained.
- Restore selection is copied into a private app-managed `restore-<uuid>/incoming.db` and completely validated. UI confirmation says all current records are replaced, without merging. Cancellation removes only the unconfirmed draft. A durable no-clobber pending marker commits the decision; native reads/writes and backup actions are then blocked until full quit/reopen.
- A first Tauri plugin acquires a process-held file lock and processes recovery before SQL plugin initialization. Startup validates the candidate again, takes a verified SQLite `original.db` snapshot, syncs an installation marker, retains the raw original DB/WAL/SHM/journal together, then installs a same-filesystem candidate. SQL plugin preload/migrations must succeed before pending state is cleared in application setup. Interruption after installation starts triggers original-snapshot rollback on the next launch. Failed/invalid recovery never silently resets records; unrecoverable I/O/recovery errors stop startup and preserve files. Invalid staged data is rejected before replacement and reported in the UI with the original DB retained.
- All persistence SQL now lives behind narrow Rust read/insert commands using the existing plugin pool. A shared mutex serializes operations and enforces the pending-restore write prohibition. Generic frontend SQL permissions were removed (including select, which could otherwise expose writable SQL); the capability has no plugin filesystem/dialog permissions. React retains only UI/typed IPC state and the existing concurrent-save guard.
- New direct Rust dependencies: dialog plugin 2.8.3 (rfd 0.16.1 and filesystem plugin 2.7.1 transitively); existing locked serde, uuid, tokio and libsqlite3-sys become direct dependencies, and SQLx becomes a runtime dependency. libsqlite3-sys configures limits through the SQLx exclusive handle guard. JavaScript replaces the SQL plugin package with `@tauri-apps/api`. No server, cloud account, automatic sync, study workflows or schema migration was added.

**Actual automated verification:**

- `npm run check`: passed TypeScript and **13 Vitest tests**, covering persistence IPC/error/concurrency behavior, application/configuration, backup controls, actionable native errors and lifecycle registration.
- `npm run build`: passed TypeScript/Vite.
- `npm run rust:check`: passed locked Cargo check.
- `npm run rust:test`: passed **16 Rust tests** (4 unchanged migration tests and 12 B4 tests). B4 covers exact multiple-record IDs/values/notes/timestamps and full history; empty export/restore; restore/reopen and repeated restore; cancellation boundaries; WAL snapshot consistency; malformed/unrelated/truncated/corrupt/oversized inputs; future/checksum/schema/constraint rejection; failed file operations and staged tampering without live changes; interrupted export cleanup; recovery before/during/after raw moves; interruption before plugin acknowledgement; corrupt recovery snapshot preservation; exclusive storage lock, active/internal source rejection and development/production isolation. Unique temporary databases only; no user database paths are used.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`: passed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings`: passed.
- During development, the native command future initially failed the Send requirement; fixed by running borrowed SQLx backup operations on blocking workers while keeping the main thread free. A defensive-size test exposed that integrity checks alone do not read every payload; explicit record type/byte validation was added and the test then passed. Some npm/patch commands initially used the repository root instead of `app/`; rerun in the correct directory. Initial registry access failed in the sandbox; dependencies/lockfiles were resolved with approved network access. No failed check is counted as passed.

**Native macOS ARM64 verification (2026-10-24): PASSED using disposable data only.**

- `npm run desktop:build -- -- --locked` passed and produced the production `.app`; it was not launched. The separate native command `npm run tauri -- build --debug --config '{"identifier":"com.missioncontrol.desktop.b4test20261024","productName":"Mission Control Backup Test"}' --bundles app -- --locked` passed. Its bundle was copied to `/private/tmp/mc-b4-native-20261024/Mission Control Backup Test.app`. Verified the identifier in `Info.plist` and absence of the dedicated storage before launch.
- Native automation launched only that test bundle with embedded assets at `tauri://localhost`, without a development server. Saved alpha and beta synthetic records through the UI, including a literal ASCII quote and Unicode rocket via paste. Both saves succeeded. Cancelled a native save dialog, then exported `b4-records.mcbackup` into the temporary test directory. The UI reported validated export success. Read-only inspection confirmed format/application markers, two records, complete versions 1/2 history and integrity `ok`.
- Added gamma after export. Cancelled the native open dialog, then selected the backup and cancelled the explicit replacement confirmation. All three records remained and controls were reenabled. Selected again and explicitly confirmed replacement. The input, Save and backup controls stayed disabled, and the UI instructed full quit/reopen. Quit with Command-Q; verified the initial process PID 25873 had exited (the sandbox denied the first `ps`, then approved read-only `ps` confirmed no such process). The closed database had three records and the durable pending marker; no WAL remained.
- Reopened the same test bundle. Startup restored only alpha/beta with their original timestamps and displayed success with original recovery retention. Visual inspection confirmed the backup section remained consistent with the existing styling. Quit completely. Closed-file snapshots confirmed **all four record fields and every migration-history field exactly matched the exported backup**, while `restore-ea3a67f5-7a74-4f24-b658-656273f1acb2/original.db` exactly matched the pre-restore three-record database. Integrity/FK checks passed for all compared snapshots and `restore.pending` was cleared.
- Reopened again and observed the same two restored records automatically. Selected a deliberately malformed temporary `.mcbackup`; the UI rejected it with the size/format guidance, retained both records and reenabled the controls. Quit the test application. Test bundle, exports, closed inspection snapshots and native recovery files were retained outside Git.
- After the final source review, all requested checks were rerun successfully (13 Vitest, 16 Rust, frontend/native checks, formatting, all-target Clippy and locked desktop packaging). The isolated debug bundle was rebuilt into the test directory's `final/` subdirectory and launched again. Native export to `b4-final.mcbackup`, validated restore selection and confirmation cancellation passed; the final export exactly matched the earlier restored records/history. The test app was quit again.
- `lsof -c mission-control -Fn` showed only the dedicated test bundle and its isolated native database/WAL/SHM/lock paths; production/development applications were not launched and their databases were never opened. Native-file inspection copied bytes into temporary snapshots only after full quit/no WAL; queries opened those snapshots read-only. This demonstrates app restart durability and isolated native backup/restore, not full device reboot, OS power loss or signed distribution.

**Limitations and remaining validation at the B4 milestone (native UI faults, startup presentation and WebView clearing superseded by B5 below):** Full device reboot/power-loss durability, non-macOS/mobile platforms, unsupported destination filesystems, permission/disk-full faults in the native UI, startup-failure presentation, signing/distribution and WebView-data clearing remain unverified. Directory fsync is implemented on Unix only; Windows directory crash durability remains unverified. The advisory process lock coordinates B4 instances, not old versions/external SQLite tools. Recovery snapshots and raw originals are retained without pruning or a recovery-management UI. Backups are unauthenticated plaintext; accept only trusted files and protect exports. Stage 2 and the broader architectural gate remain open.

### Stage B5 — Integration and resilience testing

**Status:** PARTIALLY VERIFIED on macOS ARM64 (updated 2026-10-25). M1 device reboot passed; automated scope passed; remaining acceptance/environmental scenarios prevent declaring B5 or Stage 2 fully complete. Full evidence, exact commands, per-scenario expected/actual results and native reproduction procedures: [B5 resilience report](B5-RESILIENCE-REPORT.md).

- Six new scenario tests in `resilience_tests.rs` reuse B4 temporary fixtures and complete record/history snapshots, without duplicating B3/B4's existing migration and corruption matrices. One ignored subprocess helper is explicitly invoked by the harness for five real process-death boundaries: open WAL/uncommitted transaction, unconfirmed draft, durable confirmation, installed backup before acknowledgement, and storage lock. The harness owns/reaps every terminated process and verifies independent storage concurrently.
- Exact field/history preservation, integrity/FK checks, two further reopens per boundary, distinct selected-backup/original data, retained recovery snapshots and draft cleanup passed. Added retry after original snapshot/candidate preparation, complete unpublished-export cleanup, unsupported backup formats/schema and malformed recovery markers.
- Deterministic SQLite FULL via max_page_count, READONLY and BUSY errors passed without filling the disk. Actual permission-denied creation/copy/export in a temporary Unix directory passed with successful retry. One-shot thread-local `cfg(test)` hooks exercise eight copy, marker-write/flush/publication, directory-sync, export-publication and install-rename failures. A published marker still blocks operations when directory sync reports failure. Hooks are absent from production builds; no dependencies, applied migrations, schema or product behavior changed. No production defect was exposed.
- Native detached builds at B2 `b75d65f`, B3 `52dadb7`, B4 `dce2b2f` and current ran sequentially under the fresh dedicated `com.missioncontrol.desktop.b5integration20261024` identifier. Complete snapshots proved B2 fields/v1 history preserved on B3, empty new notes and successful versions 1/2, identical B3/B4 records/history and preservation in current. The current bundle saved a third synthetic record, exported a consistent snapshot, was positively identified via executable/storage paths and killed, then reopened with exact pre-kill records/history. Repeated reopens, including a final rebuilt isolated bundle, passed. No normal development/production database was opened.
- Native concurrent startup failed explicitly with application_busy. An invalid recovery marker caused nonzero startup with invalid_recovery, a byte-identical database and retained marker. Moving aside only that synthetic marker allowed successful reopen; complete records/history remained unchanged. This verifies fail-closed startup, not user-facing startup error UX.
- **Actual validation:** `npm run check` passed (13 Vitest); `npm run build`, locked `npm run rust:check`, `npm run rust:test` (22 passed, zero failed, one child helper ignored in parent), rustfmt, locked all-target Clippy with warnings denied, and locked release `.app` packaging passed. B5-only default-parallel and serial runs passed (six scenarios each); historical and current isolated debug bundle builds passed. See the report for corrected initial harness/inspection failures; none are counted as successful checks.
- **Pending at the initial B5 milestone (native items superseded by the follow-up below):** Full device reboot and real power-loss testing; actual native UI disk/permission-fault presentation; every interior restore interruption in packaged startup; startup-error UX and WebView-data clearing; other platforms/filesystems and signed updater delivery. SIGKILL is not equivalent to power loss. Historical validation is a forward build lineage, not independent direct upgrades or reverse migration. B4 advisory-lock, Unix-only directory durability and recovery-retention limits remain unchanged. Synthetic bundles/data/worktrees were retained outside Git.

### B5 native acceptance follow-up — M3, M4 and M6

**Status (2026-10-24):** Native macOS M3/M6 scenarios below passed; eight packaged M4 interruption boundaries passed. At this historical follow-up, independent M1 evidence, M2 and M5 were pending; no reboot result was supplied then. The M1 follow-up below supersedes that pending result. B5 remains PARTIALLY VERIFIED and Stage 2 remains IN PROGRESS.

- **Git base:** Fresh fetch left `origin/main` at B4 `dce2b2f`; B5 was not integrated. Clean branch `test/b5-native-acceptance` was safely created from B5 `e2004f5`. No merge, PR or historical rewrite was performed.
- **Implementation:** Native initialization now runs recovery, dynamically initializes the same SQL plugin/migrations, and acknowledges startup before creating the normal WebView. Errors open a dedicated bundled error page containing preservation/permissions/capacity/reopen guidance and serialized details inserted through `textContent`. No reset, new migration, dependency, generic IPC permission, remote asset or browser-storage fallback was added. SQL operation errors now include write-recovery guidance instead of only suggesting a backup.
- **M3 / actual OS faults:** A disposable 64 MiB APFS image independently returned ENOSPC. Packaged export showed OS error 28 without publishing its final destination; retry after freeing only the synthetic filler succeeded. Directory mode 0500 and source mode 000 independently returned EACCES; packaged export, restore-workspace creation and backup-copy failures displayed guidance, with successful subsequent export and staging/cancellation. Native storage mode 000 produced an actionable startup error; mode 0755 restored normal launch. For database failure, the APFS image was mounted at the exact dedicated `app_config_dir`, with device IDs, absence of symlinks and the sole test process's SQLite handles verified before filling. Save returned real SQLITE_FULL/code 13; freeing the filler and retrying produced exactly one new row. All six preexisting complete records and migration rows remained exact; integrity/FK passed. The image was detached and only explicit synthetic application files restored to ordinary isolated storage.
- **M4 / real packaged terminations:** `app/scripts/native-acceptance.py` ran eight boundaries twice across isolated debug builds: before/after original snapshot, after original DB displacement, before/after incoming rename, before SQL initialization, before acknowledgement and after acknowledgement. The final run additionally asserted exact pending-marker content, displaced main/candidate state, prior recovery copies, two stable reopens, full records/history and integrity/FK. Test fixtures construct durable confirmation; process deaths occur inside packaged startup, not a reconstructed service state. Kernel-syscall interiors, every individual sidecar move and interruption midway through SQL setup remain untested.
- **Instrumentation safety:** Opt-in `native-acceptance` has no dependency or IPC/network surface, requires `com.missioncontrol.acceptance.*` with matching non-symlink native directory, and bounds waits to 60 seconds. Ordinary builds contain no acceptance controls; release compilation with the feature is explicitly rejected. The controller requires absent storage and terminates/reaps only its own `Popen` children.
- **M6:** Packaged error-window presentation passed for invalid marker, invalid recovery snapshot, real inaccessible storage and corrupted SQL database. Invalid-marker/snapshot/permission cases preserved the valid database SHA-256; corrupt input also remained byte-identical. Restoring only synthetic fixtures permitted normal launch. Three additional recognizable records were saved through the UI; only the dedicated `~/Library/WebKit/<identifier>` and `~/Library/Caches/<identifier>` directories were quarantined after quit. HTTPStorages for that identifier was absent. Reopen showed all six records with original timestamps; closed copied-database comparison proved exact records and complete migration history, integrity `ok`, no FK violations. Safari/global/browser-profile data was untouched.
- **Validation:** All seven requested checks passed on final code: `npm run check` (13 Vitest), `npm run build`, locked Rust check/test (24 passed, zero failed, one subprocess helper ignored), rustfmt, locked all-target Clippy with warnings denied, and locked release `.app` build. Additional feature-enabled all-target Clippy and both eight-boundary native matrix runs passed. See the [B5 report](B5-RESILIENCE-REPORT.md#native-acceptance-follow-up--m3-m4-and-m6) for commands, retained evidence locations, per-scenario results, release-isolation verification and corrected preparation failures.
- **Safety/limitations:** Only fresh dedicated identifiers `com.missioncontrol.acceptance.b5native20261024a` and `...b` were launched. Production/development databases were never accessed. No host filesystem filling, OS restart, power-loss experiment, security-setting change, global cache clearing or cross-platform test occurred. APFS image and synthetic artifacts remain outside Git; test permissions were restored and the image detached. At that follow-up, M1/M2/M5 remained pending (M1 superseded below), alongside filesystem/controller durability, distribution and existing retention/platform limitations.

### B5 device reboot acceptance follow-up — M1

**Status (2026-10-25): M1 PASS on macOS ARM64.** B5 remains **PARTIALLY VERIFIED** and Stage 2 remains **IN PROGRESS**.

- The user verified an isolated disposable `Mission Control M1 Reboot.app` (`com.missioncontrol.acceptance.m1reboot20261025`, `mission-control.db`): successful build/package, two synthetic records verified in SQLite, full macOS restart, successful reopen and both original timestamps visible. The reported final script result was `M1 ACCEPTANCE: PASS`, with identical dumps, complete records and SQLx migration history, integrity `ok`, no foreign-key violations and changed boot timestamps.
- Local read-only inspection of `~/Documents/MissionControl-B5-M1/` confirmed identical `before.sql`/`after.sql`, both records and migration rows, both integrity files containing `ok`, and different boot timestamps. Script output and separate foreign-key output were not present; those results rely on the supplied user observations. Evidence and disposable data were not modified. See the [M1 evidence and scope](B5-RESILIENCE-REPORT.md#device-reboot-acceptance-follow-up--m1).
- M2 real power loss remains **NOT TESTED**; remaining M5 native cross-platform/filesystem and signed distribution acceptance stays pending. This ordinary reboot does not establish power-loss durability or close the architectural gate. This documentation-only update synchronizes the report and README; application tests were not rerun.

### Stage B6 — Cross-platform compatibility and Stage 2 acceptance

**Status (2026-10-25): PARTIALLY VERIFIED; Stage 2 IN PROGRESS.** Work starts from clean fetched `origin/main` at `6fd860c`. B6 adds automatic Windows NSIS (offline WebView2) and Linux Debian packaging overlays, removes the forced macOS-only npm build argument, and adds a Windows/Linux compatibility workflow with read-only repository permissions, locked application dependencies, no artifact upload and no release/signing operations. Regression coverage checks platform packaging invariants and exact backup/restore/reopen through Unicode paths with no-clobber publication.

Initial unchanged-code macOS checks passed: 13 Vitest, 24 Rust tests with one ignored child helper, frontend build, Rust check, formatting and all-target Clippy. Final-code macOS validation passed all seven checks (14 Vitest, 25 Rust passed, one helper ignored), release/debug packaging, eight packaged interruption boundaries and isolated native UI export/restore/reopen, invalid import, WebView quarantine and fail-closed startup. Native exFAT export failed safely without partial publication; retry on APFS matched exact data/history.

Remote CI passed on Linux x64 (14 Vitest, 25 Rust with one ignored helper, checks and unsigned .deb packaging). Windows x64 passed 14 Vitest, 24 Rust with one ignored helper, checks and release compilation; initial packaging failed at the Microsoft offline WebView2 download with Peer disconnected, then a failed-job rerun passed all checks and produced the unsigned NSIS installer with offline WebView2. Run 40225328483 attempt 2 succeeded on implementation commit 8249e8c. These are service-test/build results, not native packaged-app execution or installed persistence. Detailed run/commit evidence is in [B6 compatibility report](B6-COMPATIBILITY-REPORT.md). Full Xcode, Android SDK/NDK and mobile Rust targets are unavailable. At the B6 verification run, M1/M2 were NOT TESTED; the subsequent M1 follow-up records PASS on macOS ARM64. M2 remains NOT TESTED; M5 remains PARTIALLY VERIFIED. No acceptance policy, supported-platform commitment, applied migration, identifier, production security or study behavior changes.

### B6 hosted packaged-runtime automation follow-up (2026-10-25)

- **IMPLEMENTED:** Existing Windows 2022/Ubuntu 24.04 B6 matrix extended with one identity-isolated release package build per host, NSIS/Debian installation and actual installed-process SQLite initialization, schema/checksum checks, exact records/history after termination/relaunch and native startup restore with distinct retained-original/reopen comparisons. No application behavior, migration, application dependency, package policy or security permission changed. CI-only display/keyboard/screenshot tools and synthetic evidence upload support the new checks.
- **SCOPE:** Controller-created records/confirmation markers are native startup fixtures, not UI acceptance. Linux attempts actual keyboard Save/native-dialog export under Xvfb/DBus. Windows records interactive/session/input-desktop probes and attempts Save/export through a PowerShell driver that verifies foreground ownership before sending input; unavailable desktop/foreground access is BLOCKED. Existing Rust tests remain service tests. No Vite runs; absent port 1420 is checked each launch, while Linux uses a separate network namespace and Windows disconnected-network acceptance remains untested.
- **AUDIT/SAFETY:** Fresh acceptance identifiers only; existing storage refused; only owned children terminated/reaped. Per-scenario PASS/FAIL/BLOCKED JSON/job summaries and 14-day synthetic reports/screenshots/logs (no DB/packages) include commit/run/package SHA-256/path evidence. Incomplete prerequisites start BLOCKED; runtime failures fail the job. Manual remaining Windows/Linux disposable VM procedure is in the [B6 report](B6-COMPATIBILITY-REPORT.md#hosted-packaged-runtime-acceptance-follow-up).
- **VERIFIED LOCALLY:** Four Python oracle/safety tests passed; these do not establish hosted packaged acceptance. One npm command from repository root failed with missing package.json; the rerun from `app/` passed all seven checks: 14 Vitest, frontend build, locked Cargo check/test (25 passed, one ignored helper), rustfmt, all-target Clippy with warnings denied and release macOS packaging. No normal bundle was launched. Bash syntax and diff whitespace checks passed. Hosted evidence below comes from executed jobs and inspected artifacts, never from compilation alone.
- **FIRST HOSTED ATTEMPT:** Run 40232356700 (`155d2bd`) Linux checks/package/installation succeeded, but the controller failed before app launch on an overly strict `./usr/bin` listing guard. Corrected to accept the observed exact `usr/bin` path too; not counted as runtime acceptance. Reports now preserve per-scenario BLOCKED states and continue independent restore checks after visible UI failure; identity traversal is rejected. Four controller tests passed locally after changes. Windows first-attempt NSIS installation, SQLite initialization, exact termination/relaunch and fixture-based native startup restore passed. Its desktop probe returned UserInteractive=True, session=2, input desktop=True; that attempt did not run UI automation. A PowerShell driver now attempts UI Save/export without claiming the desktop probe as UI acceptance. Final hosted results pending.
- **SECOND HOSTED ATTEMPT:** Run 40233384302 (`2dafa05`) Linux installed-release startup/schema, exact process termination/relaunch, actual WebView Save, native startup fixture restore/retained original/reopen and disconnected-network operation passed. Native UI export produced no file (FAIL); records/history remained exact and service/startup checks continued. GTK driver now targets Save’s Name field instead of Ctrl+L navigation and captures the dialog/outcome. Windows also passed actual UI Save/native-dialog export; that application-generated export was restored exactly by native startup, including retained original and another reopen. Its desktop was usable (session 2). Final Linux export rerun pending.
- **THIRD HOSTED LINUX ATTEMPT:** Run 40234673785 (`6d067e1`) repeated all previously passing native/data/offline/UI-Save checks, but export still produced no file. New screenshots show the GTK Save dialog open with unchanged default filename after keyboard input; the Xvfb driver now explicitly identifies/focuses the single other visible window owned by the app before typing. No export pass or application storage failure is inferred; final focus correction awaits hosted verification.
- **FINAL LINUX HOSTED RESULT:** Run 40235828890 (`20b1dbe`) Linux job PASS. All implemented installed-release/native schema/persistence/UI Save/UI export/startup restore/retained-original/reopen and disconnected-network checks passed. Native-dialog app export was restored exactly (three complete records/history); distinct original retained four exact rows. Screenshot shows validated-export success. Independently compared artifact JSON matches fixture/reopen, UI-save/restore/reopen and before-restore/retained-original snapshots. Overall acceptance remains BLOCKED for explicit remaining criteria.
- **FINAL WINDOWS / COMPLETE CI RESULT:** Run 40235828890 completed SUCCESS on implementation `20b1dbe`; both platform jobs passed. Windows repeated native installation/schema/persistence, actual UI Save/native-dialog export, exact startup restore of the app-generated export, retained four-row original and three-row stable reopen. Interactive/session/input-desktop = True/2/True; no blanket unavailable-desktop claim. Actual log counts: 14 Vitest and four controller tests per platform; 25 Linux / 24 Windows Rust tests, one ignored helper each. All locked check/build/format/Clippy steps passed. Downloaded final JSON independently confirms complete record/history equality; screenshots inspected. Windows network remained enabled. Final follow-up changes documentation only; application/configuration/migrations/lockfiles/capabilities/CSP diffs against main are empty.
- **UNCHANGED GATE:** B5/B6 PARTIALLY VERIFIED; Stage 2 IN PROGRESS. UI restore/cancel/invalid import, Windows disconnected-network/clean installation, reboot outside existing macOS M1, real power loss, remaining filesystem/mobile/signed update/distribution requirements remain pending. README.md and this state record are synchronized with the B6 implementation and scope.

### Approved Stage 2 acceptance scope (2026-10-25)

**APPROVED:** macOS ARM64 is the primary foundation acceptance platform. This decision supersedes the broader closure policy and the OPEN scope proposal in the historical [B6 recommendation](B6-COMPATIBILITY-REPORT.md#stage-2-recommendation); it does not change recorded evidence or establish new test results. B5/B6 remain **PARTIALLY VERIFIED** and Stage 2 remains **IN PROGRESS**.

- **Required before Stage 2 closure:** Resolve M2 and all essential macOS data-integrity acceptance gaps. Retain the existing requirements for offline packaged operation, native storage, exact record/migration preservation across reopen/reboot and compatible updates, safe backup/restore and invalid-input rejection, fault recovery, actionable startup errors and WebView independence. M1 is PASS only within the recorded macOS ARM64 scope. Review remaining intra-operation and mid-SQL-initialization gaps explicitly; do not infer coverage from adjacent passing boundaries.
- **M2 remains OPEN / NOT TESTED:** The deferral of advanced physical power-loss testing is not an M2 pass or waiver. Before closure, document the bounded macOS durability guarantees and resolve M2 through an authorized isolated test with recorded filesystem/cache/flush assumptions and exact recovery comparisons, or an explicit approved scope/risk decision addressing the remaining M2 evidence gap. SIGKILL, ordinary reboot and successful fsync alone do not resolve it. Any physical durability guarantee requires corresponding evidence.
- **Windows/Linux coverage:** Preserve the distinction between CI checks/builds and verified installed native runtime. B6 records Windows Server 2022 x64 and Ubuntu 24.04 x64 installation, startup, exact termination/relaunch preservation, UI Save/export and startup restore; restore confirmation was controller-created. Linux disconnected-network runtime passed; Windows disconnected-network operation, UI restore selection/confirmation/cancel/invalid import, reboot, filesystem-specific behavior and broader OS coverage remain gaps. These do not block the macOS foundation gate, and no general Windows/Linux support is declared. A future desktop acceptance milestone must resolve the relevant native gaps and define supported OS/filesystem versions before claiming support.
- **Deferred to explicit future milestones:** Android/iOS acceptance; public distribution; paid Apple signing/notarization and broader signed installer acceptance; advanced physical power-loss/storage-controller testing across hardware and filesystems. Signed updater preservation is now required for the MVP under the approved delivery plan and Stage U below. These remain requirements for their respective future claims, not completed or abandoned work. Mobile acceptance requires toolchains, device/provider backup integration, native persistence/recovery and offline verification. Distribution acceptance requires supported minimum versions, clean installation prerequisites, platform security checks and data-preserving signed updates. Advanced power-loss acceptance requires authorized disposable hardware/storage, documented cache/flush guarantees, controlled cuts at durability boundaries and exact post-recovery records/history/originals/integrity checks.
- **Residual risks:** Physical-flush/controller durability remains unproven; Windows directory sync is unverified; hard-link-dependent filesystems and external tools ignoring advisory locks remain limitations. Retained recovery copies lack pruning UI and backups remain unauthenticated plaintext. Keep these limits visible in future acceptance decisions; narrowing the foundation platform does not remove data-preservation obligations or authorize destructive tests.

**Documentation-only verification:** Only README.md and docs/STATE.md are changed. Existing B5/B6 evidence and historical results are preserved; no application tests or builds were run. Validation is limited to `git diff --check` and review of the documentation diff.

### Platform rationale documentation (2026-10-25)

- Added README's **Why Tauri and Web Technologies?** section to explain the existing approved installable application direction, shared React/TypeScript/Vite UI, Tauri 2 native integration and offline native SQLite persistence. The primary PWA alternative remains rejected because browser-managed storage does not meet the approved native storage and reliability requirements; no architectural decision changed.
- Distinguished planned macOS/Windows/Linux/Android/iOS support from recorded macOS ARM64 foundation and scoped Windows x64/Linux x64 installed-runtime evidence. Android/iOS native compatibility and minimum OS versions remain unverified/open; no general platform support is declared. Existing B5/B6 partial verification and Stage 2 status are unchanged.
- Documentation-only validation: reviewed the README/state diff against existing platform decisions and B6 evidence; `git diff --check` passed. Application tests and builds were not rerun. Only README.md and docs/STATE.md were updated and synchronized.

### Approved MVP delivery plan (2026-10-25)

**APPROVED:** Target MVP delivery on **November 2, 2026**, with **macOS ARM64** as the primary development platform. Prioritize a usable personal study workflow before implementing the updater. The in-app updater remains a **mandatory MVP release requirement**; optional automatic checks do not make updater delivery optional.

**PLANNED implementation sequence — targets, not guarantees of completion:**

1. **Data safety:** Prepare the real study schema, forward-only migrations, production/development isolation, backup compatibility, and preservation tests.
2. **Core usability (Stage C):** Implement configurable study areas, manual sessions with date, duration, optional times, topic and notes, basic history, editing, daily totals, and the approved 10-minute effective-study streak calculation.
3. **Personal deployment:** Establish a stable, installable macOS build for real daily use, isolated from experimental builds and destructive tests. Personal use may begin before the MVP only once study persistence, migrations, and backup compatibility are sufficiently validated under the data protection policy.
4. **Updater (Stage U):** Implement and validate the approved official Tauri updater architecture, optional automatic checks, manual checks, cryptographic signatures, and GitHub Releases.
5. **MVP acceptance:** Verify data preservation, migration safety, backup/recovery, offline operation, and actual A → B → C application updates before release; retain Stage U's complete release gate.

Defer advanced features when necessary to protect the deadline and stability without changing approved product behavior or waiving acceptance requirements. This sequencing decision does not close Stage 2, resolve M2, or change B5/B6 evidence: Stage 2 remains **IN PROGRESS** and B5/B6 remain **PARTIALLY VERIFIED**.

**Documentation-only update:** README.md and docs/STATE.md are synchronized with this approved roadmap and policy. Existing implementation and test evidence is preserved; no application builds or tests are required or rerun for this task. Documentation validation uses complete diff review and unstaged/staged whitespace checks.

### Stage U — macOS In-App Updates and Private Alpha Distribution

**Decision date:** 2026-10-25  
**Status:** APPROVED MVP REQUIREMENT / PLANNED IMPLEMENTATION / NOT VERIFIED\
**Primary platform:** macOS ARM64 (Apple Silicon)  
**Target:** Initial private alpha distribution to a small group of users.

#### U.1. Objective and delivery priority

Implement and validate a reliable, user-friendly in-app update mechanism after data safety, core study usability, and stable personal deployment. Updater implementation is not a prerequisite for beginning Stage C.

The approved November 2, 2026 MVP target includes this mandatory updater and its release acceptance gate. Personal use may begin earlier only under the validated data-safety conditions above; it does not establish MVP acceptance or private alpha updater readiness.

**Superseded planning history:** The earlier October 25 updater-first plan allowed a persistence-only alpha and deferred Stage C until updater acceptance. The approved MVP delivery plan now replaces that priority and dependency, without changing the architecture, security policy, or release gate below.

Stage 2 remains IN PROGRESS; B5/B6 remain PARTIALLY VERIFIED. M2 and existing acceptance requirements remain unchanged.

#### U.2. Approved product behavior

- **APPROVED:** The application must check for updates automatically when launched, provided the user has not disabled automatic checks.
- **APPROVED:** Users must be able to disable automatic update checks through a persistent setting.
- **APPROVED:** Provide a manual "Check for Updates" action.
- **APPROVED:** Notify users when a newer version is available and request confirmation before installation.
- **APPROVED:** Installation must not require users to open GitHub, execute terminal commands, or manually reinstall the application.
- **APPROVED:** Normal application functionality must remain available offline.
- **APPROVED:** Network failures, unavailable releases, and invalid update metadata must produce recoverable errors rather than block application startup.
- **APPROVED:** Application updates must preserve native SQLite data, migration history, backup compatibility, and the existing production application identifier.
- **APPROVED:** Prevent concurrent update installations and avoid replacing the application during critical database, backup, or restore operations.
- **APPROVED:** The initial private macOS alpha will not require a paid Apple Developer Program membership or Apple notarization.

**Acceptance limitation:** Avoiding additional macOS Gatekeeper authorization prompts is a desired user experience, not a guaranteed property of unsigned or non-notarized distribution. It must be tested on actual installed macOS applications.

#### U.3. Recommended technical architecture

Use the official Tauri 2 updater plugin rather than implementing a custom downloader or executable replacement mechanism.

Recommended components:

- Official Tauri updater plugin for version checks, download, signature verification, and installation.
- GitHub Releases as the distribution endpoint, using the official supported static update manifest format.
- GitHub Actions and `tauri-apps/tauri-action` for reproducible macOS ARM64 release builds.
- Cryptographically signed updater artifacts, using the public key embedded in the application.
- Ad-hoc macOS code signing for the initial private alpha.
- A small, isolated update service responsible for update state and operations.
- A separate React component for update notifications, progress, user confirmation, and error messages.
- Persistent native application settings for the automatic-check preference; browser-managed storage must not be authoritative.

Avoid custom update servers, mandatory cloud accounts, unnecessary frameworks, independent download managers, and application-specific installer logic.

Keep the updater independent of study-domain models, session calculations, and database schema details.

**Suggested module locations (not mandatory):**

- `app/src/services/updater.ts`
- `app/src/components/UpdateDialog.tsx`
- `app/src-tauri/tauri.conf.json`
- `app/src-tauri/src/lib.rs`
- `.github/workflows/release.yml`
- `docs/UPDATE-PROCESS.md`

Final module boundaries and plugin permissions must be validated against the installed Tauri version and current application architecture.

#### U.4. Signing, versioning, and security

- Preserve the production bundle identifier `com.missioncontrol.desktop`.
- Use monotonically increasing semantic versions.
- Keep release versions consistent across relevant application manifests and package metadata.
- Keep the updater signing private key outside the repository.
- Store CI signing credentials only in appropriately protected GitHub Actions secrets.
- Maintain a separate, securely protected recovery copy of the updater signing private key.
- Never commit private keys, passwords, signing credentials, or generated secrets.
- Verify signatures before installing downloaded update artifacts.
- Reject invalid, missing, or incompatible update signatures.
- Preserve signing-key continuity across releases. Key loss or incompatible rotation may prevent existing installations from updating.
- Limit release automation permissions and review external GitHub Actions dependencies.
- Do not publish releases automatically on ordinary commits or pushes to `main`.
- Require passing validation and explicit approval before making an update available to users.
- Keep experimental release testing isolated from the production update channel.

The public GitHub Releases endpoint should expose only approved application updates. Draft and prerelease behavior must be accounted for when selecting the production update endpoint.

#### U.5. Data integrity and recovery requirements

The updater must not directly modify or reset SQLite study data.

Preserve existing transactional migration and B4 backup/restore safeguards, subject to the mandatory personal-use data protection policy above. Extend B4 schema validation and compatibility before relying on backups for the new study schema.

Before distributing releases containing schema changes:

1. Add new forward-only migrations without modifying previously applied migrations.
2. Test upgrades from the previous distributed application version.
3. Verify exact preservation of existing records, identifiers, timestamps, and migration history.
4. Update the B4 backup schema validation and compatibility policy when migrations change the database schema.
5. Test backup export, supported historical backup restoration, and startup recovery.
6. Reject incompatible databases safely without silently resetting or replacing them.

Update verification must confirm that release configuration preserves the existing production storage path and application identifier; exercise upgrade scenarios with representative disposable data in isolated acceptance identities, never personal study data.

Network or download failures must leave the current installed version usable. Interrupted installation and startup failures require separate native acceptance tests; automatic rollback must not be assumed.

Document a recovery procedure for reinstalling a compatible application version without deleting native user data. Database downgrades must not be assumed safe.

#### U.6. Planned implementation milestones and dependencies

**Dependency:** Schedule U1–U4 after the data-safety, core-usability, and personal-deployment priorities above, before MVP acceptance. These milestones are PLANNED, not implemented or verified.

**Historical planning allocation (superseded):** The updater-first plan allocated 22% of the October 25 Codex allowance and 100% on October 26 (U1: 22%; U2: 30%; U3: 25%; U4: 45%). Those allocations were estimates, not completion evidence or the current schedule.

**U1 — Updater foundation**

- Inspect the current repository, lockfiles, native startup sequence, existing tests, and supported updater plugin versions.
- Integrate the official updater and required minimal permissions.
- Configure updater signing and macOS ad-hoc application signing.
- Prepare native updater artifacts and the public-key configuration.
- Isolate updater operations from persistence and backup services.
- Validate the foundation without changing study-domain behavior.

**U2 — Release automation**

- Implement a dedicated macOS ARM64 GitHub Actions release workflow.
- Reuse existing frontend and Rust checks.
- Build reproducible signed updater artifacts and update metadata.
- Keep release publication under explicit approval.
- Verify versions, signatures, target architecture, manifest content, and download URLs.
- Use isolated update testing before publishing to the production update channel.

**U3 — User experience**

- Implement automatic checks on startup.
- Add a persistent setting to disable automatic checks.
- Implement manual update checks.
- Display update availability, download progress, installation confirmation, success, and actionable errors.
- Prevent simultaneous installations and unsafe installation during critical data operations.
- Handle offline operation and network failures without blocking normal use.

**U4 — Native acceptance and recovery**

- Test real installed-application updates A → B → C.
- Verify persistence after each update and restart.
- Test invalid signatures, missing releases, failed downloads, and unavailable connectivity.
- Verify persistent update preferences.
- Validate compatibility with backup, restore, and startup recovery.
- Test macOS security behavior using actual application bundles installed in Applications.
- Review the complete diff, run applicable checks, and document actual evidence.
- Provide a tested recovery procedure and identify any remaining distribution blockers.

The implementation may require reallocating effort when dependencies or acceptance tests reveal blockers. Do not skip security or persistence validation to meet the target date.

#### U.7. Release acceptance gate

**Status: NOT TESTED.**

The updater is considered ready for private alpha distribution only after the following acceptance criteria pass:

1. Version A is manually installed and successfully launched on macOS ARM64.
2. Version A discovers, downloads, verifies, and installs Version B.
3. Version B discovers, downloads, verifies, and installs Version C.
4. Both updates preserve complete native SQLite records and migration history.
5. The application reopens and remains functional after each update.
6. Invalid or incompatible update signatures are rejected.
7. Offline operation and unavailable release endpoints do not block application startup.
8. Disabled automatic checks remain disabled after restarting the application.
9. Backup export, supported restoration, and database integrity checks remain functional.
10. Update installation does not interrupt critical data operations.
11. The actual Gatekeeper behavior is documented and the intended nontechnical-user update flow is verified on the target macOS installation.
12. A recovery procedure is documented and tested using isolated, disposable application data.

Use disposable acceptance identities and storage for destructive or upgrade tests. Never run acceptance tests against normal development or production user databases.

A successful build or unit test alone does not establish native updater acceptance.

If any essential requirement fails or remains unverified, keep Stage U IN PROGRESS and do not describe the update flow as guaranteed or ready for general distribution.

#### U.8. Distribution limitations and future compatibility

The initial distribution is limited to private macOS ARM64 testing.

Paid Apple Developer membership and Apple notarization are intentionally excluded from this private alpha. Ad-hoc signing does not guarantee that macOS will suppress Gatekeeper warnings during installation or later updates.

The initial application may be installed manually by the project owner, including user-authorized macOS security approval.

No permanent exemption from future macOS security checks is assumed.

Windows, Linux, Android, iOS, public distribution, paid Apple signing, notarization, and broader OS compatibility remain future milestones.

Future distribution requirements must be revisited before claiming unrestricted installation or automatic-update compatibility across supported devices.

#### U.9. Maintenance policy

Keep the update implementation minimal, modular, and independent from the study feature roadmap.

Every future release must:

- Preserve the updater trust configuration and production application identity.
- Use a compatible signing key and validated update manifest.
- Increment the version correctly.
- Pass regression and migration checks using representative disposable data before every production release, alongside relevant automated checks.
- Validate migrations and backup compatibility when changing stored data.
- Preserve a documented recovery path.
- Be published only after explicit release approval.

Maintain `docs/UPDATE-PROCESS.md` as the operational reference for signing-key recovery, release creation, acceptance checks, troubleshooting, and emergency maintenance.

Synchronize `README.md` and `docs/STATE.md` after implementation tasks as required by `AGENTS.md`.

**Official references:**

- https://v2.tauri.app/plugin/updater/
- https://v2.tauri.app/distribute/pipelines/github/
- https://v2.tauri.app/distribute/sign/macos/

### Next development milestone

**Stage C — First Functional Study Release**

**PLANNED next milestone:** Prepare the real study schema, forward-only migrations, isolated environments, updated B4 schema/backup compatibility, and preservation tests before relying on study records for personal use. Then implement Stage C core usability; Stage U is no longer a prerequisite. No new study functionality is implemented or verified by this plan.

The first functional workflow is:

**Create a study area → record a study session → persist the record → display history → calculate daily study totals and streaks.**

A calendar day qualifies after at least 10 minutes of effective study, accumulated across eligible sessions and study areas.

Timer recovery, multi-area allocations, calendar views, activity heatmaps, cumulative charts, advanced dashboards, and goals will be implemented incrementally in subsequent releases.

After core usability and stable personal deployment, implement Stage U and pass its release gate before the MVP. Once implemented, keep the updater operational and regression-tested throughout later development.

### Documentation synchronization policy (2026-10-24)

**APPROVED / DOCUMENTED:** `AGENTS.md` now requires meaningful updates to both this document and `README.md` after every development task, before the final commit and push, with a final consistency review and explicit confirmation in the completion report. This includes documentation-only work and applies even without a user request.

**Verification for this documentation task:** Reviewed all three documents against the B2 implementation, npm scripts, Rust migration/tests, and development configuration. Preserved the recorded B2 results (8 Vitest and 2 Rust tests passed); application tests and manual persistence checks were not rerun for this documentation-only change. The user-confirmed app restart check is recorded separately from implementation validation; device restart remains unverified. Documentation checks: `git diff --check` passed, the complete diff was reviewed for consistency and documentation-only scope, and `AGENTS.md` was verified below the 8,000-character limit.

**Maintenance rule:** Update both `docs/STATE.md` and `README.md` after every development task, before the final commit and push. Update each stage as work progresses. Mark tasks as completed only after implementation and appropriate validation. Never equate `APPROVED`, `RECOMMENDED`, or `PLANNED` with `IMPLEMENTED` or `TESTED`.
