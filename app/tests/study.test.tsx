import { expect, test, vi } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import StudySessions from "../src/StudySessions";
import {
  createStudyPersistence,
  displayStudyDate,
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

test("history preserves early years and calendar days across device time zones", () => {
  const originalTimezone = process.env.TZ;
  try {
    for (const timezone of ["America/Cuiaba", "Pacific/Apia", "Asia/Tokyo"]) {
      process.env.TZ = timezone;
      expect(displayStudyDate("0001-01-01")).toBe("Jan 1, 1");
      expect(displayStudyDate("0099-12-31")).toBe("Dec 31, 99");
      expect(displayStudyDate("2011-12-30")).toBe("Dec 30, 2011");
      expect(displayStudyDate("2026-11-12")).toBe("Nov 12, 2026");
    }
  } finally {
    if (originalTimezone === undefined) delete process.env.TZ;
    else process.env.TZ = originalTimezone;
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

test("native storage failures propagate without automatic duplicate writes", async () => {
  const failure = { code: "database_error", message: "SQLite write failed. Keep all database files." };
  const command = vi.fn().mockRejectedValue(failure);
  const store = createStudyPersistence(command as unknown as typeof import("@tauri-apps/api/core").invoke);
  await expect(store.createArea("Synthetic area", "#F4D35E")).rejects.toEqual(failure);
  await expect(store.createSession({ areaId: crypto.randomUUID(), durationMinutes: 1, studyDate: "2024-02-29", notes: "" })).rejects.toEqual(failure);
  await expect(store.listAreas()).rejects.toEqual(failure);
  await expect(store.listSessions()).rejects.toEqual(failure);
  expect(command).toHaveBeenCalledTimes(4);
});

test("invalid session input never reaches native storage", async () => {
  const command = vi.fn();
  const store = createStudyPersistence(command as unknown as typeof import("@tauri-apps/api/core").invoke);
  for (const durationMinutes of [-1, 0, 0.5, NaN, Infinity, Number.MAX_SAFE_INTEGER + 1]) {
    await expect(store.createSession({ areaId: crypto.randomUUID(), durationMinutes, studyDate: "2026-11-12", notes: "" })).rejects.toThrow(/minutes/);
  }
  expect(command).not.toHaveBeenCalled();
});
