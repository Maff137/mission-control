# B5 — Integration and resilience evidence

Date: 2026-10-24. Status: **PARTIALLY VERIFIED**. Automated macOS scope and the native checks below passed. Device reboot, power loss, other platforms and the remaining environmental acceptance checks are not verified. Stage 2 remains open.

## Environment and isolation

- macOS 27.0.1 (26A434), ARM64; Node 24.21.0, npm 11.19.0, Rust 1.99.1; locked Tauri 2.12.5, SQL plugin 2.6.0, SQLx 0.8.6, libsqlite3-sys 0.30.1. Existing lockfiles/dependencies and migrations are unchanged.
- Base: freshly fetched `origin/main` at B4 `dce2b2f`; clean workspace; branch `test/resilience-integration-b5`.
- Rust tests own UUID temporary directories (`mc-b4-<uuid>`), databases and exports. Native tests use only `com.missioncontrol.desktop.b5integration20261024`, verified absent before first launch, and temporary bundles. Normal production/development applications and databases were not opened.
- Test-only, one-shot, thread-local filesystem faults are behind `#[cfg(test)]`; they do not exist in packaged builds. No runtime environment variable, command or feature enables them in production. The subprocess worker is also test-only, checks a canonical temporary root, and is invoked with the exact test name. The parent waits for readiness (20-second deadline), kills only its own `Child`, and reaps it even on failure.
- `SIGKILL` tests process death while the OS/filesystem continue running. They do not simulate device power loss, reboot, storage-controller failure or lost OS write caches.

## Commands and test index

Run from `app/`:

```sh
npm run check
npm run build
npm run rust:check
npm run rust:test
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
npm run desktop:build -- -- --locked
cargo test --manifest-path src-tauri/Cargo.toml --locked b5_ -- --nocapture
cargo test --manifest-path src-tauri/Cargo.toml --locked b5_ -- --nocapture --test-threads=1
```

All seven required checks **PASS**: 13 Vitest tests; 22 Rust tests, zero failed, one helper ignored in the parent run. B5 adds six scenario tests. Both B5-only commands **PASS** (six tests, one helper ignored); the process scenario invokes that helper in five separately killed children. The helper is not an additional acceptance test. Default parallel execution and serial execution both passed, including reruns after strengthening isolation/recovery assertions. Release `.app` packaging passed; the normal production bundle was built but never launched.

The following aliases identify exact Rust test filters; run any using `cargo test --manifest-path src-tauri/Cargo.toml --locked <filter> -- --nocapture`:

| Alias | Test filter |
|---|---|
| P | `b5_process_termination_releases_lock_and_recovers_each_boundary` |
| W | `b5_sqlite_full_readonly_and_busy_preserve_complete_data` |
| F | `b5_injected_filesystem_failures_preserve_data_and_recover` |
| D | `b5_permission_denied_export_and_copy_allow_retry` |
| R | `b5_retry_after_original_snapshot_and_completed_export_cleanup` |
| V | `b5_unsupported_backup_formats_and_invalid_markers_fail_closed` |

Tests reuse B4's real Store, temporary fixtures and complete structured snapshots. Every snapshot includes all record fields (`id`, `value`, `created_at`, `note`) and all migration fields (`version`, `description`, `installed_on`, `success`, `checksum`, `execution_time`). B5 verifies `integrity_check = ok` and empty `foreign_key_check` for reopened live databases and relevant retained originals. Byte layout is not used to assert logical equality except the native failed-startup preservation check.

## Scenario results

Environment for every automated row: the isolated Rust/SQLite environment above. Expected and actual outcomes are shown together where identical. Each listed command/test actually ran in B5, including unchanged B3/B4 tests via the full suite.

| ID / purpose | Command / fault or boundary | Expected result | Actual observed result / status |
|---|---|---|---|
| A1 graceful close/reopen and idempotent initialization | Full suite: `existing_database_upgrade_and_repeated_initialization`, `snapshot_restore_exact_records_history_and_reopen`; P reopens twice after every phase | Complete records/history stable, no seeds/reset | Exact snapshots match; **PASS** |
| A2 committed WAL and uncommitted transaction on process death | P, `wal`: child commits a distinct record in WAL, starts an IMMEDIATE transaction, changes notes and inserts another row, then parent kills child | Retain committed row; discard uncommitted changes; release lock | WAL existed and was nonempty before kill; committed fields/history retained, uncommitted row/notes absent; two reopens stable; **PASS** |
| B1 actual permission-denied creation/copy/export | D: temporary directory mode 0500; assert OS PermissionDenied with a probe | Explicit failures, no export published, existing records safe, retry succeeds | All assertions and retry passed; permissions restored by guard; **PASS** on this non-root Unix environment |
| B2 deterministic database-full write | W: cap `max_page_count` to current page count, insert 1 MiB text | SQLITE_FULL without filling host disk, no partial record | SQLite code 13, exact original data/history and integrity; **PASS** |
| B3 readonly / database owned by another writer | W: readonly connection; independent zero-timeout writer while first holds IMMEDIATE transaction | SQLITE_READONLY / SQLITE_BUSY, no records changed after rollback | Codes 8 / 5; full snapshot unchanged; **PASS** |
| B4 copy/write/sync/publication failures | F: one-shot StorageFull before copy, after copy before flush, before marker write/flush/hard-link, and before export publication | Explicit errors, no live replacement; unconfirmed partial copies cleaned; unpublished export absent | All eight fault sites consumed; originals unchanged before restart; recovery/integrity passed; **PASS** |
| B5 marker publication followed by directory-sync failure | F: `directory_sync` during confirmation | Return error but keep write guard if marker already exists; restart honors published confirmation | `pending` true, `ready()` rejects; next startup restores selected backup and clears marker after acknowledgement; **PASS** |
| B6 failed replacement rename after originals moved | F: `install_rename`; unchanged B4 `replacement_file_failure_rolls_back_and_allows_retry` forces original-move failure | Roll back safely, retain original, permit later recovery/retry | Distinct original records/history recovered; B4 retry passed; **PASS** |
| C1 export interrupted before snapshot / before publication | Full suite: `interrupted_export_cleanup_preserves_published_backup`; R creates complete validated but unpublished snapshot; D fails workspace creation | Tracked partial/complete temporary snapshots cleaned; published files untouched | Export markers/workspaces cleaned, backup contents unchanged; **PASS** (constructed states, not killed during VACUUM) |
| C2 before restore confirmation | P, `draft`: kill actual child after staging | No replacement; only unconfirmed draft cleaned | Original exact, no restore directories left; **PASS** |
| C3 after durable confirmation | P, `confirmed`: kill child after confirm | Next startup installs backup; retains distinct original until acknowledgement | Backup exact, original snapshot exact, writes guarded, marker cleared after acknowledgement; **PASS** |
| C4 original recovery snapshot created, before installing marker | R: durable original plus complete candidate; reopen | Retry safely, retain prior original snapshot | Selected backup installed; original and previous-original snapshots exact; **PASS** (constructed interruption state) |
| C5 during replacement / partially moved DB and stale sidecars | Full suite: `recovery_handles_each_interrupted_move_phase`, F | Valid original recovered, sidecars displaced together, originals retained | All three B4 move phases and injected install rename passed; **PASS** (constructed states/fault injection) |
| C6 installed before SQL plugin acknowledgement | P, `installed`: actual child startup installs selected backup then is killed before `finish_startup` | Next startup restores distinct original, retains recovery snapshot | Original exact, recovery status reported, retained original exact; **PASS** (service boundary, not a killed WebView callback) |
| C7 corrupted recovery / incoming and malformed markers | Full suite: `invalid_recovery_snapshot_never_displaces_existing_database`, `failed_exports_and_staged_tampering_are_safe`; V | Invalid original blocks startup with marker retained; invalid incoming rejected before replacement | Existing complete records/history safe; invalid markers unchanged; **PASS** |
| D1 forward schema upgrade, rollback, repeated launches | Full suite: four existing migration tests; native procedure below | Preserve original fields and history, default note, safe failed migration rollback | Exact metadata/record assertions and native lineage passed; **PASS** |
| D2 unknown migrations/checksums | Full suite: `future_version_and_checksum_mismatch_fail_without_changes`, `invalid_backups_never_change_live_data` | Reject without downgrade/reset | Explicit errors, records/history unchanged; **PASS** |
| D3 unsupported backup formats/schema | V: user_version 0/2, wrong application_id, drop note; B4 future/checksum/constraint/trigger cases | Reject under existing exact B4 format-1/schema-2 policy | Rejected, full live snapshot unchanged; **PASS** |
| E1 exclusive process storage lock and independent environment | P in all five phases; B4 storage-isolation test | Other process cannot acquire same Store; independent storage remains usable | application_busy, independent Store initialized while child alive and remained exact; lock reacquired after death; **PASS** |
| E2 pending restore operation guard / concurrency | F/P plus B4 tests and 13 frontend tests | Pending restore rejects operations; mutex and frontend write guard serialize access | Store guard, process lock, database BUSY and frontend concurrency assertions passed; **PASS** within tested boundaries; no stress/IPC linearizability claim |

No production defect was exposed by these tests. Production behavior, schema, migration registration and dependencies remain unchanged. During harness development the full-disk test initially inserted a BLOB, triggering the existing CHECK constraint (code 275) before reaching disk allocation. It was corrected to bound text and then observed code 13. A root/app working-directory mistake and an initial test syntax error were corrected; only successful subsequent executions are counted above.

## Native macOS procedure and observations

Native records were synthetic and remained outside Git. Detached worktrees used the exact historical commits; no historical source, lockfile or applied migration was modified. Each historical `app/` ran `npm ci --offline` successfully (zero reported vulnerabilities; npm warned that optional fsevents install scripts were not approved/run).

Reproduction (choose a **fresh** identifier for a new run; never reuse this already-upgraded storage with an old build):

```sh
# From the repository root; choose fresh, nonexistent paths when repeating.
git worktree add --detach /private/tmp/mc-b5-b2 b75d65f
git worktree add --detach /private/tmp/mc-b5-b3 52dadb7
git worktree add --detach /private/tmp/mc-b5-b4 dce2b2f
```

In each historical worktree's `app/`, run `npm ci --offline`, then the following command, substituting B2/B3/B4 in the product name. `CARGO_TARGET_DIR` was set to the main checkout's `app/src-tauri/target` to reuse the build cache; builds ran sequentially. Copy each completed bundle into a separate temporary directory before another build.

```sh
npm run tauri -- build --debug --config '{"identifier":"com.missioncontrol.desktop.b5integration20261024","productName":"Mission Control B5 B2"}' --bundles app -- --locked
```

The current checkout ran the same command with product names `Mission Control B5 Current` and, after the test hooks were added, `Mission Control B5 Final`. All five debug builds **PASS**. Verify `Contents/Info.plist` identifiers before launch. Launch only the temporary copied bundles, in order, with all earlier processes fully exited. Use native UI automation or Finder; no development server runs.

| ID | Exact procedure / fault | Expected | Actual / status |
|---|---|---|---|
| N1 historical upgrade lineage | B2: save two distinct records through UI, including quote/Unicode, Command-Q. Copy closed DB to temporary snapshot. Launch B3, observe both rows, Command-Q and snapshot. Repeat with B4. Launch current and save third row. | Original IDs/values/timestamps intact; v1 metadata unchanged; v2 added once; later upgrades keep history | Two B2 records preserved exactly, notes empty; complete B2 migration row retained in B3, versions 1/2 successful; B3 and B4 snapshots identical. Current preserved both records and all B3/B4 history; **PASS** for B2→B3→B4→current lineage, not separate direct-upgrade installations or signed updater delivery |
| N2 packaged forced termination | Current: export `pre-kill.mcbackup` via native dialog; verify UI export success. Identify sole process by exact temporary `Contents/MacOS/mission-control` path with `ps -axo pid=,comm=`; verify dedicated DB with `lsof -p <pid> -Fn`; send `SIGKILL` only to that PID; reopen same bundle. | All three committed records and metadata survive | PID 32732 positively identified and killed; native UI reopened with all three original timestamps. After Command-Q, all record/history fields exactly equaled the pre-kill export; integrity/FK passed; **PASS** |
| N3 concurrent native startup | While recovered current is running, `subprocess.run([exact_temporary_bundle_executable], capture_output=True, timeout=20)` | Second process exits without touching database; explicit lock error | Nonzero exit, stderr contains `application_busy`; first app remained usable; **PASS** |
| N4 native recovery startup failure | Fully quit; confirm exact executable absent in `ps`. SHA-256 closed DB. Write only test storage's `restore.pending` as `../invalid-b5-recovery`. Run exact test executable with capture/20s timeout. | Nonzero startup, invalid_recovery, existing DB and marker retained | All assertions passed; DB SHA-256 unchanged. Rename synthetic marker to `b5-invalid-marker.retained` within test storage, retain it. Reopen same bundle, quit and compare snapshots; **PASS** for startup rejection/data preservation, not user-facing startup error presentation |
| N5 repeated native launches / final bundle | Reopen after SIGKILL, then after invalid marker is moved aside; quit/snapshot. Rebuild final isolated bundle, open and quit. | Stable records/history and successful packaged startup | Three records visible on all reopens; complete pre/post failure snapshots identical. Final rebuilt bundle also displayed all three; **PASS** |

Closed snapshot inspection used Python's standard library. Confirm process exit and absence of `mission-control.db-wal` before copying bytes; never inspect a running WAL database by copying its main file. Open **only the temporary copy** using `sqlite3.connect('file:<copy>?mode=ro&immutable=1', uri=True)`. Compare ordered `SELECT id,value,created_at,note ... ORDER BY id` (B2 has only the first three columns), and `SELECT version,description,installed_on,success,hex(checksum),execution_time FROM _sqlx_migrations ORDER BY version`. Require `PRAGMA integrity_check` to return only `ok` and `PRAGMA foreign_key_check` to return no rows. B2→B3 comparison requires the original three fields and v1 history exactly equal, every new note empty, and versions 1/2; B3→B4 and later no-write reopens require complete equality. These comparisons actually passed. Direct read-only access to the copied WAL-mode header initially failed (`unable to open database file`), including an escalated attempt; immutable access to the closed, sidecar-free temporary copy succeeded. No repair or write to a live source was used.

`lsof` on the killed process confirmed the dedicated B5 DB and excluded normal production/development DB paths. `ps` required sandbox approval. Native test bundles, snapshots, backup, detached worktrees and synthetic storage were retained outside Git. No raw data, logs, bundles or machine-specific override files are committed.

## Remaining acceptance and environmental checks

| ID | Procedure required | Expected | Actual / status and limit |
|---|---|---|---|
| M1 device restart | Save synthetic records using a fresh isolated bundle, quit, independently arrange a full device restart, reopen and compare complete records/history/integrity | Data persists through OS restart | **NOT TESTED**; user machine was not restarted |
| M2 real power loss | Disposable machine/VM/storage fault environment, controlled power loss at durability boundaries, reopen and compare artifacts | Filesystem-specific crash consistency matches documented guarantees | **NOT TESTED**; SIGKILL and successful fsync calls cannot establish this |
| M3 native filesystem faults | Disposable volume/sandbox with bounded capacity and permission controls; repeat UI write/export/restore/startup operations | Actionable UI errors and safe records through actual OS faults | **NOT TESTED** in WebView; Rust actual PermissionDenied/SQLITE_FULL and test-only StorageFull boundaries passed. No host disk was filled. No claim of a partial physical fsync or controller I/O failure |
| M4 native restore interior kills | Dedicated bundle/storage; external observer/scheduler to interrupt during actual snapshot/move/plugin acknowledgement, then inspect originals/markers | Same recovery invariants as service tests | **NOT TESTED** for every internal packaged timing point; deterministic service states and real child kills at five stable boundaries passed. B4's native confirmed restore/quit/reopen remains prior evidence, not rerun here |
| M5 platforms/filesystems/distribution | Windows/Linux/mobile builds and durability matrix; unsupported hard-link destinations; signed updater/installer upgrades | Explicit failures or compatible preservation | **NOT TESTED**; Unix directory sync only; advisory lock does not coordinate old B2/B3 builds/external SQLite tools. Historical bundles were never run concurrently against shared data |
| M6 startup error UX and WebView data clearing | Dedicated app: inject startup failure and inspect user-visible presentation; clear only dedicated WebView data and reopen | Actionable startup guidance; native data intact | **NOT TESTED**; native startup stderr rejection passed, not an error-dialog UX check |

B5's automated scope is reproducible and passed. B5 and Stage 2 are not declared fully complete while these acceptance/environmental gaps remain. Existing recovery-retention, plaintext-backup and platform limitations remain unchanged.
