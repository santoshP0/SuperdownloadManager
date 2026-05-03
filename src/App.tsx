import { useState } from "react";
import { AddDownloadModal } from "./components/AddDownloadModal";
import { DownloadList } from "./components/DownloadList";
import { Header } from "./components/Header";
import { StatusBar } from "./components/StatusBar";
import { useDownloads } from "./hooks/useDownloads";

function App() {
  const [showModal, setShowModal] = useState(false);
  const {
    downloads,
    addDownload,
    pauseDownload,
    resumeDownload,
    cancelDownload,
    openFile,
    openFolder,
    totalSpeed,
    activeCount,
  } = useDownloads();

  return (
    <div className="dark flex flex-col h-screen bg-surface-900 text-white overflow-hidden">
      <Header
        activeCount={activeCount}
        totalSpeed={totalSpeed}
        onNewDownload={() => setShowModal(true)}
      />
      <DownloadList
        downloads={downloads}
        onPause={pauseDownload}
        onResume={resumeDownload}
        onCancel={cancelDownload}
        onOpenFile={openFile}
        onOpenFolder={openFolder}
      />
      <StatusBar downloads={downloads} />

      {showModal && (
        <AddDownloadModal
          onAdd={addDownload}
          onClose={() => setShowModal(false)}
        />
      )}
    </div>
  );
}

export default App;
