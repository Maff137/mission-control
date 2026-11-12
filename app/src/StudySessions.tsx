import { useEffect, useRef, useState, type FormEvent } from "react";
import { nativeError } from "./Backup";
import Backup from "./Backup";
import {
  localStudyDate,
  displayStudyDate,
  nextAreaColor,
  studyPersistence,
  validateAreaName,
  validateSession,
  type StudyArea,
  type StudySession,
} from "./studyPersistence";

export default function StudySessions() {
  const [areas, setAreas] = useState<StudyArea[]>([]);
  const [sessions, setSessions] = useState<StudySession[]>([]);
  const [areaId, setAreaId] = useState("");
  const [areaName, setAreaName] = useState("");
  const [creatingArea, setCreatingArea] = useState(false);
  const [duration, setDuration] = useState("");
  const [studyDate, setStudyDate] = useState(localStudyDate);
  const [notes, setNotes] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [blocked, setBlocked] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const writing = useRef(false);

  async function refresh() {
    try {
      const [nextAreas, nextSessions] = await Promise.all([
        studyPersistence.listAreas(),
        studyPersistence.listSessions(),
      ]);
      setAreas(nextAreas);
      setSessions(nextSessions);
      setAreaId((current) =>
        nextAreas.some((area) => area.id === current) ? current : (nextAreas[0]?.id ?? ""),
      );
      setError("");
    } catch (cause) {
      setError(`Could not load study data: ${nativeError(cause)}`);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
  }, []);

  const duplicateName = areaName.trim().length > 0 && areas.some(
    (area) => area.name.toLocaleLowerCase() === areaName.trim().toLocaleLowerCase(),
  );

  async function addArea(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (blocked || loading || writing.current) return;
    const validation = validateAreaName(areaName);
    if (validation) { setError(validation); return; }
    writing.current = true;
    setSaving(true);
    setError("");
    setNotice("");
    try {
      const createdId = await studyPersistence.createArea(areaName, nextAreaColor(areas.length));
      await refresh();
      setAreaId(createdId);
      setAreaName("");
      setCreatingArea(false);
      setNotice("Study area created.");
    } catch (cause) {
      setError(`Could not create study area: ${nativeError(cause)}`);
    } finally {
      writing.current = false;
      setSaving(false);
    }
  }

  async function saveSession(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (blocked || loading || writing.current) return;
    const input = {
      areaId,
      durationMinutes: Number(duration),
      studyDate,
      notes,
    };
    const validation = validateSession(input);
    if (validation || !duration.trim()) {
      setError(validation ?? "Enter the study duration.");
      return;
    }
    writing.current = true;
    setSaving(true);
    setError("");
    setNotice("");
    try {
      await studyPersistence.createSession(input);
      setDuration("");
      setNotes("");
      setNotice("Study session saved.");
      await refresh();
    } catch (cause) {
      setError(`Could not save study session: ${nativeError(cause)}`);
    } finally {
      writing.current = false;
      setSaving(false);
    }
  }

  return (
    <>
      <div className="study-layout">
        <section className="panel" aria-labelledby="new-session-heading" aria-busy={loading || saving}>
          <p className="panel-kicker">MANUAL ENTRY</p>
          <h2 id="new-session-heading">New study session</h2>
          <p className="panel-intro">Record completed study time whenever it suits you.</p>
          <form onSubmit={saveSession} className="form-stack">
            <div className="field">
              <label htmlFor="study-area">Study area</label>
              <div className="area-picker">
                <select
                  id="study-area"
                  value={areaId}
                  disabled={blocked || loading || saving || areas.length === 0}
                  onChange={(event) => setAreaId(event.target.value)}
                  required
                >
                  {areas.length === 0 && <option value="">No areas yet</option>}
                  {areas.map((area) => (
                    <option key={area.id} value={area.id}>{area.name}</option>
                  ))}
                </select>
                <button
                  type="button"
                  className="secondary-button add-area-button"
                  aria-label={creatingArea ? "Close new study area" : "Add study area"}
                  aria-expanded={creatingArea}
                  onClick={() => setCreatingArea((previous) => !previous)}
                  disabled={blocked || loading || saving}
                >
                  {creatingArea ? "Cancel" : "+ Add"}
                </button>
              </div>
            </div>
            <div className="form-pair">
              <div className="field">
                <label htmlFor="study-duration">Duration (minutes)</label>
                <input
                  id="study-duration"
                  type="number"
                  inputMode="numeric"
                  min="1"
                  step="1"
                  placeholder="45"
                  value={duration}
                  onChange={(event) => setDuration(event.target.value)}
                  disabled={blocked || loading || saving}
                  required
                />
              </div>
              <div className="field">
                <label htmlFor="study-date">Study date</label>
                <input
                  id="study-date"
                  type="date"
                  value={studyDate}
                  onChange={(event) => setStudyDate(event.target.value)}
                  disabled={blocked || loading || saving}
                  required
                />
              </div>
            </div>
            <div className="field">
              <label htmlFor="study-notes">Notes <span className="muted">(optional)</span></label>
              <textarea
                id="study-notes"
                value={notes}
                onChange={(event) => setNotes(event.target.value)}
                placeholder="What did you study? Add any details you want."
                rows={4}
                maxLength={10000}
                disabled={blocked || loading || saving}
              />
            </div>
            <button type="submit" disabled={blocked || loading || saving || areas.length === 0}>
              {saving ? "Saving…" : "Save session"}
            </button>
          </form>
          {creatingArea && (
            <form onSubmit={addArea} className="quick-area" aria-label="Quick create study area">
              <div className="field">
                <label htmlFor="new-area-name">New study area</label>
                <input
                  id="new-area-name"
                  value={areaName}
                  onChange={(event) => setAreaName(event.target.value)}
                  maxLength={80}
                  placeholder="e.g. Programming"
                  disabled={blocked || saving}
                  required
                />
              </div>
              {duplicateName && (
                <p className="field-warning" role="status">
                  An area with this name already exists. You can still create another.
                </p>
              )}
              <p className="muted">A color will be assigned automatically.</p>
              <button type="submit" className="secondary-button" disabled={blocked || saving || !areaName.trim()}>
                Create area
              </button>
            </form>
          )}
          {loading && <p role="status">Loading study data…</p>}
          {notice && <p className="success-message" role="status">{notice}</p>}
          {error && (
            <div role="alert" className="error-message">
              <p>{error}</p>
              <button type="button" className="secondary-button" onClick={() => void refresh()} disabled={saving}>
                Retry loading
              </button>
            </div>
          )}
          {blocked && <p role="status">Restore pending. Reopen the application before saving.</p>}
        </section>
        <section className="panel history-panel" aria-labelledby="study-history-heading">
          <p className="panel-kicker">YOUR PROGRESS</p>
          <h2 id="study-history-heading">Study history</h2>
          <p className="panel-intro">Saved locally on this device.</p>
          {!loading && sessions.length === 0 && <p className="empty-state">No sessions yet. Save your first study session to see it here.</p>}
          <div className="session-cards">
            {sessions.map((session) => (
              <article className="session-card" key={session.id}>
                <span className="area-stripe" style={{ backgroundColor: session.area_color }} aria-hidden="true" />
                <div className="session-details">
                  <div className="session-topline">
                    <h3>{session.area_name}</h3>
                    <strong>{session.duration_minutes} min</strong>
                  </div>
                  <time dateTime={session.study_date}>{displayStudyDate(session.study_date)}</time>
                  {session.notes && <p className="session-notes">{session.notes}</p>}
                </div>
              </article>
            ))}
          </div>
        </section>
      </div>
      <div className="backup-panel">
        <Backup disabled={loading || saving} onBlock={setBlocked} />
      </div>
    </>
  );
}
