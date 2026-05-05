import { useState } from "react";
import { DownloadItem as DLItem } from "../types/download";
import { DownloadItem } from "./DownloadItem";

type Tab = "all" | "active" | "completed" | "failed";

interface Props {
  downloads: DLItem[];
  onPause: (id: string) => void;
  onResume: (id: string) => void;
  onCancel: (id: string) => void;
  onRemove: (id: string) => void;
  onOpenFile: (path: string, filename: string) => void;
  onOpenFolder: (path: string, filename: string) => void;
}

export function DownloadList({
  downloads,
  onPause,
  onResume,
  onCancel,
  onRemove,
  onOpenFile,
  onOpenFolder,
}: Props) {
  const [tab, setTab] = useState<Tab>("all");

  const counts = {
    all:       downloads.length,
    active:    downloads.filter((d) => ["downloading","retrying","queued"].includes(d.status)).length,
    completed: downloads.filter((d) => d.status === "completed").length,
    failed:    downloads.filter((d) => d.status === "failed").length,
  };

  const visible = downloads.filter((d) => {
    if (tab === "active")    return ["downloading","retrying","queued"].includes(d.status);
    if (tab === "completed") return d.status === "completed";
    if (tab === "failed")    return d.status === "failed";
    return true;
  });

  if (downloads.length === 0) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center gap-3 text-slate-600 select-none">
        <span className="text-6xl">📥</span>
        <p className="text-lg font-medium">No downloads yet</p>
        <p className="text-sm">
          Click{" "}
          <span className="text-slate-400 font-medium">+ New Download</span>{" "}
          to get started, or copy a download link — it will be detected automatically.
        </p>
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* Filter tabs */}
      <div className="flex gap-1 px-4 pt-3 pb-2 border-b border-slate-700/40 shrink-0">
        {(["all", "active", "completed", "failed"] as Tab[]).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`flex items-center gap-1.5 px-3 py-1 rounded-md text-xs font-medium transition-colors capitalize
              ${tab === t
                ? "bg-slate-700 text-white"
                : "text-slate-500 hover:text-slate-300 hover:bg-slate-800"
              }`}
          >
            {t}
            {counts[t] > 0 && (
              <span
                className={`text-[10px] px-1.5 py-0 rounded-full font-semibold
                  ${tab === t ? "bg-slate-500 text-white" : "bg-slate-700 text-slate-400"}`}
              >
                {counts[t]}
              </span>
            )}
          </button>
        ))}
      </div>

      {/* Download items */}
      {visible.length === 0 ? (
        <div className="flex-1 flex items-center justify-center text-slate-600 text-sm select-none">
          No {tab} downloads
        </div>
      ) : (
        <div className="flex-1 overflow-y-auto px-4 py-3 space-y-2">
          {visible.map((item) => (
            <DownloadItem
              key={item.id}
              item={item}
              onPause={onPause}
              onResume={onResume}
              onCancel={onCancel}
              onRemove={onRemove}
              onOpenFile={onOpenFile}
              onOpenFolder={onOpenFolder}
            />
          ))}
        </div>
      )}
    </div>
  );
}
