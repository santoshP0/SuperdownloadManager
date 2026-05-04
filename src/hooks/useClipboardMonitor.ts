import { useEffect, useRef } from "react";

const DOWNLOAD_EXT_RE =
  /\.(zip|rar|7z|tar|gz|bz2|xz|exe|msi|dmg|pkg|deb|rpm|iso|img|mp4|mkv|avi|mov|wmv|mp3|flac|wav|pdf|apk|jar|bin|run|sh)(\?.*)?$/i;

function looksLikeDownload(text: string): boolean {
  try {
    const url = new URL(text.trim());
    if (url.protocol !== "http:" && url.protocol !== "https:") return false;
    return DOWNLOAD_EXT_RE.test(url.pathname + url.search);
  } catch {
    return false;
  }
}

export function useClipboardMonitor(
  enabled: boolean,
  onDetected: (url: string) => void
) {
  const lastRef = useRef<string>("");
  const callbackRef = useRef(onDetected);
  callbackRef.current = onDetected;

  useEffect(() => {
    if (!enabled) return;

    const check = async () => {
      try {
        const text = await navigator.clipboard.readText();
        if (text && text !== lastRef.current && looksLikeDownload(text)) {
          lastRef.current = text;
          callbackRef.current(text.trim());
        }
      } catch {
        // Clipboard access denied or not available — silently skip
      }
    };

    const id = setInterval(check, 1500);
    return () => clearInterval(id);
  }, [enabled]);
}
