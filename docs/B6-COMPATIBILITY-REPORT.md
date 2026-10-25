# B6 — Cross-platform compatibility and Stage 2 acceptance

Verification date: 2026-10-25. **PARTIALLY VERIFIED**. Stage 2: **IN PROGRESS**.

## Scope and environment

Base: fetched `origin/main` at `6fd860c06c9076ed3c3316926ba8f707647c8b95`; clean initial workspace; branch `test/b6-platform-compatibility`. macOS 27.0.1 (26A434), ARM64; Node 24.21.0, npm 11.19.0, rustc/cargo 1.99.1. Only installed Rust target: `aarch64-apple-darwin`; Command Line Tools at `/Library/Developer/CommandLineTools`; full Xcode absent. `adb` and `sdkmanager` absent from PATH; ANDROID_HOME/ANDROID_NDK_HOME/JAVA_HOME unset. No heavyweight SDK installation or mobile initialization attempted.

Locked Tauri 2.12.5, CLI/API 2.12.4, tauri-build 2.7.4, SQL plugin 2.6.0, dialog 2.8.3, SQLx 0.8.6, SQLite sys 0.30.1. Existing lockfiles and migrations unchanged. Engineering feasibility does not establish officially supported OS versions or distribution readiness.

## Platform matrix (evidence collection in progress)

| Platform | Static inspection | Compilation | Packaging | Native launch / SQLite / backup | Device restart | Distribution |
|---|---|---|---|---|---|---|
| macOS ARM64 | PASS | PASS, initial checks | NOT TESTED in B6 yet | Prior B5 PASS within recorded scope; B6 pending | NOT TESTED | NOT TESTED signed distribution |
| Windows x64 | PARTIALLY VERIFIED | NOT TESTED; CI prepared | NOT TESTED; NSIS configured | NOT TESTED | NOT TESTED | NOT TESTED |
| Linux x64 | PARTIALLY VERIFIED | NOT TESTED; CI prepared | NOT TESTED; Debian configured | NOT TESTED | NOT TESTED | NOT TESTED |
| Android ARM64 | PARTIALLY VERIFIED | BLOCKED: SDK/NDK/target missing | BLOCKED | NOT TESTED | NOT TESTED | NOT TESTED |
| iOS ARM64 | PARTIALLY VERIFIED | BLOCKED: full Xcode/target missing | BLOCKED | NOT TESTED | NOT TESTED | NOT TESTED |

## Changes and validation

Platform-specific `tauri.windows.conf.json` / `tauri.linux.conf.json` overlays select NSIS with embedded WebView2 offline installer and Debian packaging. The base remains `.app`. `desktop:build` delegates target selection to Tauri instead of overriding all hosts with `--bundles app`. No identity, capability, CSP, migration or persistence engine change. Workflow uses Windows 2022 and Ubuntu 24.04 x64 runners, fixed Node/Rust, `npm ci`, locked Cargo checks/tests/Clippy/build, no signing/release/upload and read-only contents permissions. CI does not launch the app. Test subprocesses exercise storage services, not packaged WebViews.

Initial local checks PASS: `npm run check` (13 Vitest), `npm run build`, `npm run rust:check`, `npm run rust:test` (24 passed, one ignored helper), `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings`. Final-code results and native evidence will follow. An initial final-suite invocation from repository root failed with missing package.json; rerun from app. Sandbox denied initial fetch/GitHub auth/network metadata checks; approved retries succeeded. No failure counted as PASS.

## Storage and mobile inspection

Native Store and SQL plugin both resolve `app_config_dir`: macOS Library/Application Support/identifier, Windows Roaming AppData/identifier and Linux XDG_CONFIG_HOME/identifier (normally ~/.config/identifier). These are static resolutions outside macOS until native evidence exists. There is no browser-storage fallback. File locks use Rust `File::try_lock`; SQL connections close before replacement. Same-filesystem rename and no-clobber hard links underpin recovery/publication. Directory sync executes only under `cfg(unix)`; Windows returns success without directory flush, so power-loss directory-entry durability is unverified. Unix sync success also does not establish physical power-loss durability. Windows privacy relies on inherited ACLs; mode 0600/0700 is Unix-only. Network filesystems, removable media and filesystems without hard links are unsupported/unverified; safe errors must be checked per filesystem. No alternative unsafe copy/overwrite fallback is introduced.

Mobile entry point exists but generated mobile projects do not. SQL and dialog provide mobile implementations, but framework support does not validate this app. `choose()` converts FilePath to a local PathBuf; Android content:// providers cannot pass `into_path()`, and export requires hard links unavailable in document-provider destinations. Mobile sandbox/file-provider access, export semantics, locking, restore and suspension remain unverified. Desktop backup semantics must not be advertised as mobile support.

## Remaining acceptance procedures

### M1 — NOT TESTED

Use a fresh `com.missioncontrol.acceptance.*` identifier and disposable packaged app. Confirm Info.plist identity and actual native storage signal. Launch without Vite, save recognizable Unicode/quote records, export a backup, quit fully, copy closed DB plus any sidecars into retained evidence and snapshot every record/history field with the B5 controller helper. Record OS/filesystem/build/identifier, wall-clock time and baseline integrity/FK. User independently restarts the device at a convenient time; never automate reboot. Reopen the exact bundle offline, inspect records, quit, snapshot copies again and require exact IDs/values/notes/timestamps/full migration equality plus integrity ok and empty FK check. Repeat startup and record operator/reboot evidence. Include confirmed restore followed by reboot and retained-original comparison for restart recovery coverage. A quit/reopen or SIGKILL is insufficient.

### M2 — NOT TESTED

No disposable authorized physical power-loss environment is available. Require a sacrificial device or explicitly isolated storage/VM environment whose power-cut and flush/cache model is documented. VM process kill alone does not prove physical durability. Agree authorized boundaries and expected guarantees before testing committed writes, export publication, durable confirmation, snapshot/displacement/install/acknowledgement. Capture pre-cut complete snapshots, cut only disposable environment power, boot/recover and compare records/history, pending markers, original copies and integrity/FK after multiple reopens. Record filesystem, mount options, virtual/physical caches and hardware guarantees. No host power interruption, real-filesystem corruption or device restart performed. Proposed risk acceptance is OPEN and requires explicit user approval; successful fsync/SIGKILL is not approval or evidence.

### M5 — PARTIALLY VERIFIED

Run the compatibility workflow and retain run URL/commit/job results. On actual Windows NTFS and Linux local filesystem, install only isolated unsigned test packages under fresh identifiers, launch offline and perform the B4 UI export/restore/cancel/invalid-input and B5 startup/locking/reopen checks. Compare complete snapshots and retained originals; test non-ASCII paths, existing destinations, permissions/capacity, hard-link failure, and filesystem-specific rename/open-handle semantics. Inspect actual native paths and WebView independence. CI compilation and Rust tests cannot replace these native checks. Signed install/update preservation, clean offline Windows installation without WebView2, macOS signing/notarization/Gatekeeper, Linux minimum glibc/WebKitGTK matrix, Android SDK/device/provider and iOS Xcode/device/signing remain pending.

## Stage 2 recommendation

Keep B5 and B6 PARTIALLY VERIFIED and Stage 2 IN PROGRESS. Critical missing evidence is independent M1 reboot, M2 genuine isolated power-loss durability (or explicitly approved scope/risk decision), and M5 native platform/filesystem/distribution acceptance or explicit approval of a narrower support policy. No such decision is approved here. Historical B1–B5 evidence remains valid only within its recorded scope.

## Primary references inspected

- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/): native desktop dependencies and mobile toolchains.
- [Windows installers](https://v2.tauri.app/distribute/windows-installer/): NSIS/MSI and offline WebView2 options.
- [Dialog plugin](https://v2.tauri.app/plugin/dialog/): platform-specific picker behavior.
- Locked local crate sources: tauri path resolvers, SQL wrapper, dialog mobile implementation and FilePath::into_path; these establish current implementation details rather than newer documentation alone.
