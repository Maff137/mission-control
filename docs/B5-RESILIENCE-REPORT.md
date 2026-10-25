# B5 — Integration and resilience evidence

Initial evidence: 2026-10-24; updated: 2026-10-25. Status: **PARTIALLY VERIFIED**. Automated macOS scope and the native checks below passed. The native M3/M4/M6 follow-up below verifies additional macOS acceptance scenarios. M1 device reboot is **PASS on macOS ARM64** (2026-10-25), as documented below. Power loss and remaining cross-platform/distribution acceptance remain unverified. Stage 2 remains **IN PROGRESS**.

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

## Remaining acceptance at the initial B5 pass (historical)

The statuses in this table record the initial B5 pass. The [native acceptance follow-up](#native-acceptance-follow-up--m3-m4-and-m6) supersedes M3, M4 and M6 with passing results within its documented scope. The M1 follow-up below supersedes the historical M1 NOT TESTED result. M2 and remaining M5 requirements remain pending; interruptions inside individual filesystem operations and midway through SQL plugin initialization remain unverified. B5 remains **PARTIALLY VERIFIED** and Stage 2 remains **IN PROGRESS**.

| ID | Procedure required | Expected | Actual / status and limit |
|---|---|---|---|
| M1 device restart | Save synthetic records using a fresh isolated bundle, quit, independently arrange a full device restart, reopen and compare complete records/history/integrity | Data persists through OS restart | **NOT TESTED at initial pass**; user machine was not restarted then. Superseded by **PASS on macOS ARM64** (2026-10-25); see [M1 follow-up](#device-reboot-acceptance-follow-up--m1) |
| M2 real power loss | Disposable machine/VM/storage fault environment, controlled power loss at durability boundaries, reopen and compare artifacts | Filesystem-specific crash consistency matches documented guarantees | **NOT TESTED**; SIGKILL and successful fsync calls cannot establish this |
| M3 native filesystem faults | Disposable volume/sandbox with bounded capacity and permission controls; repeat UI write/export/restore/startup operations | Actionable UI errors and safe records through actual OS faults | **NOT TESTED** in WebView; Rust actual PermissionDenied/SQLITE_FULL and test-only StorageFull boundaries passed. No host disk was filled. No claim of a partial physical fsync or controller I/O failure |
| M4 native restore interior kills | Dedicated bundle/storage; external observer/scheduler to interrupt during actual snapshot/move/plugin acknowledgement, then inspect originals/markers | Same recovery invariants as service tests | **NOT TESTED** for every internal packaged timing point; deterministic service states and real child kills at five stable boundaries passed. B4's native confirmed restore/quit/reopen remains prior evidence, not rerun here |
| M5 platforms/filesystems/distribution | Windows/Linux/mobile builds and durability matrix; unsupported hard-link destinations; signed updater/installer upgrades | Explicit failures or compatible preservation | **NOT TESTED**; Unix directory sync only; advisory lock does not coordinate old B2/B3 builds/external SQLite tools. Historical bundles were never run concurrently against shared data |
| M6 startup error UX and WebView data clearing | Dedicated app: inject startup failure and inspect user-visible presentation; clear only dedicated WebView data and reopen | Actionable startup guidance; native data intact | **NOT TESTED**; native startup stderr rejection passed, not an error-dialog UX check |

B5's automated scope is reproducible and passed. B5 and Stage 2 are not declared fully complete while these acceptance/environmental gaps remain. Existing recovery-retention, plaintext-backup and platform limitations remain unchanged.

## Native acceptance follow-up — M3, M4 and M6

Date: 2026-10-24. **M3 tested native scenarios PASS; M4 eight packaged boundaries PASS; M6 PASS. B5 remains PARTIALLY VERIFIED and Stage 2 remains OPEN.** M1 is being conducted independently by the user; no reboot evidence was supplied. M2 and M5 were not attempted.

### Environment, base and safety

Actual environment rechecked: macOS 27.0.1 (26A434), ARM64, Node 24.21.0, npm 11.19.0, Rust 1.99.1. Locked dependencies and applied migrations remain unchanged. A successful fetch left `origin/main` at B4 `dce2b2f` and the B5 branch at `e2004f5`. B5 was not integrated; the requested clean branch `test/b5-native-acceptance` was created from `e2004f5`, preserving B5 without merging it into main.

Only these disposable identifiers were used:

- `com.missioncontrol.acceptance.b5native20261024a`: first successful eight-boundary run, native UI, M3 volume tests and M6.
- `com.missioncontrol.acceptance.b5native20261024b`: complete final-code eight-boundary run with stronger marker/displacement assertions.

The production release bundle was built but never launched. Normal production/development databases were never opened or modified. No OS reboot/shutdown, global browser clearing, user security-setting change, host-disk filling, or unrelated process termination occurred. HOME was not overridden, and symlinks were not used as isolation. Temporary native permissions were restored. The APFS image was detached after verification. Synthetic files, cache quarantine, recovery copies and logs remain outside Git.

### Correction and instrumentation

The former startup chain propagated storage/SQL errors into `.expect`, with no actionable application window. The correction keeps the existing recovery → SQL preload/migrations → startup acknowledgement order, dynamically registers the same SQL plugin inside controlled initialization, and creates the ordinary WebView only after success. A failure opens the separate bundled `startup-error.html` window. It explains preservation, permissions, capacity and reopen steps, and includes the underlying error. Details are JSON-serialized into a native initialization script and assigned using `textContent`, never interpreted as HTML. The page has no external resources or application frontend scripts and uses restrictive CSP. No failure path deletes or resets the user's database. SQL error guidance also now addresses write failures rather than suggesting only backup selection.

A first attempted asynchronous startup dialog did not provide a stable visible error surface. A persistent dedicated error window replaced it; its guidance and actual details were verified through macOS accessibility and screenshot inspection. A native-only owner-window attempt did not compile because Tauri's WindowBuilder is private; no dependency or unstable Tauri feature was added.

`native-acceptance` is an opt-in debug-only Cargo feature. It prints the actual Tauri native storage path, rejects normal identifiers and a symlink storage root, and provides bounded 60-second synchronization at eight named boundaries. The controller waits for a flushed path/boundary signal (25-second deadline), SIGKILLs only its exact owned `Popen` Child and reaps it. There is no command, IPC permission or remotely accessible fault surface. The existing simulated Rust fault hooks remain `cfg(test)` only. Normal release binary scanning found no acceptance control markers, and a release build check with the feature was intentionally rejected by `compile_error!`.

### Reproduce packaged M4

From `app/`, use a **fresh, nonexistent** acceptance identifier and product name. Do not reuse the already-tested identifiers with this controller; it refuses existing storage.

```sh
npm run tauri -- build --debug --config '{"identifier":"com.missioncontrol.acceptance.examplefresh","productName":"Mission Control Acceptance"}' --bundles app -- --locked --features native-acceptance
python3 scripts/native-acceptance.py 'src-tauri/target/debug/bundle/macos/Mission Control Acceptance.app'
```

The executed builds used the two identifiers above and product names `Mission Control B5 Acceptance` / `Mission Control B5 Final Acceptance`. Both debug builds passed. No development server was running. `scripts/native-acceptance.py` is the committed reproduction procedure, including exact fixture setup, process handling and comparisons. It first initializes the actual SQL plugin, creates three original records, and produces a distinct one-record replacement with identical migration history. It constructs a confirmed restore marker as input; it does not claim to test UI confirmation publication. Each subsequent interruption is an actual termination inside the packaged application's startup/recovery/plugin lifecycle. The inspected states are produced by that process, not constructed as a substitute for process death.

Each comparison includes every record field (`id`, `value`, `created_at`, `note`) and every migration field (`version`, `description`, `installed_on`, `success`, checksum bytes, `execution_time`). Inspection copies the closed/reaped process's main DB and any WAL/SHM/journal companions into temporary storage. It never assumes a killed WAL database consists only of its main file. Every inspected live/recovery snapshot must pass `integrity_check = ok` and empty `foreign_key_check`.

| ID / synchronization boundary | Actual fault and durable state | Expected recovery | Observed result |
|---|---|---|---|
| M4-1 `before-snapshot` | SIGKILL before original SQLite snapshot; pending marker retained, no installation marker | Resume confirmed restore, retain original | Replacement exact, original exact, marker cleared after successful startup, two stable reopens; **PASS** |
| M4-2 `after-snapshot` | SIGKILL after verified original snapshot, before installing marker | Resume restore, preserve previous snapshot too | Replacement exact; original and `previous-original-*.db` exact; two reopens/integrity/FK; **PASS** |
| M4-3 `original-displaced` | SIGKILL just after main DB renamed into recovery, before remaining move/sync steps | Installation marker causes rollback to original | Main absent and candidate/raw original present at kill; original exact after both reopens, retained snapshot exact; **PASS** |
| M4-4 `before-install` | SIGKILL after original displacement and installation marker, before candidate rename | Roll back original | Candidate/raw original and absent live main asserted; original exact, marker cleared, two reopens; **PASS** |
| M4-5 `after-install` | SIGKILL immediately after incoming rename, before following directory sync/validation | Roll back original | Original exact after both reopens, retained recovery exact; **PASS** |
| M4-6 `before-sql` | SIGKILL after completed install/validation, before SQL plugin initialization | Roll back original | Original exact, full history unchanged, marker removed only by recovered startup; **PASS** |
| M4-7 `before-ack` | SIGKILL after actual SQL plugin preload/migrations completed, before acknowledgement | Roll back original | Original exact, retained recovery exact, two reopens; **PASS** |
| M4-8 `after-ack` | SIGKILL after acknowledgement removed pending marker | Keep replacement | Marker already absent at kill; replacement exact after both reopens, original snapshot retained; **PASS** |

All eight rows passed in the first successful run and again on final code. The final bundle was subsequently opened normally through native UI automation with all three original fixture records visible, then quit. Its invalid-marker error window was also checked again; after quit, database SHA-256 and full records/history still matched the original fixture, with integrity/FK passing. Only the synthetic marker was quarantined. Final primary killed PIDs, in table order: 38802, 38805, 38808, 38811, 38814, 38817, 38820, 38823. PID values are observations, **not commands to reuse**. Each run also owns/reaps its initialization child and two reopening children per row. Final `results.json` records states and outcomes.

**Limits:** These are real packaged-process deaths with deterministic scheduling, not simulated I/O errors. They cannot interrupt halfway through an atomic kernel rename, establish power-loss/fsync durability, or cover every individual sidecar rename. No kill occurred midway through SQL plugin initialization; the before-plugin and after-plugin/before-ack boundaries were tested. Checkpoints occur before ordinary window creation, so these are packaged startup tests, not WebView callback interruptions.

### Reproduce M3 and M6 native UI procedures

Use only the freshly initialized disposable bundle/storage produced above. Launch via Finder or native UI automation; confirm the bundle identifier in Info.plist and actual `app_config_dir` signal before any fault. Quit completely before file changes, except the explicitly described runtime permission/capacity tests. Keep copied closed-database baselines and compare using the controller's `snapshot()` helper on copies.

For exact native-dialog paths, use **Command-Shift-G**, set the Go to Folder path, confirm it, and then use a basename in Save As. Entering an absolute path into Save As may instead create a colon-separated filename in the current folder. In the actual run, two such synthetic exports were moved from Documents into the test artifact directory; no unrelated file was modified. Those attempts do not count as permission-fault evidence.

The actual bounded volume commands were:

```sh
hdiutil create -size 64m -fs APFS -volname MC-B5-Faults -type SPARSE -ov /private/tmp/mc-b5-faults-20261024.sparseimage
hdiutil attach /private/tmp/mc-b5-faults-20261024.sparseimage -nobrowse -owners on -mountpoint /private/tmp/mc-b5-fault-volume
# After quitting, preserving the dedicated native directory, and comparing baselines:
hdiutil detach /private/tmp/mc-b5-fault-volume
hdiutil attach /private/tmp/mc-b5-faults-20261024.sparseimage -nobrowse -owners on -mountpoint "$HOME/Library/Application Support/com.missioncontrol.acceptance.b5native20261024a"
# At completion, after quitting and copying only explicit synthetic application files:
hdiutil detach "$HOME/Library/Application Support/com.missioncontrol.acceptance.b5native20261024a"
```

These commands were executed successfully; hdiutil printed deprecation warnings suggesting diskutil equivalents. **For a new run, choose a fresh image/path/identifier, do not overwrite this image, and prepare the empty mountpoint only after archiving the test storage.** Verify mount identity with `stat().st_dev != parent.stat().st_dev`, no symlink, APFS device output, and filesystem size below 128 MiB before writing a filler. The executed image was 64 MiB, attached as disk4/APFS disk5. For native SQLite, `ps -axo pid=,comm=` identified the sole exact bundle executable; `lsof -p <owned-test-pid> -Fn` confirmed the dedicated main/WAL/SHM paths and excluded normal production/development DB handles. Actual DB test PID was 37395. Do not reuse recorded PIDs.

To exhaust capacity, the controller wrote only `<verified-test-mount>/bounded-fill`, with bounded loops (up to 128 × 1 MiB, then up to 32768 × 4096 bytes), catching and asserting `errno.ENOSPC`. The remaining 4 KiB allocations matter: the first 1 MiB allocation failure left enough space for a small valid export, which passed and was retained; it was **not** counted as a failed export. After exhausting residual blocks, native export failed with OS error 28. Free capacity by removing **only the positively identified synthetic filler**; a truncate attempt itself returned ENOSPC. Never fill a path without independently proving it is the bounded test mount.

| ID | Setup and action | Expected | Actual UI/error and preservation / status |
|---|---|---|---|
| M3-1 / M6-1 inaccessible native storage | Quit; native test directory mode 000, independent write probe must raise PermissionError; launch same bundle. Restore mode 0755 after quit | Actionable error, no reset; normal reopen after permissions restored | Dedicated error window included `file_operation: Permission denied (os error 13)`, preserve-files and permissions/capacity/reopen guidance. Valid DB SHA-256 unchanged; normal launch succeeded; **PASS**, actual OS EACCES |
| M3-2 export permission | `/private/tmp/mc-b5-export-denied` mode 0500; independent failed probe; export `denied.mcbackup` there | Explicit error; no published destination; records safe | UI showed OS error 13 and permission/disk guidance. Final destination absent. Directory restored to 0700; subsequent export succeeded on recovered APFS volume; six original records/full history exact; **PASS**, actual EACCES. Same-directory retry was not separately exercised |
| M3-3 export capacity | Verified bounded APFS at `/private/tmp/mc-b5-fault-volume`, exhaust 4 KiB allocations; export `full-exhausted.mcbackup` | ENOSPC, no partial final backup; exact retry succeeds after freeing capacity | UI showed `No space left on device (os error 28)`. Final path absent before freeing filler. Same name/volume retry succeeded; exported records/history exactly matched six-row baseline; integrity/FK passed; **PASS**, actual OS ENOSPC |
| M3-4 staging workspace permission | While bundle running, native test directory mode 0500; verified EACCES probe; select valid external backup | Workspace creation fails; no live replacement; retry possible | Native UI showed OS error 13; original six-row dataset/history unchanged. Directory restored to 0755; same backup later reached confirmation, then Cancel succeeded; **PASS**, actual EACCES |
| M3-5 source-copy permission | Native directory writable; synthetic `valid.mcbackup` mode 000, independent read probe fails; select it in native dialog | Copy fails safely, no confirmation or database replacement | UI showed OS error 13. Restore source to 0600 and select same file: validation/confirmation succeeded, Cancel retained all six original rows/history; **PASS**, actual EACCES |
| M3-6 native SQLite capacity | Quit/archive isolated storage, mount bounded APFS at the exact native app-config directory, copy six-row fixture, launch, verify actual SQLite handles, exhaust volume, Save `APFS disk-full write must fail` | SQLITE_FULL; no partial row or loss; one successful row only after freeing capacity and retry | UI showed `Could not save record` with SQLite code 13, records stayed visible. After deleting only filler, same input saved successfully. Final DB had exactly the original six complete rows plus one retry row; every migration field unchanged; integrity/FK passed; **PASS**, actual APFS capacity failure, not max_page_count simulation |

The observed SQL FULL error originally ended with backup-only advice. Final code adds permissions/free-space/preserve-files advice, with a Rust regression test. The OS failure and successful UI retry were executed before that wording-only change; the final build and regression tests passed afterward. Neither native READONLY/BUSY nor partial physical flush failure is claimed by this follow-up; existing Rust READONLY/BUSY evidence remains separate.

No valid committed data was silently lost in any M3 row. Published exports remained exact; failed final export paths were absent. Closed comparisons covered the six original records/history after all permission/export/copy/WebView scenarios and after the native SQLite failure/retry. Recovery copies from M4 were independently checked for exact originals and integrity. The final volume was detached and the seven-row test DB returned to ordinary isolated native storage. A broad artifact copy initially encountered APFS's inaccessible `.fseventsd`; cleanup was corrected to preserve only the explicit synthetic DB/lock/export files, without changing volume system permissions.

| ID / startup or WebView scenario | Exact setup / expected behavior | Observed result / status |
|---|---|---|
| M6-2 invalid marker | Quit, hash closed test DB; `restore.pending = ../invalid-acceptance-marker`; launch | Visible dedicated error window showed `invalid_recovery: Invalid restore marker.` plus recovery guidance. After quit DB SHA-256 unchanged; synthetic marker retained separately; **PASS** |
| M6-3 invalid recovery snapshot | Create `restore-<uuid>/installing`, invalid `original.db`, pending marker referencing that directory; launch | Dedicated window showed `invalid_size` and underlying 512-byte–64-MiB constraint; existing valid DB SHA-256 unchanged. Invalid snapshot and marker retained; removing only the synthetic pending fixture allowed later launch; **PASS** |
| M6-4 corrupt native DB / SQL plugin error | Quit and preserve valid DB; replace only test DB with 4117-byte non-SQLite fixture; launch | Dedicated window showed SQL plugin migration failure/code 26, `file is not a database`, and preserve/reopen guidance. Corrupt input SHA-256 unchanged. Test controller retained corrupt fixture and reinstated the preserved valid test DB; successful launch showed original records; **PASS**. This is explicit fixture restoration, not automatic repair |
| M6-5 WebView independence | Save three new UI records (Unicode/quote), giving six rows; quit. Capture all fields/history. Quarantine only exact test WebKit and cache directories; reopen and quit again | Six values and identical original timestamps visible after reopen. Closed-copy comparison proved exact IDs/values/notes/timestamps and every migration field, integrity `ok`, no FK rows; **PASS** |

For M6-5 the exact quarantined paths were `~/Library/WebKit/com.missioncontrol.acceptance.b5native20261024a` and `~/Library/Caches/com.missioncontrol.acceptance.b5native20261024a`. The former contained dedicated WebsiteData (IndexedDB, LocalStorage, resource-load statistics, salts); the latter contained WebKit NetworkCache/CacheStorage/HSTS. Both had the exact disposable identifier and were checked not to be symlinks. `~/Library/HTTPStorages/<same-id>` did not exist. Quarantine was a reversible move after quit into `/private/tmp/mc-b5-m6-evidence/webview-quarantine/`; no Safari/global profile data was accessed. Native SQLite was untouched by quarantine and remained authoritative.

### Actual validation and retained evidence

All requested final-code checks from `app/` **PASS**:

- `npm run check`: TypeScript + 13 Vitest tests.
- `npm run build`: TypeScript + Vite assets, including the new error page.
- `npm run rust:check`: locked Cargo check.
- `npm run rust:test`: **24 passed, zero failed, one child helper ignored** in the parent run; existing B3–B5 suites plus two error-guidance regressions.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings`.
- `npm run desktop:build -- -- --locked`: release `.app`, never launched.

Additional **PASS**: feature-enabled locked all-target Clippy; both isolated debug `.app` builds; both successful eight-boundary controller executions; final release binary control-marker absence; intentional release-feature rejection using `cargo check --manifest-path src-tauri/Cargo.toml --locked --release --features native-acceptance` (nonzero with the expected debug-only compile error). This expected rejection is not a successful compilation. Historical B5-only test commands above were not separately rerun in this follow-up; their tests ran in the complete suite.

Retained local evidence (synthetic data; never staged/committed):

- First successful matrix: temporary-directory basename `mc-b5-native-gc2s7p6a`, under the user's macOS temporary root; per-child logs, archived states, fixtures and `results.json`.
- Final matrix: `mc-b5-native-hwfqtz_u` under the same temporary root; eight PASS rows and strengthened durable-state assertions. Actual root was `/private/var/folders/hj/8f48nsds0nx1hzqbj6vy__tw0000gn/T/` (also presented by macOS as `/var/folders/...`).
- `/private/tmp/mc-b5-m6-evidence/`: SHA-256 baselines, retained invalid marker/snapshot fixture references, corrupt input, `before-webview.json`, `m6-comparison.json`, `m3-comparison.json`, verified native SQLite handles, cache quarantine and pre-volume/final-volume native data.
- `/private/tmp/mc-b5-faults-20261024.sparseimage`: detached bounded image with synthetic exports/DB; no host-space exhaustion.

Preparation failures are not counted as acceptance passes: an npm command first used the repository root instead of `app/`; Python bytecode compilation could not write its default cache; the first fixture controller failed to close Python SQLite connections (context managers commit but do not close), leaving a WAL-backed replacement incomplete when copied. Explicit connection closure fixed this and the full matrix then passed twice. Direct read-only access to a closed UI-created native DB again failed on this environment; inspection was changed to copied main/sidecar files. One preparation command chain continued to mount after a comparison failure, temporarily hiding the existing disposable directory; the image was detached, data compared/preserved, and the ordered setup was rerun successfully before any SQLite fault injection. No data was lost or production/development storage accessed.

**Remaining requirements after this historical follow-up (M1 superseded below):** M2 controlled real power loss on a disposable environment; M5 cross-platform/filesystem/distribution testing. Intra-syscall/physical-flush and mid-SQL-initialization timing remain unverified. B4's retained-copy management, unauthenticated plaintext backup, advisory-lock and Unix directory-durability limits still apply. This follow-up does not close B5 or Stage 2.

## Device reboot acceptance follow-up — M1

Date: 2026-10-25. **M1: PASS on macOS ARM64.** B5 remains **PARTIALLY VERIFIED** and Stage 2 remains **IN PROGRESS**.

- **Environment:** Isolated disposable `Mission Control M1 Reboot.app`, identifier `com.missioncontrol.acceptance.m1reboot20261025`, native database `mission-control.db`. Evidence is retained outside Git in `~/Documents/MissionControl-B5-M1/`.
- **Procedure and observations supplied by the user:** The isolated Tauri application compiled and packaged successfully. Before reboot, `B5 M1 Reboot Test Alpha` and `B5 M1 Reboot Test Beta` were created and verified directly in SQLite. The user performed a full macOS restart; the same application reopened successfully and displayed both records with their original timestamps. The final acceptance script returned `M1 ACCEPTANCE: PASS`, verifying identical pre/post SQLite dumps, unchanged complete records and SQLx migration history, `PRAGMA integrity_check = ok`, no `PRAGMA foreign_key_check` violations, and different macOS boot timestamps.
- **Local evidence inspected for this documentation update:** `before.sql` and `after.sql` are byte-for-byte identical (1,252 bytes each), including both complete records and both SQLx migration rows. Alpha retains `2026-10-25T20:21:35.543Z`; Beta retains `2026-10-25T20:21:43.557Z`. `integrity-before.txt` and `integrity-after.txt` both contain `ok`. `boot-before.txt` records boot seconds `1792378119`; `boot-after.txt` records `1792959986`, confirming different boots. The acceptance script/output and separate foreign-key-check output were not present in this evidence directory; those results are recorded from the user's supplied observations, not claimed as rerun here. No application tests, reboot, or live database inspection were performed for this documentation update; evidence files were read only and retained unchanged.
- **Scope and remaining requirements:** This verifies ordinary device-restart persistence only for the isolated macOS ARM64 run. M2 controlled real power loss remains **NOT TESTED**. M5 remains **PARTIALLY VERIFIED** by B6 evidence, with native Windows/Linux/mobile execution, remaining filesystem/durability coverage and signed installer/updater distribution acceptance pending. It does not establish power-loss, physical-flush or storage-controller durability. Existing mid-SQL-initialization, retention, plaintext-backup, advisory-lock and platform limitations remain unchanged.
