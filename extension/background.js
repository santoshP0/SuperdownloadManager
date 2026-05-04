const API = "http://localhost:9876";

// File extensions we intercept — skip .html, images, fonts, etc.
const CAPTURE_EXTS = new Set([
  "zip","rar","7z","tar","gz","bz2","xz","zst",
  "exe","msi","dmg","pkg","deb","rpm","apk",
  "mp4","mkv","avi","mov","wmv","webm","flv","m4v",
  "mp3","flac","wav","aac","ogg","m4a","opus",
  "pdf","epub","mobi","doc","docx","xls","xlsx","ppt","pptx",
  "iso","img","bin","run",
  "jar","whl","crx",
]);

function getExt(url, filename) {
  const name = filename || url.split("?")[0].split("/").pop() || "";
  return name.split(".").pop()?.toLowerCase() ?? "";
}

function shouldCapture(url, filename) {
  return CAPTURE_EXTS.has(getExt(url, filename));
}

chrome.downloads.onCreated.addListener(async (item) => {
  const { enabled = true } = await chrome.storage.local.get("enabled");
  if (!enabled) return;
  if (!shouldCapture(item.url, item.filename)) return;

  // Pause immediately to minimise data wasted if API call is slow
  chrome.downloads.pause(item.id);

  const filename =
    (item.filename ? item.filename.split(/[\\/]/).pop() : null) ||
    item.url.split("?")[0].split("/").pop() ||
    "download";

  try {
    const res = await fetch(`${API}/add`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ url: item.url, filename }),
    });

    if (res.ok) {
      // Hand off successful — kill the browser download
      chrome.downloads.cancel(item.id);
      setTimeout(() => chrome.downloads.erase({ id: item.id }), 600);

      chrome.notifications.create({
        type: "basic",
        title: "Super Download Manager",
        message: `Captured: ${filename}`,
        iconUrl: chrome.runtime.getURL("icon.png"),
      });
    } else {
      chrome.downloads.resume(item.id);
    }
  } catch {
    // App not running — let the browser handle it
    chrome.downloads.resume(item.id);
  }
});

// Badge: show active + queued count
async function updateBadge() {
  try {
    const res = await fetch(`${API}/status`, { signal: AbortSignal.timeout(1500) });
    const data = await res.json();
    const n = (data.active ?? 0) + (data.queued ?? 0);
    chrome.action.setBadgeText({ text: n > 0 ? String(n) : "" });
    chrome.action.setBadgeBackgroundColor({ color: "#3b82f6" });
  } catch {
    chrome.action.setBadgeText({ text: "" });
  }
}

setInterval(updateBadge, 2000);
updateBadge();
