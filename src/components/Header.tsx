import { formatSpeed } from "../utils/format";

interface Props {
  activeCount: number;
  totalSpeed: number;
  onNewDownload: () => void;
  onOpenSettings: () => void;
}

export function Header({ activeCount, totalSpeed, onNewDownload, onOpenSettings }: Props) {
  return (
    <header className="flex items-center justify-between px-5 py-3.5 border-b border-slate-700/60 bg-surface-900 shrink-0">
      <div className="flex items-center gap-3">
        <span className="text-xl select-none">⚡</span>
        <span className="text-white font-semibold text-base tracking-tight">
          Super Download Manager
        </span>
        {activeCount > 0 && (
          <span className="text-xs bg-blue-600 text-white px-2 py-0.5 rounded-full font-medium">
            {activeCount} active
          </span>
        )}
      </div>

      <div className="flex items-center gap-3">
        {totalSpeed > 0 && (
          <span className="text-blue-400 text-sm font-medium">
            ↓ {formatSpeed(totalSpeed)}
          </span>
        )}
        <button
          onClick={onOpenSettings}
          title="Settings"
          className="w-8 h-8 flex items-center justify-center rounded-lg text-slate-400
                     hover:bg-slate-700 hover:text-white transition-colors text-base"
        >
          ⚙
        </button>
        <button
          onClick={onNewDownload}
          className="flex items-center gap-1.5 px-3.5 py-1.5 bg-blue-600 hover:bg-blue-500
                     text-white text-sm font-medium rounded-lg transition-colors"
        >
          <span className="text-base leading-none">+</span>
          New Download
        </button>
      </div>
    </header>
  );
}
