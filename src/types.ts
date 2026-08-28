export type Category = "app" | "sync" | "update" | "backup" | "system" | "unrecognized";

export interface ProcessBreakdownEntry {
  label: string;
  downloadBps: number;
  uploadBps: number;
  pidCount: number;
  isExtension: boolean;
}

export interface ProcessRate {
  groupKey: string;
  displayName: string;
  category: Category;
  downloadBps: number;
  uploadBps: number;
  pidCount: number;
  sustainedSeconds: number;
  /** Plain-English sentence saying what this process is. */
  whatItIs: string;
  /** Plain-English verdict saying whether the user should do anything. */
  verdict: string;
  /** Per-job breakdown (page content, graphics, extensions, ...), when available. */
  breakdown: ProcessBreakdownEntry[] | null;
  /** Points at the browser's own Task Manager when it's a significant consumer. */
  taskManagerHint: string | null;
}

export interface Snapshot {
  processes: ProcessRate[];
  totalDownloadBps: number;
  totalUploadBps: number;
  timestampMs: number;
}

export type SuggestionKind =
  | "top-consumer"
  | "sync"
  | "update"
  | "backup"
  | "background"
  | "info";

export interface Suggestion {
  id: string;
  kind: SuggestionKind;
  title: string;
  detail: string;
}

export interface SnapshotPayload {
  snapshot: Snapshot;
  suggestions: Suggestion[];
}
