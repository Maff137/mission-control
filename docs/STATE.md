# Mission Control — Project State

**Last updated:** 2026-10-24\
**Phase:** Stage B2 native SQLite persistence implemented and validated on macOS; wider architectural gate pending.
**Repository:** `Maff137/mission-control`  
**Historical reference:** `definicoes-mission-control.md` (October 20, 2026 compilation).

## Status conventions

- **APPROVED:** Product behavior or technical direction explicitly accepted for the rebuild.
- **RECOMMENDED:** Engineering proposal selected as the preferred direction, pending implementation and/or validation.
- **PLANNED:** Intended future work, not necessarily included in the first release.
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
- **OPEN:** Exact supported OS versions, installers, final study schema and future migrations, backup format, cross-platform test matrix and release process.
- **VERIFIED (2026-10-24):** Stage B2 adds native SQLite test-record persistence, transactional initialization, isolated dev storage, and packaged-app quit/reopen verification in both environments. See B2 results below; the wider foundation gate is still open.
- **HISTORICAL B1 VALIDATION (2026-10-24):** Stage B1 initializes the official Tauri 2 React/TypeScript/Vite template in `app/`, reduced to a static welcome screen and minimal Rust entry point. macOS ARM64 frontend checks, native checks, and `.app` packaging passed. The packaged window was visually inspected with the development server stopped. No persistence or study features are implemented.
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
| Backup and restore | **APPROVED DIRECTION** — portable, user-controlled export/import with validation and consistent snapshots. File format, retention and automatic backup policy are **OPEN**. |
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
- **OPEN:** Other OS storage paths, final study schema, transactional backup method, import conflict strategy, backup encryption and sensitive-data protection.
- **LIMITATION:** Native application storage is independent of browser eviction but not immune to disk failure, corruption, user deletion, OS cleanup or mobile app uninstallation. Backups remain necessary.

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
6. Verifies platform-specific persistence and build viability, including an early mobile test.

**Status:** These checks are acceptance criteria for the foundation, **not completed tests**.

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
11. Backup file format, validation rules, restoration semantics and optional encryption.
12. Priorities for the first functional release after the persistence prototype passes.

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

**Status:** IN PROGRESS — B1 application shell implemented; persistence and the remaining architectural gate are not completed.

**Objective:** Establish a functional, durable, offline-first application foundation before implementing study features.

1. **COMPLETED (B1, macOS ARM64):** Initialize Tauri 2 within `app/` using React, TypeScript, and Vite. See verified scope below.
2. Implement native SQLite persistence outside browser-managed storage.
3. Verify data durability across application closure, reopening, and device restarts.
4. Implement and test database schema migrations without losing existing records.
5. Implement consistent backup export and validated restoration, including safe rejection of invalid backups.
6. Perform initial cross-platform compatibility checks, including early mobile feasibility testing.
7. Record actual test results, known limitations, and remaining platform-specific requirements.

**Completion criteria:** The application launches without a manually managed server, operates offline, persists data reliably, and demonstrates safe migration and backup/restore behavior.

### Stage B1 — Minimal application foundation

**Status:** IMPLEMENTED AND BUILD-VERIFIED on macOS ARM64 (2026-10-24). Other platforms remain unverified.

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
- `npm run rust:test`: passed; the Rust test harness currently contains **zero native tests**.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`: passed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --locked -- -D warnings`: passed.
- `npm run desktop:build -- -- --locked`: passed; generated `Mission Control.app` for macOS ARM64.
- `npm run desktop:dev`: Vite started and Cargo compiled/launched the native executable. The UI automation tool could not select the unbundled development process; its rendering was not visually verified.
- Packaged application: opened the `.app` with the Vite process stopped; accessibility inspection showed the Mission Control heading at `tauri://localhost`, and a screenshot confirmed the styled welcome screen. Closed the package after verification. This verifies embedded assets without a development server, not a disconnected-network test.

**Limitations:** Local development bundle only; distribution signing/notarization, custom icons, end-to-end WebView automation, Windows/Linux/mobile builds, and supported OS minimums remain unverified. No SQLite, schemas, migrations, backup/restore, durable user data, or study workflows were added. The wider Stage 2 architectural gate remains open.

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
- Read-only inspection of both physical SQLite files confirmed the separate records, successful migration version 1 and `PRAGMA integrity_check = ok`. Synthetic records remain in native application storage and are not committed.

**Limitations / next work:** Device reboot, application upgrades and later migration versions, backup/restore, disk corruption/permission fault injection in the UI, clearing WebView data, signed distribution and non-macOS platforms remain unverified. Error propagation is tested automatically, but native initialization failures have not been manually injected. The interface is temporary and duplicate text is allowed intentionally; the guard prevents simultaneous submissions, not repeated intentional saves. The broader Stage 2 gate remains open. Next foundation work should validate compatible upgrades, backups/restores and platform viability before production study workflows; no next-stage implementation is included in B2.

### Next development milestone

After Stage 2 passes its acceptance criteria, implement the first tested functional workflow:

**Create a study area → record a study session → persist the record → display history → calculate study totals and streaks.**

Subsequent features, including timer recovery, multi-area allocations, calendar views, heatmaps, and dashboards, should be implemented incrementally according to approved requirements.

**Maintenance rule:** Update each stage as work progresses. Mark tasks as completed only after implementation and appropriate validation. Never equate `APPROVED`, `RECOMMENDED`, or `PLANNED` with `IMPLEMENTED` or `TESTED`.
