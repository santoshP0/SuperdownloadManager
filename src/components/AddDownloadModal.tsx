import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface Props {
  initialUrl?: string;
  onAdd: (url: string, filename: string, savePath: string, chunks: number) => void;
  onClose: () => void;
}

export function AddDownloadModal({ initialUrl, onAdd, onClose }: Props) {
  const [url, setUrl] = useState(initialUrl ?? "");
  const [filename, setFilename] = useState("");
  const [savePath, setSavePath] = useState("");
  const [chunks, setChunks] = useState(8);
  const urlRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    invoke<string>("get_default_download_dir").then(setSavePath).catch(() => {});
    urlRef.current?.focus();
  }, []);

  // Auto-extract filename from URL
  useEffect(() => {
    if (!url) return;
    try {
      const raw = new URL(url).pathname.split("/").pop() ?? "";
      if (raw) setFilename(decodeURIComponent(raw));
    } catch {
      const raw = url.split("/").pop()?.split("?")[0] ?? "";
      if (raw) setFilename(raw);
    }
  }, [url]);

  const handleBrowse = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected) setSavePath(selected as string);
  };

  const handleSubmit = () => {
    if (!url.trim() || !filename.trim() || !savePath.trim()) return;
    onAdd(url.trim(), filename.trim(), savePath.trim(), chunks);
    onClose();
  };

  return (
    <div
      className="fixed inset-0 bg-black/70 flex items-center justify-center z-50"
      onClick={(e) => e.target === e.currentTarget && onClose()}
    >
      <div className="bg-surface-800 border border-slate-700 rounded-xl w-[520px] shadow-2xl p-6 space-y-5">
        <h2 className="text-white font-semibold text-lg">New Download</h2>

        {/* URL */}
        <div className="space-y-1.5">
          <label className="text-slate-400 text-sm">Download URL</label>
          <input
            ref={urlRef}
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="https://example.com/file.zip"
            className="w-full bg-surface-700 border border-slate-600 rounded-lg px-3 py-2
                       text-white text-sm placeholder-slate-500 focus:outline-none
                       focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
            onKeyDown={(e) => e.key === "Enter" && handleSubmit()}
          />
        </div>

        {/* Filename */}
        <div className="space-y-1.5">
          <label className="text-slate-400 text-sm">File Name</label>
          <input
            value={filename}
            onChange={(e) => setFilename(e.target.value)}
            placeholder="filename.zip"
            className="w-full bg-surface-700 border border-slate-600 rounded-lg px-3 py-2
                       text-white text-sm placeholder-slate-500 focus:outline-none
                       focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
          />
        </div>

        {/* Save path */}
        <div className="space-y-1.5">
          <label className="text-slate-400 text-sm">Save To</label>
          <div className="flex gap-2">
            <input
              value={savePath}
              onChange={(e) => setSavePath(e.target.value)}
              className="flex-1 bg-surface-700 border border-slate-600 rounded-lg px-3 py-2
                         text-white text-sm placeholder-slate-500 focus:outline-none
                         focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
            />
            <button
              onClick={handleBrowse}
              className="px-3 py-2 bg-surface-600 hover:bg-surface-500 border border-slate-600
                         rounded-lg text-slate-300 text-sm transition-colors"
            >
              Browse
            </button>
          </div>
        </div>

        {/* Chunk count */}
        <div className="space-y-1.5">
          <label className="text-slate-400 text-sm">
            Parallel Connections — <span className="text-white font-medium">{chunks}</span>
          </label>
          <input
            type="range"
            min={1}
            max={16}
            step={1}
            value={chunks}
            onChange={(e) => setChunks(Number(e.target.value))}
            className="w-full accent-blue-500"
          />
          <div className="flex justify-between text-xs text-slate-500">
            <span>1 (standard)</span>
            <span>8 (fast)</span>
            <span>16 (max)</span>
          </div>
        </div>

        {/* Actions */}
        <div className="flex justify-end gap-3 pt-1">
          <button
            onClick={onClose}
            className="px-4 py-2 rounded-lg text-slate-400 hover:text-white text-sm transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={!url.trim() || !filename.trim() || !savePath.trim()}
            className="px-5 py-2 rounded-lg bg-blue-600 hover:bg-blue-500 disabled:opacity-40
                       disabled:cursor-not-allowed text-white text-sm font-medium transition-colors"
          >
            Start Download
          </button>
        </div>
      </div>
    </div>
  );
}
