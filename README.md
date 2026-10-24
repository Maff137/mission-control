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
| Frontend | React, TypeScript, Vite | Recommended |
| Native integration | Rust | Recommended |
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
├── app/          # Application source (pending initialization)
├── docs/         # Project state and specifications
├── tests/        # Test suite (pending implementation)
├── AGENTS.md     # Coding agent instructions
├── .gitignore
└── README.md
```

The structure will evolve as implementation progresses.

## Getting Started

Development environment setup, build instructions, and installation procedures will be documented once the application foundation is established.

## Documentation

For current decisions, approved requirements, development priorities, and unresolved technical questions, refer to:

- [Project State](docs/STATE.md)

The project state document is the authoritative reference for the current development phase.

---

**Mission Control — Track your time. Build consistency. Understand your progress.**