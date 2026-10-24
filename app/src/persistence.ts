import Database from "@tauri-apps/plugin-sql";

export const DATABASE_URL = "sqlite:mission-control.db";
export interface TestRecord {
  id: string;
  value: string;
  created_at: string;
}

type Connection = Pick<Database, "execute" | "select">;

// Injectable boundary keeps native IPC out of tests and SQL out of components.
export function createPersistence(load: () => Promise<Connection> = () => Database.load(DATABASE_URL)) {
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
    return db.select<TestRecord[]>(
      "SELECT id, value, created_at FROM persistence_test_records ORDER BY created_at ASC, id ASC",
    );
  }
  async function insertRecord(input: string): Promise<void> {
    const value = input.trim();
    if (!value) throw new Error("Enter a non-empty test record.");
    if (writing) throw new Error("A record is already being saved.");
    writing = true;
    try {
      const db = await initialize();
      await db.execute(
        "INSERT INTO persistence_test_records (id, value) VALUES ($1, $2)",
        [crypto.randomUUID(), value],
      );
    } finally {
      writing = false;
    }
  }
  return { initialize, readRecords, insertRecord };
}

export const persistence = createPersistence();
