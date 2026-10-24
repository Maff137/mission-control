import { useEffect, useRef, useState, type FormEvent } from "react";
import { persistence, type TestRecord } from "./persistence";
import "./App.css";
import Backup, { nativeError } from "./Backup";

export default function App() {
  const [records, setRecords] = useState<TestRecord[]>([]);
  const [value, setValue] = useState("");
  const [loading, setLoading] = useState(true);
  const [blocked, setBlocked] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");
  const pending = useRef(false);

  async function refresh() {
    setLoading(true);
    setError("");
    try {
      setRecords(await persistence.readRecords());
    } catch (cause) {
      setError(`Could not load records: ${nativeError(cause)}`);
    } finally {
      setLoading(false);
    }
  }
  useEffect(() => { void refresh(); }, []);

  async function save(event: FormEvent) {
    event.preventDefault();
    if (blocked || pending.current || !value.trim()) return;
    pending.current = true;
    setSaving(true);
    setError("");
    setSuccess("");
    try {
      await persistence.insertRecord(value);
      setValue("");
      setSuccess("Record saved to SQLite.");
      await refresh();
    } catch (cause) {
      setError(`Could not save record: ${nativeError(cause)}`);
    } finally {
      pending.current = false;
      setSaving(false);
    }
  }
  return (
    <main className="welcome">
      <p className="eyebrow">Your study companion</p>
      <h1>Mission Control</h1>
      <p>A focused space for your study journey.</p>
      <section aria-labelledby="persistence-heading" aria-busy={loading || saving}>
        <h2 id="persistence-heading">Persistence test</h2>
        <p>Temporary native SQLite validation interface.</p>
        <form onSubmit={save}>
          <label htmlFor="record">Test record</label>
          <div className="record-input">
            <input id="record" value={value} onChange={(event) => setValue(event.target.value)} disabled={saving || loading || blocked} />
            <button disabled={loading || saving || blocked || !value.trim()}>{saving ? "Saving…" : "Save"}</button>
          </div>
        </form>
        {loading && <p role="status">Loading records…</p>}
        {success && <p role="status">{success}</p>}
        {error && <div role="alert"><p>{error}</p><button type="button" disabled={loading || saving || blocked} onClick={() => void refresh()}>Retry loading</button></div>}
        {!loading && !error && records.length === 0 && <p>No saved records yet.</p>}
        <ul>{records.map((record) => <li key={record.id}><span>{record.value}</span><time dateTime={record.created_at}>{record.created_at}</time></li>)}</ul>
      </section>
      <Backup disabled={loading || saving} onBlock={setBlocked} />
    </main>
  );
}
