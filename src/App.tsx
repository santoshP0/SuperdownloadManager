import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AddDownloadModal } from "./components/AddDownloadModal";
import { DownloadList } from "./components/DownloadList";
import { Header } from "./components/Header";
import { SettingsPanel } from "./components/SettingsPanel";
import { StatusBar } from "./components/StatusBar";
import { useClipboardMonitor } from "./hooks/useClipboardMonitor";
import { useDownloads } from "./hooks/useDownloads";
import { useSpeedHistory } from "./hooks/useSpeedHistory";
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
    setSoundEnabled,
  } = useDownloads();

  const speedHistory = useSpeedHistory(totalSpeed);

  // Load preferences from settings on mount
  useEffect(() => {
    invoke<Settings>("get_settings").then((s) => {
      setClipboardEnabled(s.clipboard_monitor);
      setSoundEnabled(s.sound_on_completion);
    }).catch(() => {});
  }, [setSoundEnabled]);

  useClipboardMonitor(clipboardEnabled, (url) => {
    setClipboardUrl(url);
    setShowModal(true);
  });

  const handleSettingsClose = () => {
    setShowSettings(false);
    invoke<Settings>("get_settings").then((s) => {
      setClipboardEnabled(s.clipboard_monitor);
      setSoundEnabled(s.sound_on_completion);
    }).catch(() => {});
  };

  return (
    <div className="dark flex flex-col h-screen bg-surface-900 text-white overflow-hidden">
      <Header
        activeCount={activeCount}
        totalSpeed={totalSpeed}
        speedHistory={speedHistory}
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

      {showSettings && <SettingsPanel onClose={handleSettingsClose} />}
    </div>
  );
}

export default App;
