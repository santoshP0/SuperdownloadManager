export type DownloadStatus =
  | "queued"
  | "downloading"
  | "retrying"
  | "paused"
  | "completed"
  | "failed"
  | "cancelled";

export interface DownloadItem {
  id: string;
  url: string;
  filename: string;
  save_path: string;
  total_size: number;
  downloaded: number;
  status: DownloadStatus;
  speed: number;
  eta_seconds: number;
  created_at: number;
  chunk_count: number;
  error: string | null;
  supports_resume: boolean;
  retry_count: number;
}

export interface ProgressEvent {
  id: string;
  downloaded: number;
  total: number;
  speed: number;
  eta_seconds: number;
}

export interface CompletedEvent {
  id: string;
}

export interface FailedEvent {
  id: string;
  error: string;
}

export interface RetryingEvent {
  id: string;
  attempt: number;
}
