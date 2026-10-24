import { invoke } from "@tauri-apps/api/core";

export interface TestRecord {
  id: string;
  value: string;
  created_at: string;
}

interface Connection {
  readRecords(): Promise<TestRecord[]>;
  insertRecord(id: string, value: string): Promise<void>;
}

// Injectable boundary keeps native IPC out of tests and SQL out of components.
export function createPersistence(load: () => Promise<Connection> = async () => ({
  readRecords: () => invoke<TestRecord[]>("read_records"),
  insertRecord: (id, value) => invoke<void>("insert_record", { id, value }),
})) {
  let connection: Promise<Connection> | undefined;
  let writing = false;
  function initialize() {
    connection ??= load().catch((error: unknown) => {
      connection = undefined;
      throw error;
    });
    return connection;
  }
  async function readRecords(): Promise<TestRecord[]> {
    const db = await initialize();
    return db.readRecords();
  }
  async function insertRecord(input: string): Promise<void> {
    const value = input.trim();
    if (!value) throw new Error("Enter a non-empty test record.");
    if (writing) throw new Error("A record is already being saved.");
    writing = true;
    try {
      const db = await initialize();
      await db.insertRecord(crypto.randomUUID(), value);
    } finally {
      writing = false;
    }
  }
  return { initialize, readRecords, insertRecord };
}

export const persistence = createPersistence();
