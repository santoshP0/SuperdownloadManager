export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(i === 0 ? 0 : 1)} ${sizes[i]}`;
}

export function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec === 0) return "—";
  return `${formatBytes(bytesPerSec)}/s`;
}

export function formatETA(seconds: number): string {
  if (seconds === 0 || seconds > 86400 * 365) return "—";
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  return `${h}h ${m}m`;
}

export function formatPercent(downloaded: number, total: number): number {
  if (total === 0) return 0;
  return Math.min(100, Math.round((downloaded / total) * 100));
}
