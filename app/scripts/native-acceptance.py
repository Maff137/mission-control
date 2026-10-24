#!/usr/bin/env python3
"""macOS packaged debug acceptance controller; all artifacts are retained.

Requires a fresh com.missioncontrol.acceptance.* bundle built with native-acceptance.
Never launches the normal application or changes HOME/storage path resolution.
"""
import argparse
from contextlib import contextmanager, closing
import json
import os
from pathlib import Path
import plistlib
import shutil
import sqlite3
import subprocess
import tempfile
import time
import uuid


@contextmanager
def connection(*args, **kwargs):
    with closing(sqlite3.connect(*args, **kwargs)) as db:
        with db:
            yield db


def snapshot(path):
    # Inspect only a disposable copy after the controlled process has been reaped.
    # Include WAL left by SIGKILL; never assume the main file alone is current.
    with tempfile.TemporaryDirectory(prefix="mc-b5-inspection-") as temp:
        copied = Path(temp) / "snapshot.db"
        for suffix in ["", "-wal", "-shm", "-journal"]:
            source = Path(str(path) + suffix)
            if source.exists():
                shutil.copy2(source, Path(str(copied) + suffix))
        with connection(copied) as db:
            assert db.execute("PRAGMA integrity_check").fetchall() == [("ok",)]
            assert db.execute("PRAGMA foreign_key_check").fetchall() == []
            return {
                "records": db.execute("SELECT id,value,created_at,note FROM persistence_test_records ORDER BY id").fetchall(),
                "migrations": db.execute("SELECT version,description,installed_on,success,hex(checksum),execution_time FROM _sqlx_migrations ORDER BY version").fetchall(),
            }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bundle", type=Path)
    args = parser.parse_args()
    bundle = args.bundle.resolve(strict=True)
    with (bundle / "Contents/Info.plist").open("rb") as stream:
        info = plistlib.load(stream)
    identifier = info["CFBundleIdentifier"]
    assert identifier.startswith("com.missioncontrol.acceptance.")
    storage = Path.home() / "Library/Application Support" / identifier
    assert not storage.exists() and not storage.is_symlink(), "Use a fresh identifier"
    executable = bundle / "Contents/MacOS" / info["CFBundleExecutable"]
    artifacts = Path(tempfile.mkdtemp(prefix="mc-b5-native-"))
    print(f"ARTIFACTS={artifacts}", flush=True)
    sequence = 0

    def stop_at(boundary):
        nonlocal sequence
        sequence += 1
        log = artifacts / f"{sequence:02}-{boundary}.log"
        with log.open("w") as output:
            child = subprocess.Popen([str(executable)], stdout=output, stderr=subprocess.STDOUT,
                                     env={**os.environ, "MC_ACCEPTANCE_BOUNDARY": boundary})
            try:
                deadline = time.monotonic() + 25
                while time.monotonic() < deadline:
                    text = log.read_text()
                    if f"ACCEPTANCE_BOUNDARY={boundary}\n" in text:
                        assert f"ACCEPTANCE_STORAGE={storage}\n" in text
                        return child.pid
                    assert child.poll() is None, text
                    time.sleep(0.05)
                raise AssertionError(f"Checkpoint timeout: {log.read_text()}")
            finally:
                if child.poll() is None:
                    child.kill()  # Only the exact Popen Child owned by this controller.
                child.wait(timeout=10)

    def archive(name):
        if storage.exists():
            shutil.move(str(storage), artifacts / name)

    stop_at("after-ack")
    database = storage / "mission-control.db"
    with connection(database) as db:
        for value in ["original alpha 🚀", "original beta ' quoted", "original gamma"]:
            db.execute("INSERT INTO persistence_test_records(id,value,note) VALUES(?,?,?)",
                       (str(uuid.uuid4()), value, "original note"))
    original = snapshot(database)
    original_file = artifacts / "original.db"
    with connection(database) as source, connection(original_file) as target:
        source.backup(target)
    replacement_file = artifacts / "replacement.mcbackup"
    with connection(original_file) as source, connection(replacement_file) as target:
        source.backup(target)
        target.execute("DELETE FROM persistence_test_records")
        target.execute("INSERT INTO persistence_test_records(id,value,note) VALUES(?,?,?)",
                       (str(uuid.uuid4()), "replacement only Ω", "replacement note"))
        target.execute("PRAGMA application_id=1296253520")
        target.execute("PRAGMA user_version=1")
    replacement = snapshot(replacement_file)
    archive("initial-storage")
    results = []
    boundaries = ["before-snapshot", "after-snapshot", "original-displaced", "before-install",
                  "after-install", "before-sql", "before-ack", "after-ack"]
    for boundary in boundaries:
        storage.mkdir()
        shutil.copy2(original_file, database)
        work = storage / f"restore-{uuid.uuid4()}"
        work.mkdir()
        shutil.copy2(replacement_file, work / "incoming.db")
        (storage / "restore.pending").write_text(work.name)
        pid = stop_at(boundary)
        pending_before = (storage / "restore.pending").exists()
        installing_before = (work / "installing").exists()
        if pending_before:
            assert (storage / "restore.pending").read_text() == work.name
        if boundary != "before-snapshot":
            assert snapshot(work / "original.db") == original
        if boundary in ["original-displaced", "before-install"]:
            assert (work / "original-live.db").exists()
            assert not database.exists()
            assert (work / "candidate.db").exists()
        assert pending_before == (boundary != "after-ack")
        if boundary in ["original-displaced", "before-install", "after-install", "before-sql", "before-ack", "after-ack"]:
            assert installing_before
        stop_at("after-ack")
        expected = replacement if boundary in ["before-snapshot", "after-snapshot", "after-ack"] else original
        assert snapshot(database) == expected, boundary
        assert snapshot(work / "original.db") == original, boundary
        for retained in work.glob("previous-original-*.db"):
            assert snapshot(retained) == original, boundary
        assert not (storage / "restore.pending").exists()
        stop_at("after-ack")
        assert snapshot(database) == expected
        results.append({"boundary": boundary, "killed_pid": pid, "status": "PASS",
                        "pending_at_kill": pending_before, "installing_at_kill": installing_before,
                        "reopened_dataset": "replacement" if expected == replacement else "original",
                        "exact_records_and_history": True, "retained_original_exact": True,
                        "integrity": "ok", "foreign_keys": [], "reopens": 2})
        archive(boundary)
        print(f"PASS {boundary}", flush=True)
    # Leave a recognizable native dataset for manual UI/startup/WebView checks.
    storage.mkdir()
    shutil.copy2(original_file, database)
    (artifacts / "results.json").write_text(json.dumps(results, indent=2))
    print(json.dumps({"storage": str(storage), "artifacts": str(artifacts), "results": results}, indent=2))


if __name__ == "__main__":
    main()
