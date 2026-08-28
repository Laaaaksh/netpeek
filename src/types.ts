export type Category = "app" | "sync" | "update" | "backup" | "background";

export interface ProcessRate {
  groupKey: string;
  displayName: string;
  category: Category;
  downloadBps: number;
  uploadBps: number;
  pidCount: number;
  sustainedSeconds: number;
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
