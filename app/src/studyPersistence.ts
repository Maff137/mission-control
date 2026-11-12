import { invoke } from "@tauri-apps/api/core";

export interface StudyArea {
  id: string;
  name: string;
  color: string;
  created_at: string;
}

export interface StudySession {
  id: string;
  area_id: string;
  area_name: string;
  area_color: string;
  duration_minutes: number;
  study_date: string;
  notes: string;
  created_at: string;
}

export interface SessionInput {
  areaId: string;
  durationMinutes: number;
  studyDate: string;
  notes: string;
}

const areaPalette = ["#F4D35E", "#6A9FE6", "#B18BEA", "#67BAC3", "#E8A36E", "#A7AACA"];

export function nextAreaColor(existingCount: number): string {
  return areaPalette[existingCount % areaPalette.length];
}

export function localStudyDate(date = new Date()): string {
  const year = String(date.getFullYear()).padStart(4, "0");
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export function validStudyDate(date: string): boolean {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(date)) return false;
  const [year, month, day] = date.split("-").map(Number);
  if (year < 1 || month < 1 || month > 12 || day < 1) return false;
  const leap = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  const days = [31, leap ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
  return day <= days[month - 1];
}

export function validateAreaName(value: string): string | null {
  const name = value.trim();
  if (!name || Array.from(name).length > 80) {
    return "Enter an area name between 1 and 80 characters.";
  }
  return null;
}

export function validateSession(input: SessionInput): string | null {
  if (!input.areaId) return "Choose a study area.";
  if (!Number.isSafeInteger(input.durationMinutes) || input.durationMinutes < 1) {
    return "Enter a whole number of minutes greater than zero.";
  }
  if (!validStudyDate(input.studyDate)) return "Enter a valid calendar date.";
  if (Array.from(input.notes).length > 10000) return "Notes cannot exceed 10,000 characters.";
  return null;
}

export function createStudyPersistence(command: typeof invoke = invoke) {
  return {
    listAreas: () => command<StudyArea[]>("list_study_areas"),
    listSessions: () => command<StudySession[]>("list_study_sessions"),
    async createArea(nameInput: string, color: string): Promise<string> {
      const error = validateAreaName(nameInput);
      if (error) throw new Error(error);
      const id = crypto.randomUUID();
      await command<void>("create_study_area", { id, name: nameInput.trim(), color });
      return id;
    },
    async createSession(input: SessionInput): Promise<void> {
      const error = validateSession(input);
      if (error) throw new Error(error);
      await command<void>("create_study_session", {
        id: crypto.randomUUID(),
        areaId: input.areaId,
        durationMinutes: input.durationMinutes,
        studyDate: input.studyDate,
        notes: input.notes,
      });
    },
  };
}

export const studyPersistence = createStudyPersistence();
