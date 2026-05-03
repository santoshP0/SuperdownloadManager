import { DownloadItem } from "../types/download";

interface Props {
  downloads: DownloadItem[];
}

export function StatusBar({ downloads }: Props) {
  const completed = downloads.filter((d) => d.status === "completed").length;
  const failed = downloads.filter((d) => d.status === "failed").length;
  const queued = downloads.filter((d) => d.status === "queued").length;
  const total = downloads.length;

  return (
    <footer className="shrink-0 flex items-center gap-4 px-5 py-2 border-t border-slate-700/60 bg-surface-900 text-xs text-slate-500">
      <span>Total: {total}</span>
      {completed > 0 && <span className="text-green-500">✓ {completed} done</span>}
      {queued > 0 && <span className="text-slate-400">⏳ {queued} queued</span>}
      {failed > 0 && <span className="text-red-500">✗ {failed} failed</span>}
      <span className="ml-auto">API: localhost:9876</span>
    </footer>
  );
}
