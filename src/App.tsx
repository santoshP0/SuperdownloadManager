import { useEffect, useState } from "react";
import { AddDownloadModal } from "./components/AddDownloadModal";
import { DownloadList } from "./components/DownloadList";
import { Header } from "./components/Header";
import { SettingsPanel } from "./components/SettingsPanel";
import { StatusBar } from "./components/StatusBar";
import { useClipboardMonitor } from "./hooks/useClipboardMonitor";
import { useDownloads } from "./hooks/useDownloads";
import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "./types/download";

function App() {
  const [showModal, setShowModal] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [clipboardUrl, setClipboardUrl] = useState<string | null>(null);
  const [clipboardEnabled, setClipboardEnabled] = useState(true);

  const {
    downloads,
    addDownload,
    pauseDownload,
    resumeDownload,
    cancelDownload,
    removeDownload,
    openFile,
    openFolder,
    totalSpeed,
    activeCount,
  } = useDownloads();

  // Load clipboard_monitor preference once on mount
  useEffect(() => {
    invoke<Settings>("get_settings")
      .then((s) => setClipboardEnabled(s.clipboard_monitor))
      .catch(() => {});
  }, []);

  useClipboardMonitor(clipboardEnabled, (url) => {
    setClipboardUrl(url);
    setShowModal(true);
  });

  return (
    <div className="dark flex flex-col h-screen bg-surface-900 text-white overflow-hidden">
      <Header
        activeCount={activeCount}
        totalSpeed={totalSpeed}
        onNewDownload={() => setShowModal(true)}
        onOpenSettings={() => setShowSettings(true)}
      />
      <DownloadList
        downloads={downloads}
        onPause={pauseDownload}
        onResume={resumeDownload}
        onCancel={cancelDownload}
        onRemove={removeDownload}
        onOpenFile={openFile}
        onOpenFolder={openFolder}
      />
      <StatusBar downloads={downloads} />

      {showModal && (
        <AddDownloadModal
          initialUrl={clipboardUrl ?? undefined}
          onAdd={addDownload}
          onClose={() => { setShowModal(false); setClipboardUrl(null); }}
        />
      )}

      {showSettings && (
        <SettingsPanel
          onClose={() => {
            setShowSettings(false);
            // Re-read clipboard setting in case user changed it
            invoke<Settings>("get_settings")
              .then((s) => setClipboardEnabled(s.clipboard_monitor))
              .catch(() => {});
          }}
        />
      )}
    </div>
  );
}

export default App;
