import { expect, test, vi } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import StudySessions from "../src/StudySessions";
import {
  createStudyPersistence,
  localStudyDate,
  nextAreaColor,
  validateAreaName,
  validateSession,
  validStudyDate,
} from "../src/studyPersistence";

test("study screen exposes the approved inline form and history", () => {
  const markup = renderToStaticMarkup(<StudySessions />);
  for (const text of ["New study session", "Study history", "Study area", "Duration (minutes)", "Study date", "Notes", "Add", "Save session"]) {
    expect(markup).toContain(text);
  }
  expect(markup).toContain('type="date"');
});

test("local dates do not use UTC midnight conversion", () => {
  expect(localStudyDate(new Date(2026, 10, 12, 23, 59))).toBe("2026-11-12");
  expect(validStudyDate("2024-02-29")).toBe(true);
  for (const date of ["2026-02-29", "2026-04-31", "0000-01-01", "2026-13-01", "2026-1-2"]) {
    expect(validStudyDate(date)).toBe(false);
  }
});

test("rejects invalid areas and sessions without blocking free-form notes", () => {
  expect(validateAreaName("   ")).toMatch(/area name/);
  expect(validateAreaName("a".repeat(81))).toMatch(/area name/);
  expect(validateAreaName("Programming")).toBe(null);
  const valid = { areaId: "test", durationMinutes: 45, studyDate: "2026-11-12", notes: "Rust & SQLite\n\"quotes\"" };
  expect(validateSession(valid)).toBe(null);
  expect(validateSession({ ...valid, areaId: "" })).toMatch(/area/);
  expect(validateSession({ ...valid, durationMinutes: 0 })).toMatch(/minutes/);
  expect(validateSession({ ...valid, durationMinutes: 1.25 })).toMatch(/minutes/);
  expect(validateSession({ ...valid, studyDate: "2026-02-30" })).toMatch(/date/);
  expect(validateSession({ ...valid, notes: "x".repeat(10001) })).toMatch(/Notes/);
});

test("automatic colors are deterministic, cycling without requiring extra dependencies", () => {
  expect(nextAreaColor(0)).toBe("#F4D35E");
  expect(nextAreaColor(6)).toBe(nextAreaColor(0));
});

test("native calls use separate area and session commands with bound arguments", async () => {
  const command = vi.fn(async () => undefined);
  const store = createStudyPersistence(command as unknown as typeof import("@tauri-apps/api/core").invoke);
  await store.createArea(" Programming ", "#F4D35E");
  expect(command).toHaveBeenCalledWith("create_study_area", expect.objectContaining({ name: "Programming", color: "#F4D35E" }));
  await store.createSession({ areaId: crypto.randomUUID(), durationMinutes: 45, studyDate: "2026-11-12", notes: "Free-form details" });
  expect(command).toHaveBeenCalledWith("create_study_session", expect.objectContaining({
    durationMinutes: 45, studyDate: "2026-11-12", notes: "Free-form details",
  }));
  expect(command).toHaveBeenCalledTimes(2);
});
