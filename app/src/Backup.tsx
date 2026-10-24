import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
export function nativeError(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error) return String(error.message);
  return String(error);
}
export default function Backup({ disabled, onBlock }: { disabled: boolean; onBlock: (blocked: boolean) => void }) {
  const [busy, setBusy] = useState(false);
  const [confirmation, setConfirmation] = useState(false);
  const [confirmed, setConfirmed] = useState(false);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const running = useRef(false);
  useEffect(() => { void invoke<string>("backup_status").then(setMessage).catch((cause: unknown) => setError(nativeError(cause))); }, []);
  async function run(action: () => Promise<void>) {
    if (running.current) return;
    running.current = true; setBusy(true); setError(""); setMessage(""); onBlock(true);
    try { await action(); }
    catch (cause) { setError(nativeError(cause)); }
    finally { running.current = false; setBusy(false); }
  }
  async function exportFile() {
    await run(async () => { if (await invoke<boolean>("export_backup")) setMessage("Backup exported and validated successfully."); });
    onBlock(false);
  }
  async function selectFile() {
    let selected = false;
    await run(async () => { selected = await invoke<boolean>("stage_restore"); setConfirmation(selected); });
    if (!selected) onBlock(false);
  }
  async function cancel() {
    await run(async () => { await invoke("cancel_restore"); setConfirmation(false); });
    onBlock(false);
  }
  async function confirm() {
    await run(async () => {
      await invoke("confirm_restore"); setConfirmation(false); setConfirmed(true);
      setMessage("Restore is confirmed. Quit Mission Control completely, then reopen it to apply the backup. Writes are blocked until restart.");
    });
  }
  return <section aria-labelledby="backup-heading" aria-busy={busy}>
    <h2 id="backup-heading">Backup and restore</h2>
    <p>Export a portable .mcbackup snapshot. Restore replaces all current data after a full application restart.</p>
    <button type="button" disabled={disabled || busy || confirmation || confirmed} onClick={() => void exportFile()}>Export Backup</button>{" "}
    <button type="button" disabled={disabled || busy || confirmation || confirmed} onClick={() => void selectFile()}>Restore Backup</button>
    {busy && <p role="status">Backup operation in progress…</p>}
    {confirmation && <div role="alertdialog" aria-modal="true" aria-label="Confirm data replacement">
      <p>This validated backup will replace ALL current records, rather than merge them. A recoverable copy of the original database will be kept. Apply it only if you trust its source.</p>
      <button disabled={busy} onClick={() => void confirm()}>Replace data on next startup</button>{" "}
      <button disabled={busy} onClick={() => void cancel()}>Cancel</button>
    </div>}
    {message && <p role="status">{message}</p>}
    {error && <p role="alert">{error}</p>}
  </section>;
}
