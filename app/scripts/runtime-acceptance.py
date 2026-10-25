#!/usr/bin/env python3
"""Installed release acceptance. Owns only fresh acceptance identities and children.

No application instrumentation, Vite, third-party Python modules or real user data.
Database fixtures are explicitly NOT UI writes or application-generated exports.
"""
import argparse
from contextlib import closing
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import shutil
import socket
import sqlite3
import subprocess
import sys
import time
import uuid

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("b5", HERE / "native-acceptance.py")
b5 = importlib.util.module_from_spec(spec)
spec.loader.exec_module(b5)


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def storage_path(identifier):
    require(identifier.startswith("com.missioncontrol.acceptance.ci"), "Unsafe identity")
    if sys.platform == "win32":
        return Path(os.environ["APPDATA"]) / identifier
    require(sys.platform == "linux", "Only Windows/Linux hosted VMs are supported")
    return Path(os.environ.get("XDG_CONFIG_HOME", Path.home() / ".config")) / identifier


def verify_schema(database):
    with closing(sqlite3.connect(database)) as db, closing(sqlite3.connect(":memory:")) as expected:
        # Reconstruct the exact SQLx schema plus the two immutable app migrations.
        expected.execute("CREATE TABLE _sqlx_migrations (version BIGINT PRIMARY KEY, description TEXT NOT NULL, installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, success BOOLEAN NOT NULL, checksum BLOB NOT NULL, execution_time BIGINT NOT NULL)")
        files = sorted((HERE.parent / "src-tauri/migrations").glob("*.sql"))
        for file in files:
            expected.executescript(file.read_text())
        # SQLx whitespace is not significant; PRAGMA columns and schema object names are.
        objects = "SELECT type,name,tbl_name FROM sqlite_schema ORDER BY type,name"
        require(db.execute(objects).fetchall() == expected.execute(objects).fetchall(), "Unexpected schema objects")
        for table in ["persistence_test_records", "_sqlx_migrations"]:
            require(db.execute(f"PRAGMA table_info({table})").fetchall() == expected.execute(f"PRAGMA table_info({table})").fetchall(), f"Wrong columns: {table}")
            sql = "SELECT sql FROM sqlite_schema WHERE type='table' AND name=?"
            normalize = lambda text: "".join(text.split()).lower()
            require(normalize(db.execute(sql, (table,)).fetchone()[0]) == normalize(expected.execute(sql, (table,)).fetchone()[0]), f"Wrong schema constraints: {table}")
        history = db.execute("SELECT version,description,success,checksum FROM _sqlx_migrations ORDER BY version").fetchall()
        wanted = [(int(file.name[:4]), ("create_persistence_test_records" if file.name.startswith("0001_") else file.stem[5:]), 1, hashlib.sha384(file.read_bytes()).digest()) for file in files]
        require(history == wanted, "Wrong migration versions/descriptions/checksums/success")


def fixture(database):
    with closing(sqlite3.connect(database)) as db, db:
        for index, value in enumerate(["CI alpha 🚀", "CI beta ' quoted Ω"]):
            db.execute("INSERT INTO persistence_test_records(id,value,created_at,note) VALUES(?,?,?,?)", (str(uuid.uuid4()), value, f"2026-10-25T12:00:0{index}.123Z", f"note {index}"))


def make_backup(database, target):
    with closing(sqlite3.connect(database)) as source, closing(sqlite3.connect(target)) as destination:
        source.backup(destination)
        destination.execute("PRAGMA application_id=1296253520")
        destination.execute("PRAGMA user_version=1")
        destination.commit()


class Acceptance:
    def __init__(self, root):
        self.root = root.resolve()
        self.manifest = json.loads((self.root / "manifest.json").read_text())
        self.storage = storage_path(self.manifest["identifier"])
        self.database = self.storage / "mission-control.db"
        self.rows = []
        self.child = None
        self.counter = 0
        self.executable = None

    def result(self, name, status, detail):
        self.rows.append({"scenario": name, "status": status, "detail": detail})
        self.publish()
        print(f"{status}: {name}: {detail}", flush=True)

    def publish(self):
        overall = "FAIL" if any(row["status"] == "FAIL" for row in self.rows) else "BLOCKED" if any(row["status"] == "BLOCKED" for row in self.rows) else "PASS"
        data = {"overall": overall, **self.manifest, "platform": platform.platform(), "storage": str(self.storage), "executable": str(self.executable), "results": self.rows}
        (self.root / "results.json").write_text(json.dumps(data, indent=2, ensure_ascii=False), encoding="utf-8")
        summary = f"Packaged acceptance overall: **{overall}** (see individual scopes).\n\n" + "| Scenario | Result | Evidence / limit |\n|---|---|---|\n" + "".join(f"| {row['scenario']} | {row['status']} | {row['detail']} |\n" for row in self.rows)
        (self.root / "summary.md").write_text(summary, encoding="utf-8")

    def command(self, args, timeout=120):
        result = subprocess.run(args, capture_output=True, text=True, timeout=timeout)
        self.counter += 1
        (self.root / f"command-{self.counter}.log").write_text(result.stdout + result.stderr, encoding="utf-8")
        require(result.returncode == 0, f"Command failed ({result.returncode}): {args}; see command-{self.counter}.log")
        return result.stdout.strip()

    def install(self):
        bundle = HERE.parent / "src-tauri/target/release/bundle"
        candidates = list(bundle.glob("nsis/*-setup.exe" if sys.platform == "win32" else "deb/*.deb"))
        require(len(candidates) == 1, "Expected exactly one freshly built release package")
        package = candidates[0].resolve()
        self.manifest["package_sha256"] = hashlib.sha256(package.read_bytes()).hexdigest()
        self.manifest["package"] = package.name
        if sys.platform == "win32":
            destination = self.root / "installed"
            self.command([str(package), "/S", f"/D={destination}"], timeout=300)
            self.executable = destination / "mission-control.exe"
        else:
            self.command(["sudo", "dpkg", "-i", str(package)])
            listing = self.command(["dpkg-deb", "-c", str(package)])
            require("./usr/bin/mission-control" in listing, "Unexpected Debian executable")
            self.executable = Path("/usr/bin/mission-control")
        require(self.executable.is_file(), "Installed executable missing")
        self.result("package installation", "PASS", f"Installed {package.name}; SHA-256 in results.json; isolated identifier {self.manifest['identifier']}")

    def start(self):
        require(self.child is None, "Child still owned")
        # A real listener would invalidate the no-Vite acceptance condition.
        with socket.socket() as probe:
            require(probe.connect_ex(("127.0.0.1", 1420)) != 0, "Port 1420 is listening")
        self.counter += 1
        log = (self.root / f"launch-{self.counter}.log").open("wb")
        try:
            self.child = subprocess.Popen([str(self.executable)], stdout=log, stderr=subprocess.STDOUT)
        finally:
            log.close()
        deadline = time.monotonic() + 45
        while time.monotonic() < deadline:
            require(self.child.poll() is None, "Installed application exited during startup")
            if self.database.exists():
                try:
                    # Probe only the dedicated DB; no mutations while the application runs.
                    with closing(sqlite3.connect(f"{self.database.as_uri()}?mode=ro", uri=True, timeout=0.2)) as db:
                        if db.execute("SELECT count(*) FROM _sqlx_migrations WHERE success=1").fetchone() == (2,) and not (self.storage / "restore.pending").exists():
                            time.sleep(2)
                            require(self.child.poll() is None, "App exited after SQL initialization")
                            return
                except sqlite3.Error:
                    pass
            time.sleep(0.2)
        raise RuntimeError("Timed out waiting for native SQL initialization/restore acknowledgement")

    def stop(self):
        if self.child is not None:
            child, self.child = self.child, None
            if child.poll() is None:
                child.kill()  # Only this controller's exact Popen child; never process-name kills.
            child.wait(timeout=15)

    def snapshot(self, label):
        require(self.child is None, "Must reap app before copying DB/sidecars")
        data = b5.snapshot(self.database)
        (self.root / f"{label}.json").write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")
        verify_schema(self.database)
        return data

    def linux_ui(self):
        self.start()
        window = self.command(["xdotool", "search", "--sync", "--onlyvisible", "--pid", str(self.child.pid), "--name", "Mission Control"], timeout=30).splitlines()[0]
        self.command(["xdotool", "windowfocus", "--sync", window])
        self.command(["xdotool", "key", "--clearmodifiers", "Tab"])
        self.command(["xdotool", "type", "--clearmodifiers", "--delay", "40", "CI UI save through bundled WebView"])
        self.command(["xdotool", "key", "Return"])
        time.sleep(3)
        self.command(["import", "-window", window, str(self.root / "linux-ui.png")])
        # After submission the input remains focused; Save is disabled when empty.
        self.command(["xdotool", "key", "Tab", "Return"])
        time.sleep(3)
        self.command(["xdotool", "key", "ctrl+l"])
        self.command(["xdotool", "type", "--clearmodifiers", str(self.root / "ui-export.mcbackup")])
        self.command(["xdotool", "key", "Return"])
        time.sleep(2)
        self.command(["xdotool", "key", "Return"])
        time.sleep(3)
        self.stop()
        actual = self.snapshot("ui-save")
        require(sum(row[1] == "CI UI save through bundled WebView" for row in actual["records"]) == 1, "UI Save did not commit exactly one record")
        self.result("Linux bundled UI save", "PASS", "Xvfb/xdotool keyboard submission committed exactly one row; screenshot retained")
        exported = self.root / "ui-export.mcbackup"
        require(exported.is_file(), "Native UI export did not produce a backup")
        require(b5.snapshot(exported) == actual, "UI export records/history differ")
        with closing(sqlite3.connect(exported)) as db:
            require(db.execute("PRAGMA application_id").fetchone() == (1296253520,), "Export format identity missing")
            require(db.execute("PRAGMA user_version").fetchone() == (1,), "Export format version missing")
        self.result("Linux native UI export", "PASS", "Native Save dialog produced format-1 snapshot with exact records/full history; integrity/FK passed")
        return actual, exported

    def run(self):
        require(not self.storage.exists() and not self.storage.is_symlink(), "Refusing existing native storage")
        self.install()
        self.start()
        self.stop()
        initial = self.snapshot("initialized")
        require(initial["records"] == [], "Fresh database was not empty")
        self.result("native initialization/schema", "PASS", "Installed release created empty native SQLite; exact schema columns/objects and SHA-384 migration checksums; integrity/FK passed")
        fixture(self.database)
        baseline = self.snapshot("fixture")
        self.start()
        self.stop()
        require(self.snapshot("reopened") == baseline, "Exact fixture changed after relaunch")
        self.result("process termination/relaunch", "PASS", "Two synthetic SQLite fixture records preserved exactly, including IDs/Unicode/quotes/notes/timestamps and all migration fields; not UI inserts")
        if sys.platform == "linux":
            baseline, selected = self.linux_ui()
        else:
            # Probe actual runner desktop rather than asserting all hosted runners use Session 0.
            desktop = self.command(["powershell", "-NoProfile", "-Command", "Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public class DesktopProbe { [DllImport(\"user32.dll\")] public static extern IntPtr OpenInputDesktop(uint f, bool i, uint a); [DllImport(\"user32.dll\")] public static extern bool CloseDesktop(IntPtr h); }'; $h=[DesktopProbe]::OpenInputDesktop(0,$false,1); Write-Output ([Environment]::UserInteractive); Write-Output ([System.Diagnostics.Process]::GetCurrentProcess().SessionId); Write-Output ($h -ne [IntPtr]::Zero); if ($h -ne [IntPtr]::Zero) { [void][DesktopProbe]::CloseDesktop($h) }"])
            self.result("Windows UI/dialog acceptance", "BLOCKED", f"Desktop probe UserInteractive/session/input-desktop: {desktop.replace(chr(10), ', ')}. No reliable Windows WebView/dialog driver configured; process/SQLite success does not verify rendering or UI actions")
            selected = self.root / "fixture.mcbackup"
            make_backup(self.database, selected)
            self.result("Windows application UI export", "BLOCKED", "Restore input is a Python SQLite fixture, not an app-generated export; native service export is covered separately by existing Rust tests")
        # Distinct live data proves replacement instead of a vacuous unchanged comparison.
        with closing(sqlite3.connect(self.database)) as db, db:
            db.execute("INSERT INTO persistence_test_records(id,value,note) VALUES(?,?,?)", (str(uuid.uuid4()), "live-only row must be retained in original", "pre-restore"))
        original = self.snapshot("before-restore")
        work = self.storage / f"restore-{uuid.uuid4()}"
        work.mkdir()
        shutil.copy2(selected, work / "incoming.db")
        (self.storage / "restore.pending").write_text(work.name)
        self.start()
        self.stop()
        require(self.snapshot("after-restore") == baseline, "Native startup did not restore exact selected records/history")
        require(b5.snapshot(work / "original.db") == original, "Retained original differs")
        require(not (self.storage / "restore.pending").exists(), "Pending marker not acknowledged")
        self.start()
        self.stop()
        require(self.snapshot("restore-reopened") == baseline, "Restored records/history changed on second reopen")
        self.result("packaged native startup restore", "PASS", "Controller-constructed confirmation fixture; actual app restored exact backup, retained distinct original, cleared pending marker; second reopen/integrity/FK passed. UI selection/confirmation NOT tested")
        self.result("UI restore confirmation/cancel/invalid import", "BLOCKED", "Manual VM procedure required; startup fixture acceptance is not native dialog/confirmation acceptance")
        self.result("bundled runtime without Vite", "PASS", "Installed embedded release assets; port 1420 absent on every launch; no Vite/Node/server started. Network isolation is reported separately")
        if sys.platform == "linux":
            interfaces = sorted(link["ifname"] for link in json.loads(self.command(["ip", "-j", "link"])))
            require(os.environ.get("MC_NETWORK_ISOLATED") == "1" and interfaces == ["lo"], "Linux acceptance requires an isolated network namespace")
            self.result("disconnected Linux runtime", "PASS", "Controller and every app launch ran as the original unprivileged user in a separate network namespace with only loopback; X11/DBus use Unix sockets; host network unchanged")
        else:
            self.result("disconnected Windows runtime", "BLOCKED", "Network interfaces remained enabled; no host firewall/security changes attempted")
        self.result("reboot/power-loss/distribution", "BLOCKED", "No host reboot, physical power cut, signed delivery, clean prerequisite-free installation or update acceptance performed")


def prepare(root):
    root.mkdir(parents=True, exist_ok=False)
    identifier = "com.missioncontrol.acceptance.ci" + uuid.uuid4().hex
    manifest = {"identifier": identifier, "commit": os.environ.get("GITHUB_SHA", "local"), "run_url": f"https://github.com/{os.environ.get('GITHUB_REPOSITORY', 'Maff137/mission-control')}/actions/runs/{os.environ.get('GITHUB_RUN_ID', 'local')}"}
    (root / "manifest.json").write_text(json.dumps(manifest, indent=2))
    (root / "config.json").write_text(json.dumps({"identifier": identifier, "productName": "Mission Control CI Acceptance"}))
    # Evidence survives failed prerequisites; never infer acceptance from build success.
    (root / "results.json").write_text(json.dumps({**manifest, "results": [{"scenario": "packaged runtime", "status": "BLOCKED", "detail": "Build/install/runtime has not completed"}]}, indent=2))
    (root / "summary.md").write_text("Packaged native runtime: **BLOCKED** until build/install/controller completes.\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["prepare", "run"])
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    if args.mode == "prepare":
        prepare(args.root)
        return
    acceptance = Acceptance(args.root)
    try:
        acceptance.run()
    except Exception as error:
        acceptance.result("runtime controller", "FAIL", str(error))
        raise
    finally:
        acceptance.stop()


if __name__ == "__main__":
    main()
