# Mission Control — Project State

**Last updated:** 2026-10-20  
**Phase:** Rebuild from scratch; functional decisions preserved; technical architecture under review.  
**Repository:** `Maff137/mission-control`  
**Historical reference:** `definicoes-mission-control.md` (October 20, 2026 compilation).

## Status conventions

- **APPROVED:** Behavior explicitly retained for the rebuilt product.
- **PLANNED:** Historical roadmap direction, not necessarily included in the first release.
- **OPEN:** Decision not finalized.
- **HISTORICAL:** Old implementation detail, not a requirement for the rebuild.

**Guiding rule:** Preserve approved product behavior, but do not automatically reuse the old architecture, technologies, codebase, roadmap, or teaching methodology. A specification is not proof of implementation.

## 1. Current state

- **APPROVED:** Rebuild Mission Control from scratch because the previous technical foundation was unsatisfactory.
- **APPROVED:** Focus on delivering a professional, stable, maintainable product rather than using the project as a programming lesson.
- **APPROVED:** Keep project code and documentation entirely in English; conversations may be in Brazilian Portuguese.
- **OPEN:** Web vs. desktop vs. hybrid app; target OSs and packaging; programming languages, frameworks, database, deployment and architectural boundaries.
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
- **APPROVED:** Persist timer state and timestamps so browser/app closure, suspension or interruption does not silently erase session history.
- **APPROVED:** After **24 consecutive hours** left unresolved in `running`/`paused`, interrupt the timer and move the session to `needs_review`.
- **APPROVED:** Functional states include `running`, `paused`, `finished`, `needs_review` and `discarded`. Discarding an unfinished timer is different from deleting a saved session.
- **APPROVED:** `needs_review` sessions contribute **nothing** to streak, totals, line graphs, heatmap or study-area analytics until resolved.
- **APPROVED BEHAVIOR / IMPLEMENTATION OPEN:** Recovery flow should let the user accept calculated time, accept time until the last trustworthy activity, manually edit duration, or discard. Any inferred suggestion must be editable and confirmed.
- **OPEN:** Technical persistence, inactivity detection, last trustworthy activity, exact 24-hour transition mechanism and automated suggestion logic.

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
- **APPROVED:** Use the user's time zone (initially inferred from device, configurable by user), never blindly the server time zone, for calendar and streak calculations.
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
- **PLANNED:** Full trash-management UI, with storage mechanism to be selected later.
- **OPEN:** Trash retention, permanent purge and audit behavior.

## 3. Planned features — not requirements of the first release

- **PLANNED:** Goals and objectives (potential properties include title, area, description, deadline, status and progress); definitive data model and calculation rules remain open.
- **PLANNED:** Additional weekly/monthly analytics, comparisons across areas, study times and trends.
- **PLANNED:** Better mobile/multi-device experience where technically appropriate, subject to platform selection.
- **PLANNED/HISTORICAL:** Possible private alpha, self-hosted use or SaaS later; no obligation to implement accounts, payments, cloud services or multi-user support now.
- **NOT APPROVED AS MVP REQUIREMENTS:** AI, notifications, payments or additional gamification.

## 4. Technical decisions reopened

| Area | Status |
| --- | --- |
| Platform | **OPEN** — web, native desktop, hybrid, installation and packaging |
| Languages/frameworks | **OPEN** — old Python/FastAPI choice is historical only |
| Storage | **OPEN** — old SQLite choice is historical only |
| UI stack | **OPEN** — old HTML/CSS/JavaScript choice is historical only |
| Local-first/offline behavior | **OPEN** — earlier design goal must be reconsidered and precisely defined |
| Authentication, sync, hosting | **OPEN** — do not introduce without justified product needs |
| Architecture, APIs, schemas, migrations, tests | **OPEN** — design against approved functional behavior |

**Historical, not binding:** The previous implementation was associated with the `backend365days` repository, `projects/backend365-mission-control`, branch `feature/mission-control-foundation`, Python 3.14.6 and FastAPI 0.141.1. This is not evidence of currently implemented functionality.

## 5. Former roadmap (historical; subject to redesign)

- **MVP 1.0:** Study areas, manual entry, persistence, basic history, total hours, streak, initial dashboard and recoverable deletion foundation.
- **MVP 1.1:** Visual history/calendar, heatmap, cumulative time chart and study-area statistics.
- **MVP 1.2:** Timer, tracked pauses, durable session state, recovery and review, multi-area/topic allocations.
- **MVP 1.5:** Goals.
- **2.0:** Advanced analytics.
- **2.2:** Mobile/responsive/PWA enhancements.

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
9. Priorities for the first functional release after architecture selection.

## 7. Immediate next steps

1. Treat this document as the quick decision record. Use the historical definition document for detailed examples and provenance.
2. Choose the target user experience and distribution model, then compare platforms and technologies accordingly.
3. Record the chosen architecture and its reasons before building new foundations.
4. Define a small vertical implementation slice: **create a study area -> log a session -> persist it -> view history -> compute totals/streak**, with tests.
5. After every delivery, separately record **implemented functionality**, **tests actually run**, and **remaining work**.

**Maintenance rule:** On each approved product decision, update the appropriate section and status. Never equate `APPROVED` or `PLANNED` with `IMPLEMENTED`.
