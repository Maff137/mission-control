# B6 — Cross-platform compatibility and Stage 2 acceptance

Verification date: 2026-10-25. **PARTIALLY VERIFIED**. Stage 2: **IN PROGRESS**.

B6 removes a concrete desktop packaging restriction and establishes macOS ARM64 native recovery/persistence evidence plus Windows/Linux automated feasibility checks. Only macOS native application execution was tested. Mobile builds are blocked by missing toolchains and unresolved backup-provider integration. M1 reboot and M2 real power loss remain NOT TESTED; M5 is PARTIALLY VERIFIED. No critical requirement was waived and no official cross-platform support is declared.

## Scope and environment

Base: fetched `origin/main` at `6fd860c06c9076ed3c3316926ba8f707647c8b95`; clean initial workspace; branch `test/b6-platform-compatibility`. macOS 27.0.1 (26A434), ARM64; Node 24.21.0, npm 11.19.0, rustc/cargo 1.99.1. Only installed Rust target: `aarch64-apple-darwin`; Command Line Tools at `/Library/Developer/CommandLineTools`; full Xcode absent. `adb` and `sdkmanager` absent from PATH; ANDROID_HOME/ANDROID_NDK_HOME/JAVA_HOME unset. `/usr/libexec/java_home -V` also found no Java Runtime; `/usr/bin/java` is only the system launcher. Standard Xcode/Android Studio/SDK locations inspected were absent. No heavyweight SDK installation or mobile initialization attempted.

Locked Tauri 2.12.5, CLI/API 2.12.4, tauri-build 2.7.4, SQL plugin 2.6.0, dialog 2.8.3, SQLx 0.8.6, SQLite sys 0.30.1. Frontend lock versions: React/ReactDOM 19.3.1, TypeScript 6.0.3, Vite 8.3.5, Vitest 4.1.12. A direct installed-manifest versus package-lock comparison passed for every declared npm dependency/devDependency. `tauri info` also reported a leftover JavaScript SQL-plugin installation locally; it is absent from declared dependencies and the lockfile, and no application import uses it. Clean `npm ci` on CI passed without that package. Existing lockfiles and migrations unchanged. Engineering feasibility does not establish officially supported OS versions or distribution readiness.

## Platform compatibility matrix

| Platform | Static inspection | Compilation | Packaging | Native launch | Native DB initialization / quit-reopen | Backup / restore in native app | Device restart | Distribution readiness |
|---|---|---|---|---|---|---|---|---|
| macOS ARM64 | PASS | PASS | PASS release/debug .app | PASS | PASS | PASS, APFS; exFAT export safely rejected | NOT TESTED | NOT TESTED signed delivery |
| Windows x64 | PARTIALLY VERIFIED | PASS CI check/tests/release | PASS unsigned NSIS on retry | NOT TESTED | NOT TESTED | NOT TESTED | NOT TESTED | NOT TESTED |
| Linux x64 | PARTIALLY VERIFIED | PASS CI check/tests/release | PASS unsigned .deb | NOT TESTED | NOT TESTED | NOT TESTED | NOT TESTED | NOT TESTED |
| Android ARM64 | PARTIALLY VERIFIED | BLOCKED: SDK/NDK/JDK/target missing | BLOCKED | NOT TESTED | NOT TESTED | NOT TESTED | NOT TESTED | NOT TESTED |
| iOS ARM64 | PARTIALLY VERIFIED | BLOCKED: full Xcode/target missing | BLOCKED | NOT TESTED | NOT TESTED | NOT TESTED | NOT TESTED | NOT TESTED |

CI test binaries exercise native storage code on runner temporary files. Their success is not the native application/database initialization evidence requested in the matrix.

## Changes and validation

Platform-specific `tauri.windows.conf.json` / `tauri.linux.conf.json` overlays select NSIS with embedded WebView2 offline installer and Debian packaging. The base remains `.app`. `desktop:build` delegates target selection to Tauri instead of overriding all hosts with `--bundles app`. No identity, capability, CSP, migration or persistence engine change. Workflow uses Windows 2022 and Ubuntu 24.04 x64 runners, fixed Node/Rust, `npm ci`, locked Cargo checks/tests/Clippy/build, no signing/release/upload and read-only contents permissions. CI does not launch the app. Test subprocesses exercise storage services, not packaged WebViews.

Initial local checks PASS: `npm run check` (13 Vitest), `npm run build`, `npm run rust:check`, `npm run rust:test` (24 passed, one ignored helper), `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings`. Final-code results: all seven requested commands PASS, with **14 Vitest tests and 25 Rust tests passed, zero failed, one child helper ignored**. Release `.app` packaging PASS (6.22 MiB); production bundle never launched. See native evidence below. An initial final-suite invocation from repository root failed with missing package.json; rerun from app. Sandbox denied initial fetch/GitHub auth/network metadata checks; approved retries succeeded. No failure counted as PASS. A final post-error native reopen and closed DB SHA-256 comparison also passed; the app was left quit and the test volume detached.

### Remote CI evidence

Implementation commit: `8249e8cda379a97d5ec8888b40ee84da8e5bb167`. [Compatibility run 40225328483](https://github.com/Maff137/mission-control/actions/runs/40225328483). This was an actual branch-push execution, not an assumed future workflow result. Both runner environment logs reported Node 24.21.0, npm 11.19.0 and Cargo 1.99.1.

- **Linux x64 PASS:** Ubuntu 24.04 runner image `20261012.334.1`, Rust `1.99.1-x86_64-unknown-linux-gnu`; 14 Vitest, 25 Rust passed, zero failed, one ignored helper; frontend build, locked Rust check, rustfmt and all-target locked Clippy PASS. `npm run desktop:build -- -- --locked` produced `Mission Control_0.1.0_amd64.deb` (3.16 MiB). Packaging completed at 19:25:13 UTC. No app launch or installation performed.
- **Windows x64 initial attempt PARTIALLY VERIFIED:** Windows Server 2022 image `20261022.344.1`, Rust `1.99.1-x86_64-pc-windows-msvc`; 14 Vitest and **24 Rust passed**, zero failed, one ignored helper. The Unix-only permission test is not compiled on Windows, explaining the count difference. Frontend build, locked Rust check, formatting and all-target Clippy PASS. Release `mission-control.exe` compiled successfully (4m43s). NSIS 3.13 and its Tauri utility downloaded/hash-validated, but packaging **FAIL** at Microsoft offline WebView2 download `https://go.microsoft.com/fwlink/?linkid=2124701` with `Peer disconnected`. This is not a successful installer or an application-code compile failure. One failed-job rerun **PASS** on the same implementation commit: all checks passed again (14 Vitest, 24 Rust, one ignored helper), release compilation completed in 5m49s and `Mission Control_0.1.0_x64-setup.exe` was produced at 19:42:13 UTC (209.84 MiB). Overall run attempt 2 succeeded; no source/config/dependency change was needed between attempts. The initial network failure remains part of the record, not reclassified as a pass. No installer was launched. Offline WebView2 policy was not weakened. The [CLI 2.12.4 bundler source](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.4/crates/tauri-bundler/src/bundle/windows/util.rs) resolves the Microsoft URL and downloads the offline installer at build time; application lockfiles do not pin that externally served runtime. This network dependency belongs to package creation, not ordinary installed application storage/operation. No TLS bypass, dependency patch or online-only installer fallback was applied.
- Workflow commands are identical to the local table below after `npm ci`, except host-specific native dependency installation/toolchain setup recorded in the workflow. Logs are retained locally under `/private/tmp/mc-b6-evidence/`; no packages, databases or credentials uploaded as artifacts. A log request via `gh run view --log` was blocked while the overall run was active; completed-job API retrieval succeeded after explicitly allowing terminal escapes into a file and stripping them before inspection. These retrieval issues are not build failures.

### Required local automated checks (implementation commit 8249e8c)

Every command ran from `app/` on macOS ARM64:

| Command | Actual result |
|---|---|
| `npm run check` | PASS: TypeScript and 14 Vitest tests |
| `npm run build` | PASS: TypeScript and Vite bundled assets |
| `npm run rust:check` | PASS: Cargo check --locked |
| `npm run rust:test` | PASS: 25 tests, zero failed, one ignored subprocess helper |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | PASS |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings` | PASS |
| `npm run desktop:build -- -- --locked` | PASS: release .app, production bundle not launched |

The B6 Rust regression covers Unicode/space paths, exact record/history export and restore, repeated startup, existing destination rejection and byte-identical no-clobber backup retention. The frontend regression prevents a host-independent `.app` CLI override and checks native overlay targets without identity/security overrides. Full migration, malformed backup, recovery, filesystem fault and process-death suites remain included; no applied migrations were modified.

## Storage and mobile inspection

Native Store and SQL plugin both resolve `app_config_dir`: macOS Library/Application Support/identifier, Windows Roaming AppData/identifier and Linux XDG_CONFIG_HOME/identifier (normally ~/.config/identifier). These are static resolutions outside macOS until native evidence exists. There is no browser-storage fallback. File locks use Rust `File::try_lock`; SQL connections close before replacement. Same-filesystem rename and no-clobber hard links underpin recovery/publication. The SQL plugin uses SQLx connection defaults; locked SQLx leaves journal mode unchanged and documents SQLite synchronous FULL as the default. These defaults do not substitute for measured power-loss behavior. Directory sync executes only under `cfg(unix)`; Windows returns success without directory flush, so power-loss directory-entry durability is unverified. Unix sync success also does not establish physical power-loss durability. Windows privacy relies on inherited ACLs; mode 0600/0700 is Unix-only. Network filesystems, removable media and filesystems without hard links are unsupported/unverified; safe errors must be checked per filesystem. No alternative unsafe copy/overwrite fallback is introduced.

Mobile entry point exists but generated mobile projects do not. SQL and dialog provide mobile implementations, but framework support does not validate this app. `choose()` converts FilePath to a local PathBuf; Android content:// providers cannot pass `into_path()`, and export requires hard links unavailable in document-provider destinations. Mobile sandbox/file-provider access, export semantics, locking, restore and suspension remain unverified. Desktop backup semantics must not be advertised as mobile support.

## Distribution findings and unsupported configurations

**PARTIALLY VERIFIED:** Packaging configuration is an engineering feasibility probe, not a release support policy. Minimum OS versions remain OPEN. Locked tauri-utils defaults are macOS 10.13, iOS 15.0 and Android API 24; they are framework defaults, not tested Mission Control minimums, and an ARM64 bundle cannot establish Intel/macOS 10.13 support. No explicit Windows minimum WebView2 version is configured. The Linux runner baseline is Ubuntu 24.04; its package must not be advertised for older glibc distributions without testing. [Tauri Debian guidance](https://v2.tauri.app/distribute/debian/) requires building on the oldest intended compatible base and lists WebKitGTK/GTK runtime dependencies. `.deb` packaging does not bundle all OS dependencies for an offline clean system; provision those dependencies before offline operation.

**NOT TESTED:** Signed delivery, installers on clean machines, uninstall/reinstall preservation and signed upgrades/updater behavior. No updater plugin or signing configuration was introduced. Windows embeds the offline WebView2 installer, but clean offline installation and SmartScreen/certificate behavior require real Windows acceptance. macOS local bundles do not establish notarization/Gatekeeper behavior. [Apple distribution guidance](https://v2.tauri.app/distribute/app-store/) requires Developer Program enrollment and signing; Store sandbox behavior may affect file selection and paths. Android distribution requires a signing/key and package policy review ([Google Play guide](https://v2.tauri.app/distribute/google-play/)); no credentials, store uploads or releases were used.

**Known limitations:** exFAT export is unsupported and failed safely in the actual native test; application storage itself also needs hard-link support for markers. Windows/Linux storage service behavior has automated evidence on runner temporary directories; exact runner filesystem types were not independently recorded. Network shares, FAT variants, case-sensitive/case-insensitive path aliases, Windows reparse points/ACL changes, antivirus interference, mobile provider URLs and directory-entry power-loss guarantees need native acceptance. Existing retained plaintext backups/originals have no pruning UI or authentication. The process lock coordinates current builds, not legacy B2/B3 or external SQLite tools. Portable schema/migration compatibility is covered by unchanged tests; actual transfer of a backup between different operating systems remains NOT TESTED.

## macOS native execution and recovery evidence

Executed on implementation commit `8249e8cda379a97d5ec8888b40ee84da8e5bb167` with the unchanged application runtime, using only fresh `com.missioncontrol.acceptance.b6native20261025a`, product `Mission Control B6 Acceptance`. The exact commands from `app/` were:

```sh
npm run tauri -- build --debug --config '{"identifier":"com.missioncontrol.acceptance.b6native20261025a","productName":"Mission Control B6 Acceptance"}' --bundles app -- --locked --features native-acceptance
python3 scripts/native-acceptance.py 'src-tauri/target/debug/bundle/macos/Mission Control B6 Acceptance.app'
```

Both PASS. Reproduction requires a new identifier; controller rejects existing storage. Build generated a 40.00 MiB `.app`. The native controller initialized the actual SQL plugin and passed all eight packaged interruption boundaries: before/after snapshot, original displacement, before/after incoming rename, before SQL, before acknowledgement and after acknowledgement. Every row preserved the expected exact records/full migration history, retained originals, integrity/FK and two stable reopens. Per-boundary logs and `results.json` are retained under temporary basename `mc-b5-native-fk5k4_mc` in the macOS user temporary root. This repeats B5 process-death evidence, not M1 or M2; fixture confirmation is constructed by the controller, not performed through UI in those rows.

Subsequent UI acceptance actually exercised native confirmation separately:

| Scenario | Expected / actual | Status |
|---|---|---|
| Bundled startup and native location | UI loaded `tauri://localhost` without Vite; three controller-created synthetic rows displayed. Controller path signal and `lsof -c mission-control -Fn` identified only test bundle/native DB/lock; no normal development/production database | PASS |
| Native insert/export/reopen | Saved Unicode/quote fourth row, exported `b6-native.mcbackup` via native Save, inserted fifth row, quit/reopened with all five original timestamps. Closed-copy snapshots: backup four rows, live five, identical complete history, integrity/FK | PASS |
| Confirmed restart restore | Selected backup through native Open, explicit replacement confirmation blocked writes; quit/reopen displayed four restored rows and retained-original success. Complete live/export/retry snapshots equal; retained `restore-*/original.db` equals exact five-row baseline, pending marker cleared | PASS |
| Invalid import | Selected 1152-byte non-SQLite `.mcbackup`; UI rejected with format error, four-row DB/history unchanged | PASS |
| Unsupported destination filesystem | Disposable 64 MiB exFAT image, different device ID from parent, no symlink, size bounded below 128 MiB. Independent hard-link probe returned OS error 45. Native export also reported Operation not supported; no final backup or `.mission-control-export-*` workspace remained. Export retry to local APFS succeeded and matched original backup exactly | PASS for safe rejection; exFAT export unsupported |
| Browser-storage independence | Quit; moved only test identifier's Library/WebKit and Library/Caches directories into evidence quarantine. Reopened with four unchanged rows/timestamps; copied native snapshot exactly equaled pre-quarantine records/full history, integrity/FK | PASS |
| Fail-closed startup | Closed test DB SHA-256 recorded; synthetic `restore.pending = ../invalid-b6-marker`. Launch displayed bundled startup-error page with `invalid_recovery` and preservation guidance; after quit DB SHA-256 and marker unchanged. Only synthetic marker moved to retained evidence afterward; normal reopen displayed all four rows again and app was quit | PASS |

Snapshots include `id,value,created_at,note` and every `_sqlx_migrations` field (including checksum bytes, installed_on and execution_time). Inspection uses B5 `snapshot()` on disposable copies including any sidecars after exact process exit confirmation. Integrity must be `ok` and foreign-key checks empty. UI commands never accessed real records. A final `lsof -nP -iTCP:1420 -sTCP:LISTEN` probe confirmed no development-server listener. Network interfaces were not disabled; bundled/offline architecture was verified without a development server or required account, not by host network disconnection.

Retained local synthetic evidence: `/private/tmp/mc-b6-evidence/` contains exports, before/after JSON snapshots, SHA-256 baseline, invalid input/retained marker and WebView quarantine. Native isolated storage remains available for manual M1, and recovery originals remain in it; never downgrade that storage with historical builds. The exFAT image is retained detached as `exfat.sparseimage`. Executed image commands:

```sh
hdiutil create -size 64m -fs ExFAT -volname MC-B6-ExFAT -type SPARSE /private/tmp/mc-b6-evidence/exfat.sparseimage
hdiutil attach /private/tmp/mc-b6-evidence/exfat.sparseimage -nobrowse -mountpoint /private/tmp/mc-b6-exfat
diskutil info /private/tmp/mc-b6-exfat
hdiutil detach /private/tmp/mc-b6-exfat
```

`diskutil` identified File System Personality ExFAT; the partition label Windows_NTFS is a partition type, not evidence of NTFS testing. A directory-based host diskutil probe failed; `df /private/tmp` identified `/dev/disk3s5`, and `diskutil info /dev/disk3s5` confirmed the local retry filesystem as APFS. Image operations passed with hdiutil deprecation warnings; no host disk filling or physical filesystem failure. A post-quit UI state query returned procNotFound as expected after process exit; independent process checks preceded file inspection. No new UI OS permission/capacity injection or historical native upgrade lineage was rerun: B5 supplies those separately; unchanged B3/B4 failure/migration tests ran in the full automated suite.

## Remaining acceptance procedures

### M1 — NOT TESTED

Prepared evidence exists: the B6 isolated bundle/storage has four committed records, a matching exported backup and `after-restore.json` containing every record/migration field. Its final closed DB remained byte-identical after error recovery/reopen. This is a ready baseline for an independently scheduled restart once copied to a stable acceptance location; no restart result is implied.

For a new independent run, use a fresh `com.missioncontrol.acceptance.*` identifier and disposable packaged app. Confirm Info.plist identity and actual native storage signal. Launch without Vite, save recognizable Unicode/quote records, export a backup, quit fully, copy closed DB plus any sidecars into retained evidence and snapshot every record/history field with the B5 controller helper. Record OS/filesystem/build/identifier, wall-clock time and baseline integrity/FK. Before reboot, copy the isolated bundle and evidence to a stable, user-chosen acceptance folder outside temporary/build directories so temporary cleanup or a later build cannot invalidate the baseline. Do not change the bundle identifier between runs. User independently restarts the device at a convenient time; never automate reboot. Reopen the exact bundle offline, inspect records, quit, snapshot copies again and require exact IDs/values/notes/timestamps/full migration equality plus integrity ok and empty FK check. Repeat startup and record operator/reboot evidence. Include confirmed restore followed by reboot and retained-original comparison for restart recovery coverage. A quit/reopen or SIGKILL is insufficient.

### M2 — NOT TESTED

No disposable authorized physical power-loss environment is available. Require a sacrificial device or explicitly isolated storage/VM environment whose power-cut and flush/cache model is documented. VM process kill alone does not prove physical durability. Agree authorized boundaries and expected guarantees before testing committed writes, export publication, durable confirmation, snapshot/displacement/install/acknowledgement. Capture pre-cut complete snapshots, cut only disposable environment power, boot/recover and compare records/history, pending markers, original copies and integrity/FK after multiple reopens. Record filesystem, mount options, virtual/physical caches and hardware guarantees. No host power interruption, real-filesystem corruption or device restart performed. Proposed risk acceptance is OPEN and requires explicit user approval; successful fsync/SIGKILL is not approval or evidence.

### M5 — PARTIALLY VERIFIED

The compatibility workflow executed successfully on Windows/Linux after one Windows retry; retain its run URL/commit/job results when repeating. On actual Windows NTFS and Linux local filesystem, install only isolated unsigned test packages under fresh identifiers, launch offline and perform the B4 UI export/restore/cancel/invalid-input and B5 startup/locking/reopen checks. Compare complete snapshots and retained originals; test non-ASCII paths, existing destinations, permissions/capacity, hard-link failure, and filesystem-specific rename/open-handle semantics. Inspect actual native paths and WebView independence. CI compilation and Rust tests cannot replace these native checks. Signed install/update preservation, clean offline Windows installation without WebView2, macOS signing/notarization/Gatekeeper, Linux minimum glibc/WebKitGTK matrix, Android SDK/device/provider and iOS Xcode/device/signing remain pending.

## Stage 2 recommendation

Keep B5 and B6 PARTIALLY VERIFIED and Stage 2 IN PROGRESS. Critical missing evidence is independent M1 reboot, M2 genuine isolated power-loss durability (or explicitly approved scope/risk decision), and M5 native platform/filesystem/distribution acceptance or explicit approval of a narrower support policy. **OPEN proposed decision:** whether to narrow the Stage 2 acceptance gate to the tested macOS ARM64 scope and explicitly defer power-loss/mobile/distribution guarantees. This would change the existing gate and requires user approval; the current broader criteria remain binding. No such decision is approved here. Historical B1–B5 evidence remains valid only within its recorded scope.

## Primary references inspected

- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/): native desktop dependencies and mobile toolchains.
- [Windows installers](https://v2.tauri.app/distribute/windows-installer/): NSIS/MSI and offline WebView2 options.
- [Dialog plugin](https://v2.tauri.app/plugin/dialog/): platform-specific picker behavior.
- Locked local crate sources: tauri path resolvers, SQL wrapper, dialog mobile implementation and FilePath::into_path; these establish current implementation details rather than newer documentation alone.

Documentation verification: complete task diff reviewed; both README.md and docs/STATE.md synchronized with this evidence; local Markdown link targets and `git diff --check` passed. Final evidence commit changes documentation only; automated/native results above apply to implementation commit 8249e8c.

## Hosted packaged-runtime acceptance follow-up

Implementation branch: `test/native-runtime-acceptance`, based on freshly fetched main `ee8492a`. The existing B6 matrix/checks/toolchains/lockfiles, NSIS offline WebView2 policy, Debian dependencies, CSP, capabilities and application source remain unchanged. The single existing package build per platform now uses a generated, fresh `com.missioncontrol.acceptance.ci<uuid>` overlay (only identity/product name), then installs the release NSIS or Debian package. No duplicate build job or debug instrumentation is needed.

`app/scripts/runtime-acceptance.py` refuses existing storage and normal application identities, owns/reaps only its launched processes, and inspects copied closed SQLite databases including sidecars. It verifies native SQL initialization, schema objects/columns, immutable migration checksums and complete records/history, then compares termination/relaunch and actual packaged startup restore with a distinct retained original and another reopen. Initial synthetic records and the durable restore-confirmation input are constructed by the controller; these steps are **packaged native startup acceptance, not UI insert/export/confirmation tests**. Windows restore input is an explicitly labeled Python-generated SQLite fixture. Existing Rust service tests retain separate service-only scope.

Linux additionally attempts keyboard-driven Save and native Save-dialog export through the installed bundled WebView under Xvfb/DBus, checking the committed row and complete exported snapshot rather than counting a visible process as UI success. Windows probes actual `UserInteractive`, process session and `OpenInputDesktop` availability; these observations do not themselves prove WebView rendering or dialog usability. Windows UI acceptance remains BLOCKED without a reliable UI driver, even if an input desktop exists. No assumption that every Windows runner is in Session 0 is made. See [GitHub runner images](https://github.com/actions/runner-images) for image inventory; image availability is not a guarantee of interactive UI automation.

Each run starts with BLOCKED evidence, fails on unexpected runtime errors, and publishes per-scenario PASS/FAIL/BLOCKED JSON and a job summary. An overall BLOCKED result preserves incomplete acceptance even when tested subsets pass. Fourteen-day artifacts contain only synthetic JSON, logs, Markdown and Linux screenshots; no installer, database, user files or credentials are uploaded. Package SHA-256, commit, run URL, identifier, storage path and exact snapshots make the evidence auditable. Failed prerequisite/check/build steps remain BLOCKED for runtime rather than becoming compilation-based passes.

No Vite/server is started and the controller refuses a listener on port 1420 before every launch. Release assets are embedded. Linux runs as the original unprivileged user in an isolated network namespace with only loopback and Unix-domain X11/DBus access; the host network is unchanged. Windows disconnected-network operation, clean offline installation without prerequisites, UI restore/cancel/invalid-import, reboot, power loss, mobile and signed update/distribution acceptance remain BLOCKED/untested by this automation. No host firewall, security setting, production/development database or user file is changed. B5/B6 remain **PARTIALLY VERIFIED**, Stage 2 **IN PROGRESS**.

### Minimal remaining Windows/Linux VM procedure

1. Provision a disposable Windows x64 or Ubuntu 24.04 x64 VM with an interactive desktop, platform WebView/GTK prerequisites and a local filesystem supporting hard links. Use a fresh acceptance identifier/product-name build overlay as generated by the controller; never install a normal production/development identity. Record commit, package SHA-256, OS, filesystem, identity and actual native config path. Install the NSIS (`/S /D=<disposable path>`) or Debian (`sudo dpkg -i <package>`) package. Do not downgrade existing test storage.
2. Disconnect **only the disposable VM's** virtual network adapter after prerequisites/install. Stop any development server. Launch the installed app and save two recognizable Unicode/quote records through the UI. Export through the native dialog to a fresh external test folder. Quit fully, preserve copies of the dedicated DB and any sidecars, and use `snapshot()` in `app/scripts/native-acceptance.py` on those copies. Require exact IDs/values/notes/timestamps/full migration history, integrity `ok`, no FK rows; preserve evidence outside the app directory.
3. Relaunch the same installed app, verify both records and save a distinct third row. Cancel the export dialog; cancel import selection; select the valid export and cancel replacement confirmation. Verify all three rows remain. Select again, explicitly confirm replacement, verify writes blocked, quit fully and reopen. Compare closed snapshots: live/export must match exactly; retained `restore-*/original.db` must match the distinct three-row baseline; pending marker must be absent. Reopen again and compare all fields/history.
4. Select a small non-SQLite `.mcbackup` through the UI and verify actionable rejection with unchanged data/history. Record screenshots, process exit evidence and PASS/FAIL/BLOCKED for each action. Preserve the isolated storage/evidence; do not delete databases to recover from failure. Reboot, physical power loss, filesystem faults, signing/upgrades and mobile remain separate existing acceptance procedures; this procedure authorizes none of those host operations.

### Current verification

Controller oracle/safety unit tests: **PASS (3 tests)** locally on macOS; these are not Windows/Linux packaged acceptance. Hosted run results will be recorded only after execution. Existing historical B6 evidence above remains unchanged. One local npm invocation from the repository root failed with missing `package.json`; the rerun from `app/` passed `npm run check` (14 Vitest), `npm run build`, `npm run rust:check`, `npm run rust:test` (25 passed, one ignored helper), rustfmt, all-target locked Clippy with warnings denied and `npm run desktop:build -- -- --locked` (6.22 MiB release macOS bundle, not launched). Bash syntax and `git diff --check` passed.
