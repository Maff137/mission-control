import { expect, test, vi } from "vitest";
import { createPersistence, DATABASE_URL } from "../src/persistence";

test("shares initialization and reads in deterministic order", async () => {
  const select = vi.fn().mockResolvedValue([]);
  const load = vi.fn().mockResolvedValue({ select, execute: vi.fn() });
  const store = createPersistence(load);
  await Promise.all([store.initialize(), store.initialize(), store.readRecords()]);
  expect(load).toHaveBeenCalledTimes(1);
  expect(select).toHaveBeenCalledWith(expect.stringContaining("ORDER BY created_at ASC, id ASC"));
  expect(DATABASE_URL).toBe("sqlite:mission-control.db");
});

test("rejects whitespace without loading and binds values instead of interpolating SQL", async () => {
  const execute = vi.fn().mockResolvedValue({ rowsAffected: 1 });
  const load = vi.fn().mockResolvedValue({ select: vi.fn(), execute });
  const store = createPersistence(load);
  await expect(store.insertRecord(" \t\n ")).rejects.toThrow("non-empty");
  expect(load).not.toHaveBeenCalled();
  await store.insertRecord("  quote'; DROP TABLE example; --  ");
  expect(execute).toHaveBeenCalledWith(expect.stringContaining("VALUES ($1, $2)"), [expect.any(String), "quote'; DROP TABLE example; --"]);
});

test("propagates initialization errors and allows retry", async () => {
  const load = vi.fn().mockRejectedValueOnce(new Error("unavailable")).mockResolvedValue({ select: vi.fn().mockResolvedValue([]) });
  const store = createPersistence(load);
  await expect(store.readRecords()).rejects.toThrow("unavailable");
  await expect(store.readRecords()).resolves.toEqual([]);
});

test("blocks concurrent writes and releases the guard after errors", async () => {
  const execute = vi.fn().mockRejectedValue(new Error("read only"));
  const store = createPersistence(async () => ({ execute, select: vi.fn() }));
  const first = store.insertRecord("first");
  await expect(store.insertRecord("second")).rejects.toThrow("already");
  await expect(first).rejects.toThrow("read only");
  await expect(store.insertRecord("retry")).rejects.toThrow("read only");
  expect(execute).toHaveBeenCalledTimes(2);
});

test("propagates select failures without fabricating empty records", async () => {
  const store = createPersistence(async () => ({ execute: vi.fn(), select: vi.fn().mockRejectedValue(new Error("corrupt")) }));
  await expect(store.readRecords()).rejects.toThrow("corrupt");
});

test("blocks dependent operations when native initialization fails", async () => {
  const execute = vi.fn();
  const select = vi.fn();
  const load = vi.fn(async () => {
    await Promise.reject(new Error("migration failed"));
    return { execute, select };
  });
  const store = createPersistence(load);
  await expect(store.readRecords()).rejects.toThrow("migration failed");
  await expect(store.insertRecord("keep data")).rejects.toThrow("migration failed");
  expect(execute).not.toHaveBeenCalled();
  expect(select).not.toHaveBeenCalled();
});
