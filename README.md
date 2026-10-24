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

The proposed technical stack includes:

| Component | Technology | Status |
|---|---|---|
| Application framework | Tauri 2 | Approved |
| Frontend | React, TypeScript, Vite | Implemented; macOS foundation validated |
| Native integration | Rust | Minimal Tauri entry point implemented |
| Local database | SQLite | Recommended |

Technical recommendations will be finalized through implementation and validation.

## Development Status

**Current phase: Project foundation and architectural validation.**

The project is being rebuilt from scratch while preserving previously approved functional requirements.

Current priorities include establishing the application foundation, validating native persistent storage, testing schema migrations, and verifying backup and restore capabilities.

No production-ready release or installation package is currently available.

## Repository Structure

```text
mission-control/
├── app/          # React/Vite frontend, smoke tests, and src-tauri/ native shell
├── docs/         # Project state and specifications
├── tests/        # Reserved for future repository-level tests
├── AGENTS.md     # Coding agent instructions
├── .gitignore
└── README.md
```

The structure will evolve as implementation progresses.

## Getting Started

Stage B1 provides a minimal native window with a static Mission Control welcome screen. Study features and persistence are not implemented.

### Prerequisites

- Node.js 24 LTS and npm (validated with Node 24.21.0 and npm 11.19.0).
- Stable Rust with Cargo; `rustup component add rustfmt clippy` for native quality checks (validated with Rust 1.99.1).
- Platform dependencies from the [official Tauri prerequisites](https://v2.tauri.app/start/prerequisites/). On macOS desktop, install Xcode Command Line Tools with `xcode-select --install`. Full Xcode is needed for iOS work, which is outside B1.

### Development and validation

Run from `app/`:

```sh
npm ci
npm run desktop:dev
```

The Tauri CLI starts and stops the Vite development server automatically. Development uses localhost; the packaged application embeds the built frontend and does not require Node, a user-managed server, an account, or network access during normal use.

```sh
npm run check          # TypeScript and two smoke tests
npm run build          # Frontend production assets
npm run rust:check     # Cargo check using Cargo.lock
npm run rust:test      # Rust test harness (no native unit tests yet)
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked -- -D warnings
npm run desktop:build -- -- --locked  # macOS .app bundle
```

`npm run dev` and `npm run preview` are frontend-only helpers. `npm run tauri -- <command>` exposes the native CLI. The desktop build command currently targets the validated macOS `.app` format; other platform packaging remains unverified. Lockfiles are committed for reproducible dependency resolution.

The macOS bundle is generated at `app/src-tauri/target/release/bundle/macos/Mission Control.app`. Open it through Finder, or copy it to Applications for local use. This is a local development bundle, not a signed/notarized distribution release. Template icons remain placeholders.

Smoke tests check server-rendered application identity and the bundled-assets/window configuration. Native launch and packaged rendering require a desktop smoke check; these tests are not end-to-end WebView automation. See [Project State](docs/STATE.md) for actual results and limitations.

## Documentation

For current decisions, approved requirements, development priorities, and unresolved technical questions, refer to:

- [Project State](docs/STATE.md)

The project state document is the authoritative reference for the current development phase.

---

**Mission Control — Track your time. Build consistency. Understand your progress.**
