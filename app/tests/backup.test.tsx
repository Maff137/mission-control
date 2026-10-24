import { expect, test } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import Backup, { nativeError } from "../src/Backup";
import { readFileSync } from "node:fs";

test("backup interface explains replacement and exposes both native actions", () => {
  const html = renderToStaticMarkup(<Backup disabled={false} onBlock={() => {}} />);
  expect(html).toContain("Export Backup");
  expect(html).toContain("Restore Backup");
  expect(html).toContain("Restore replaces all current data");
  expect(html).toContain(".mcbackup");
});

test("backup controls are disabled during persistence operations", () => {
  const html = renderToStaticMarkup(<Backup disabled onBlock={() => {}} />);
  expect(html.match(/disabled=""/g)).toHaveLength(2);
});

test("structured native errors retain actionable messages", () => {
  expect(nativeError({ code: "destination_exists", message: "Choose a new filename." })).toBe("Choose a new filename.");
  expect(nativeError(new Error("Check permissions."))).toBe("Check permissions.");
  expect(nativeError("Cancelled")).toBe("Cancelled");
});

test("restore lifecycle registration precedes SQL preload and startup acknowledgement", () => {
  const source = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
  expect(source.indexOf('new("backup-lifecycle")')).toBeLessThan(source.indexOf("tauri_plugin_sql::Builder"));
  expect(source).toContain("store.startup()");
  expect(source).toContain(".finish_startup()");
  const capabilities = JSON.parse(readFileSync(new URL("../src-tauri/capabilities/default.json", import.meta.url), "utf8"));
  expect(capabilities.permissions).toEqual([]);
});
