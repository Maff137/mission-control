import { expect, test, vi } from "vitest";
import { createPersistence } from "../src/persistence";

test("shares initialization and delegates reads to the native boundary", async () => {
  const readRecords = vi.fn().mockResolvedValue([]);
  const load = vi.fn().mockResolvedValue({ readRecords, insertRecord: vi.fn() });
  const store = createPersistence(load);
  await Promise.all([store.initialize(), store.initialize(), store.readRecords()]);
  expect(load).toHaveBeenCalledTimes(1);
  expect(readRecords).toHaveBeenCalledWith();
});

test("rejects whitespace without loading and passes literal values to native insertion", async () => {
  const insertRecord = vi.fn().mockResolvedValue({ rowsAffected: 1 });
  const load = vi.fn().mockResolvedValue({ readRecords: vi.fn(), insertRecord });
  const store = createPersistence(load);
  await expect(store.insertRecord(" \t\n ")).rejects.toThrow("non-empty");
  expect(load).not.toHaveBeenCalled();
  await store.insertRecord("  quote'; DROP TABLE example; --  ");
  expect(insertRecord).toHaveBeenCalledWith(expect.any(String), "quote'; DROP TABLE example; --");
});

test("propagates initialization errors and allows retry", async () => {
  const load = vi.fn().mockRejectedValueOnce(new Error("unavailable")).mockResolvedValue({ readRecords: vi.fn().mockResolvedValue([]) });
  const store = createPersistence(load);
  await expect(store.readRecords()).rejects.toThrow("unavailable");
  await expect(store.readRecords()).resolves.toEqual([]);
});

test("blocks concurrent writes and releases the guard after errors", async () => {
  const insertRecord = vi.fn().mockRejectedValue(new Error("read only"));
  const store = createPersistence(async () => ({ insertRecord, readRecords: vi.fn() }));
  const first = store.insertRecord("first");
  await expect(store.insertRecord("second")).rejects.toThrow("already");
  await expect(first).rejects.toThrow("read only");
  await expect(store.insertRecord("retry")).rejects.toThrow("read only");
  expect(insertRecord).toHaveBeenCalledTimes(2);
});

test("propagates readRecords failures without fabricating empty records", async () => {
  const store = createPersistence(async () => ({ insertRecord: vi.fn(), readRecords: vi.fn().mockRejectedValue(new Error("corrupt")) }));
  await expect(store.readRecords()).rejects.toThrow("corrupt");
});

test("blocks dependent operations when native initialization fails", async () => {
  const insertRecord = vi.fn();
  const readRecords = vi.fn();
  const load = vi.fn(async () => {
    await Promise.reject(new Error("migration failed"));
    return { insertRecord, readRecords };
  });
  const store = createPersistence(load);
  await expect(store.readRecords()).rejects.toThrow("migration failed");
  await expect(store.insertRecord("keep data")).rejects.toThrow("migration failed");
  expect(insertRecord).not.toHaveBeenCalled();
  expect(readRecords).not.toHaveBeenCalled();
});
