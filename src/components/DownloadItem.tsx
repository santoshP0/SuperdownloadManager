import { DownloadItem as DLItem } from "../types/download";
import { formatBytes, formatETA, formatPercent, formatSpeed } from "../utils/format";
import { ProgressBar } from "./ProgressBar";

interface Props {
  item: DLItem;
  onPause: (id: string) => void;
  onResume: (id: string) => void;
  onCancel: (id: string) => void;
  onRemove: (id: string) => void;
  onOpenFile: (path: string, filename: string) => void;
  onOpenFolder: (path: string) => void;
}

const FILE_ICON: Record<string, string> = {
  zip: "🗜",
  rar: "🗜",
  "7z": "🗜",
  tar: "🗜",
  gz: "🗜",
  mp4: "🎬",
  mkv: "🎬",
  avi: "🎬",
  mov: "🎬",
  mp3: "🎵",
  flac: "🎵",
  wav: "🎵",
  exe: "📦",
  msi: "📦",
  pdf: "📄",
  iso: "💿",
};

function fileIcon(filename: string) {
  const ext = filename.split(".").pop()?.toLowerCase() ?? "";
  return FILE_ICON[ext] ?? "📥";
}

const STATUS_BADGE: Record<string, string> = {
  queued:     "bg-slate-600 text-slate-300",
  downloading:"bg-blue-500/20 text-blue-400",
  retrying:   "bg-orange-500/20 text-orange-400",
  paused:     "bg-yellow-500/20 text-yellow-400",
  completed:  "bg-green-500/20 text-green-400",
  failed:     "bg-red-500/20 text-red-400",
  cancelled:  "bg-slate-700 text-slate-400",
};

export function DownloadItem({
  item,
  onPause,
  onResume,
  onCancel,
  onRemove,
  onOpenFile,
  onOpenFolder,
}: Props) {
  const percent = formatPercent(item.downloaded, item.total_size);
  const isActive    = item.status === "downloading";
  const isRetrying  = item.status === "retrying";
  const isPaused    = item.status === "paused";
  const isCompleted = item.status === "completed";
  const isFailed    = item.status === "failed";

  return (
    <div className="bg-surface-800 border border-slate-700/60 rounded-xl p-4 hover:border-slate-600 transition-colors">
      <div className="flex items-start gap-3">
        {/* File type icon */}
        <span className="text-2xl select-none mt-0.5 shrink-0">
          {fileIcon(item.filename)}
        </span>

        {/* Main info */}
        <div className="flex-1 min-w-0">
          {/* Filename + status badge */}
          <div className="flex items-center gap-2 mb-1">
            <span className="text-white font-medium text-sm truncate">
              {item.filename}
            </span>
            <span
              className={`shrink-0 text-xs px-2 py-0.5 rounded-full font-medium capitalize ${STATUS_BADGE[item.status] ?? ""}`}
            >
              {isRetrying ? `Retrying (${item.retry_count}/3)` : item.status}
            </span>
          </div>

          {/* Source URL */}
          <p className="text-slate-500 text-xs truncate mb-2">{item.url}</p>

          {/* Progress bar */}
          <ProgressBar percent={percent} status={item.status} />

          {/* Stats row */}
          <div className="flex items-center justify-between mt-2 text-xs text-slate-400">
            <span>
              {formatBytes(item.downloaded)}
              {item.total_size > 0 && ` of ${formatBytes(item.total_size)}`}
              {item.total_size > 0 && (
                <span className="ml-1 font-semibold text-slate-300">{percent}%</span>
              )}
            </span>
            <div className="flex items-center gap-3">
              {isActive && (
                <>
                  <span className="text-blue-400 font-medium">{formatSpeed(item.speed)}</span>
                  <span>ETA: {formatETA(item.eta_seconds)}</span>
                </>
              )}
              {item.chunk_count > 1 && (
                <span className="text-slate-500">{item.chunk_count} connections</span>
              )}
            </div>
          </div>

          {/* Error / retry message */}
          {isFailed && item.error && (
            <p className="mt-1.5 text-red-400 text-xs">
              <span className="font-medium">Error:</span> {item.error}
            </p>
          )}
          {isRetrying && item.error && (
            <p className="mt-1.5 text-orange-400 text-xs">{item.error}</p>
          )}
        </div>

        {/* Action buttons */}
        <div className="flex items-center gap-1 shrink-0 ml-1 flex-wrap justify-end max-w-[120px]">
          {isActive && (
            <Btn onClick={() => onPause(item.id)} label="Pause" icon="⏸" />
          )}
          {isPaused && (
            <Btn onClick={() => onResume(item.id)} label="Resume" icon="▶" accent />
          )}
          {isFailed && (
            <Btn onClick={() => onResume(item.id)} label="Retry" icon="↺" accent />
          )}
          {isCompleted && (
            <>
              <Btn onClick={() => onOpenFile(item.save_path, item.filename)} label="Open" icon="📂" />
              <Btn onClick={() => onOpenFolder(item.save_path)} label="Folder" icon="📁" />
            </>
          )}
          {isCompleted ? (
            <Btn onClick={() => onRemove(item.id)} label="Remove" icon="✕" danger />
          ) : (
            <Btn onClick={() => onCancel(item.id)} label="Cancel" icon="✕" danger />
          )}
        </div>
      </div>
    </div>
  );
}

function Btn({
  onClick,
  label,
  icon,
  danger = false,
  accent = false,
}: {
  onClick: () => void;
  label: string;
  icon: string;
  danger?: boolean;
  accent?: boolean;
}) {
  const base = "flex items-center gap-1 px-2 py-1 rounded-md text-xs font-medium transition-colors select-none";
  const style = danger
    ? "text-slate-500 hover:bg-red-500/20 hover:text-red-400"
    : accent
    ? "text-blue-400 hover:bg-blue-500/20"
    : "text-slate-400 hover:bg-slate-700 hover:text-white";

  return (
    <button onClick={onClick} title={label} className={`${base} ${style}`}>
      <span>{icon}</span>
      <span>{label}</span>
    </button>
  );
}
