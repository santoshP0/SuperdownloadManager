export type DownloadStatus =
  | "queued"
  | "downloading"
  | "retrying"
  | "paused"
  | "completed"
  | "failed"
  | "cancelled";

export interface ChunkProgress {
  id: number;
  downloaded: number;
  size: number;
}

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
  chunk_progress?: ChunkProgress[];
}

export interface ProgressEvent {
  id: string;
  downloaded: number;
  total: number;
  speed: number;
  eta_seconds: number;
  chunks?: ChunkProgress[];
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

export interface Settings {
  default_save_path: string;
  default_chunk_count: number;
  max_concurrent: number;
  speed_limit_kbps: number;
  clipboard_monitor: boolean;
  notifications: boolean;
  sound_on_completion: boolean;
  category_folders: boolean;
}
