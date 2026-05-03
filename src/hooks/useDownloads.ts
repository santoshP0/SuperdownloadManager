import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  CompletedEvent,
  DownloadItem,
  FailedEvent,
  ProgressEvent,
  RetryingEvent,
} from "../types/download";

export function useDownloads() {
  const [downloads, setDownloads] = useState<DownloadItem[]>([]);
  const unlistenRefs = useRef<Array<() => void>>([]);

  const refresh = useCallback(async () => {
    try {
      const items = await invoke<DownloadItem[]>("get_downloads");
      setDownloads(items);
    } catch (e) {
      console.error("get_downloads failed:", e);
    }
  }, []);

  useEffect(() => {
    refresh();

    const setupListeners = async () => {
      const unProgress = await listen<ProgressEvent>(
        "download://progress",
        ({ payload }) => {
          setDownloads((prev) =>
            prev.map((d) =>
              d.id === payload.id
                ? {
                    ...d,
                    downloaded: payload.downloaded,
                    total_size: payload.total,
                    speed: payload.speed,
                    eta_seconds: payload.eta_seconds,
                    status: "downloading",
                  }
                : d
            )
          );
        }
      );

      const unCompleted = await listen<CompletedEvent>(
        "download://completed",
        ({ payload }) => {
          setDownloads((prev) =>
            prev.map((d) =>
              d.id === payload.id
                ? { ...d, status: "completed", speed: 0, eta_seconds: 0 }
                : d
            )
          );
        }
      );

      const unFailed = await listen<FailedEvent>(
        "download://failed",
        ({ payload }) => {
          setDownloads((prev) =>
            prev.map((d) =>
              d.id === payload.id
                ? { ...d, status: "failed", error: payload.error, speed: 0 }
                : d
            )
          );
        }
      );

      const unRetrying = await listen<RetryingEvent>(
        "download://retrying",
        ({ payload }) => {
          setDownloads((prev) =>
            prev.map((d) =>
              d.id === payload.id
                ? { ...d, status: "retrying", retry_count: payload.attempt, speed: 0 }
                : d
            )
          );
        }
      );

      unlistenRefs.current = [unProgress, unCompleted, unFailed, unRetrying];
    };

    setupListeners();

    return () => {
      unlistenRefs.current.forEach((fn) => fn());
    };
  }, [refresh]);

  const addDownload = useCallback(
    async (
      url: string,
      filename: string,
      savePath: string,
      chunkCount: number
    ) => {
      const id = await invoke<string>("add_download", {
        url,
        filename,
        savePath,
        chunkCount,
      });
      await refresh();
      return id;
    },
    [refresh]
  );

  const pauseDownload = useCallback(
    async (id: string) => {
      await invoke("pause_download", { id });
      setDownloads((prev) =>
        prev.map((d) =>
          d.id === id ? { ...d, status: "paused", speed: 0 } : d
        )
      );
    },
    []
  );

  const resumeDownload = useCallback(
    async (id: string) => {
      await invoke("resume_download", { id });
      setDownloads((prev) =>
        prev.map((d) =>
          d.id === id ? { ...d, status: "queued" } : d
        )
      );
    },
    []
  );

  const cancelDownload = useCallback(async (id: string) => {
    await invoke("cancel_download", { id });
    setDownloads((prev) => prev.filter((d) => d.id !== id));
  }, []);

  const openFile = useCallback((path: string, filename: string) => {
    invoke("open_file", { path: `${path}/${filename}` });
  }, []);

  const openFolder = useCallback((path: string) => {
    invoke("open_folder", { path });
  }, []);

  const totalSpeed = downloads
    .filter((d) => d.status === "downloading")
    .reduce((sum, d) => sum + d.speed, 0);

  const activeCount = downloads.filter(
    (d) => d.status === "downloading"
  ).length;

  return {
    downloads,
    addDownload,
    pauseDownload,
    resumeDownload,
    cancelDownload,
    openFile,
    openFolder,
    totalSpeed,
    activeCount,
  };
}
