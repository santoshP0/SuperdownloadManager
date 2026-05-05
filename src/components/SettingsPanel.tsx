import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { Settings } from "../types/download";

interface Props {
  onClose: () => void;
}

const DEFAULT: Settings = {
  default_save_path: "",
  default_chunk_count: 8,
  max_concurrent: 3,
  speed_limit_kbps: 0,
  clipboard_monitor: true,
  notifications: true,
  sound_on_completion: true,
  category_folders: false,
};

export function SettingsPanel({ onClose }: Props) {
  const [settings, setSettings] = useState<Settings>(DEFAULT);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    invoke<Settings>("get_settings")
      .then(setSettings)
      .catch(() => {});
  }, []);

  const handleSave = async () => {
    try {
      await invoke("save_settings", { newSettings: settings });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error("save_settings failed:", e);
    }
  };

  const browseSavePath = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected) setSettings((s) => ({ ...s, default_save_path: selected as string }));
  };

  const set = <K extends keyof Settings>(key: K, value: Settings[K]) =>
    setSettings((s) => ({ ...s, [key]: value }));

  return (
    <div
      className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4"
      onClick={(e) => e.target === e.currentTarget && onClose()}
    >
      <div className="bg-surface-800 border border-slate-700 rounded-xl w-[520px] max-h-[90vh] shadow-2xl flex flex-col">
        {/* Sticky header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-700/60 shrink-0">
          <h2 className="text-white font-semibold text-lg">Settings</h2>
          <button
            onClick={onClose}
            className="text-slate-400 hover:text-white transition-colors text-xl leading-none"
          >
            ✕
          </button>
        </div>

        {/* Scrollable body */}
        <div className="flex-1 overflow-y-auto px-6 py-5 space-y-6 min-h-0">
          {/* Default save path */}
          <div className="space-y-1.5">
            <label className="text-slate-400 text-sm">Default Download Folder</label>
            <div className="flex gap-2">
              <input
                value={settings.default_save_path}
                onChange={(e) => set("default_save_path", e.target.value)}
                className="flex-1 bg-surface-700 border border-slate-600 rounded-lg px-3 py-2
                           text-white text-sm placeholder-slate-500 focus:outline-none
                           focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
              />
              <button
                onClick={browseSavePath}
                className="px-3 py-2 bg-surface-600 hover:bg-surface-500 border border-slate-600
                           rounded-lg text-slate-300 text-sm transition-colors"
              >
                Browse
              </button>
            </div>
          </div>

          {/* Default chunk count */}
          <div className="space-y-2">
            <label className="text-slate-400 text-sm">
              Default Parallel Connections —{" "}
              <span className="text-white font-medium">{settings.default_chunk_count}</span>
            </label>
            <input
              type="range" min={1} max={16} step={1}
              value={settings.default_chunk_count}
              onChange={(e) => set("default_chunk_count", Number(e.target.value))}
              className="w-full accent-blue-500"
            />
            <div className="flex justify-between text-xs text-slate-500">
              <span>1 (standard)</span><span>8 (fast)</span><span>16 (max)</span>
            </div>
          </div>

          {/* Max concurrent */}
          <div className="space-y-2">
            <label className="text-slate-400 text-sm">
              Max Simultaneous Downloads —{" "}
              <span className="text-white font-medium">{settings.max_concurrent}</span>
            </label>
            <input
              type="range" min={1} max={10} step={1}
              value={settings.max_concurrent}
              onChange={(e) => set("max_concurrent", Number(e.target.value))}
              className="w-full accent-blue-500"
            />
            <div className="flex justify-between text-xs text-slate-500">
              <span>1</span><span>5</span><span>10</span>
            </div>
          </div>

          {/* Speed limit */}
          <div className="space-y-2">
            <label className="text-slate-400 text-sm">
              Speed Limit —{" "}
              <span className="text-white font-medium">
                {settings.speed_limit_kbps === 0
                  ? "Unlimited"
                  : settings.speed_limit_kbps >= 1024
                    ? `${(settings.speed_limit_kbps / 1024).toFixed(0)} MB/s`
                    : `${settings.speed_limit_kbps} KB/s`}
              </span>
            </label>
            <input
              type="range" min={0} max={102400} step={256}
              value={settings.speed_limit_kbps}
              onChange={(e) => set("speed_limit_kbps", Number(e.target.value))}
              className="w-full accent-blue-500"
            />
            <div className="flex justify-between text-xs text-slate-500">
              <span>Unlimited</span><span>50 MB/s</span><span>100 MB/s</span>
            </div>
          </div>

          {/* Divider */}
          <div className="border-t border-slate-700/60" />

          {/* Toggles */}
          <div className="space-y-4">
            <Toggle
              label="Clipboard Monitor"
              description="Auto-detect download URLs copied to clipboard"
              value={settings.clipboard_monitor}
              onChange={(v) => set("clipboard_monitor", v)}
            />
            <Toggle
              label="Desktop Notifications"
              description="Show a toast when a download completes"
              value={settings.notifications}
              onChange={(v) => set("notifications", v)}
            />
            <Toggle
              label="Sound on Completion"
              description="Play a short ding when each download finishes"
              value={settings.sound_on_completion}
              onChange={(v) => set("sound_on_completion", v)}
            />
            <Toggle
              label="Category Folders"
              description="Auto-sort downloads into Videos, Music, Documents… subfolders"
              value={settings.category_folders}
              onChange={(v) => set("category_folders", v)}
            />
          </div>
        </div>

        {/* Sticky footer */}
        <div className="flex justify-end gap-3 px-6 py-4 border-t border-slate-700/60 shrink-0">
          <button
            onClick={onClose}
            className="px-4 py-2 rounded-lg text-slate-400 hover:text-white text-sm transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            className={`px-5 py-2 rounded-lg text-white text-sm font-medium transition-all duration-200
              ${saved ? "bg-green-600 hover:bg-green-500" : "bg-blue-600 hover:bg-blue-500"}`}
          >
            {saved ? "✓ Saved" : "Save Settings"}
          </button>
        </div>
      </div>
    </div>
  );
}

function Toggle({
  label,
  description,
  value,
  onChange,
}: {
  label: string;
  description: string;
  value: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between gap-4">
      <div className="min-w-0">
        <p className="text-white text-sm font-medium">{label}</p>
        <p className="text-slate-500 text-xs mt-0.5">{description}</p>
      </div>

      <button
        onClick={() => onChange(!value)}
        role="switch"
        aria-checked={value}
        className={`
          relative inline-flex shrink-0 items-center
          w-12 h-6 rounded-full
          transition-colors duration-200 ease-in-out
          focus:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-900
          ${value ? "bg-blue-500" : "bg-slate-600"}
        `}
      >
        <span
          className={`
            inline-block w-5 h-5 rounded-full bg-white shadow-md
            transform transition-transform duration-200 ease-in-out
            ${value ? "translate-x-6" : "translate-x-0.5"}
          `}
        />
      </button>
    </div>
  );
}
