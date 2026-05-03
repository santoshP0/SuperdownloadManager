import { DownloadItem as DLItem } from "../types/download";
import { DownloadItem } from "./DownloadItem";

interface Props {
  downloads: DLItem[];
  onPause: (id: string) => void;
  onResume: (id: string) => void;
  onCancel: (id: string) => void;
  onOpenFile: (path: string, filename: string) => void;
  onOpenFolder: (path: string) => void;
}

export function DownloadList({
  downloads,
  onPause,
  onResume,
  onCancel,
  onOpenFile,
  onOpenFolder,
}: Props) {
  if (downloads.length === 0) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center gap-3 text-slate-600 select-none">
        <span className="text-6xl">📥</span>
        <p className="text-lg font-medium">No downloads yet</p>
        <p className="text-sm">Click <span className="text-slate-400 font-medium">+ New Download</span> to get started</p>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto px-4 py-3 space-y-2">
      {downloads.map((item) => (
        <DownloadItem
          key={item.id}
          item={item}
          onPause={onPause}
          onResume={onResume}
          onCancel={onCancel}
          onOpenFile={onOpenFile}
          onOpenFolder={onOpenFolder}
        />
      ))}
    </div>
  );
}
