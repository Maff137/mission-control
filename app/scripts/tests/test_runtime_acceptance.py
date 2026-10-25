"""Regression tests for the acceptance oracle and its fail-closed safeguards."""
from contextlib import closing
import hashlib
import importlib.util
import json
from pathlib import Path
import sqlite3
import tempfile
import unittest
from unittest.mock import patch

SCRIPT = Path(__file__).resolve().parents[1] / "runtime-acceptance.py"
spec = importlib.util.spec_from_file_location("runtime", SCRIPT)
runtime = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runtime)


class RuntimeAcceptanceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="mc-runtime-oracle-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.database = self.root / "test.db"
        with closing(sqlite3.connect(self.database)) as db, db:
            db.execute("CREATE TABLE _sqlx_migrations (version BIGINT PRIMARY KEY, description TEXT NOT NULL, installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, success BOOLEAN NOT NULL, checksum BLOB NOT NULL, execution_time BIGINT NOT NULL)")
            for file in sorted((SCRIPT.parents[1] / "src-tauri/migrations").glob("*.sql")):
                db.executescript(file.read_text())
                description = "create_persistence_test_records" if file.name.startswith("0001_") else file.stem[5:]
                db.execute("INSERT INTO _sqlx_migrations(version,description,success,checksum,execution_time) VALUES(?,?,1,?,1)", (int(file.name[:4]), description, hashlib.sha384(file.read_bytes()).digest()))

    def test_exact_snapshot_backup_and_distinct_live_fixture(self):
        runtime.verify_schema(self.database)
        runtime.fixture(self.database)
        before = runtime.b5.snapshot(self.database)
        backup = self.root / "test.mcbackup"
        runtime.make_backup(self.database, backup)
        self.assertEqual(runtime.b5.snapshot(backup), before)
        with closing(sqlite3.connect(self.database)) as db, db:
            db.execute("UPDATE persistence_test_records SET note='changed'")
        self.assertNotEqual(runtime.b5.snapshot(self.database), before)

    def test_unknown_schema_and_wrong_checksum_rejected(self):
        with closing(sqlite3.connect(self.database)) as db, db:
            db.execute("UPDATE _sqlx_migrations SET checksum=x'00' WHERE version=1")
        with self.assertRaisesRegex(RuntimeError, "checksums"):
            runtime.verify_schema(self.database)
        with closing(sqlite3.connect(self.database)) as db, db:
            db.execute("CREATE TABLE unexpected (id INTEGER)")
        with self.assertRaisesRegex(RuntimeError, "schema objects"):
            runtime.verify_schema(self.database)

    def test_same_columns_without_required_constraint_are_rejected(self):
        with closing(sqlite3.connect(self.database)) as db, db:
            db.execute("DROP TABLE persistence_test_records")
            db.execute("CREATE TABLE persistence_test_records (id TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL, created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')), note TEXT NOT NULL DEFAULT '')")
        with self.assertRaisesRegex(RuntimeError, "schema constraints"):
            runtime.verify_schema(self.database)

    def test_prepare_blocked_and_refuse_reuse_or_unsafe_identity(self):
        target = self.root / "evidence"
        runtime.prepare(target)
        report = json.loads((target / "results.json").read_text())
        self.assertEqual(report["results"][0]["status"], "BLOCKED")
        with self.assertRaises(FileExistsError):
            runtime.prepare(target)
        with self.assertRaisesRegex(RuntimeError, "Unsafe identity"):
            runtime.storage_path("com.missioncontrol.desktop")
        with self.assertRaisesRegex(RuntimeError, "Unsafe identity"):
            runtime.storage_path("com.missioncontrol.acceptance.ci/../../desktop")
        existing = self.root / "existing"
        existing.mkdir()
        with patch.object(runtime, "storage_path", return_value=existing):
            acceptance = runtime.Acceptance(target)
            acceptance.result("package installation", "PASS", "synthetic unit fixture")
            self.assertEqual(json.loads((target / "results.json").read_text())["overall"], "BLOCKED")
            with patch.object(acceptance, "install") as install:
                with self.assertRaisesRegex(RuntimeError, "existing native storage"):
                    acceptance.run()
                install.assert_not_called()


if __name__ == "__main__":
    unittest.main()
