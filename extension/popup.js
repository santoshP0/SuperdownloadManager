const API = "http://localhost:9876";

const toggle    = document.getElementById("toggle");
const dot       = document.getElementById("dot");
const statusTxt = document.getElementById("status-text");
const statsEl   = document.getElementById("stats");
const listEl    = document.getElementById("list");
const statActive = document.getElementById("stat-active");
const statQueued = document.getElementById("stat-queued");
const statDone   = document.getElementById("stat-done");

function fmt(bytes) {
  if (!bytes) return "0 B";
  const k = 1024, sizes = ["B","KB","MB","GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return (bytes / Math.pow(k, i)).toFixed(1) + " " + sizes[i];
}
function fmtSpeed(b) { return b > 0 ? fmt(b) + "/s" : "—"; }
function fmtPct(dl, total) { return total > 0 ? Math.round((dl / total) * 100) : 0; }

// Load stored toggle state
chrome.storage.local.get({ enabled: true }, ({ enabled }) => {
  toggle.checked = enabled;
});
toggle.addEventListener("change", () => {
  chrome.storage.local.set({ enabled: toggle.checked });
});

async function refresh() {
  try {
    const res = await fetch(`${API}/status`, { signal: AbortSignal.timeout(1500) });
    const data = await res.json();

    dot.className = "connected";
    statusTxt.textContent = "Connected to Super Download Manager";
    statsEl.classList.remove("hidden");

    const active = data.downloads.filter(d => d.status === "downloading").length;
    const queued = data.downloads.filter(d => d.status === "queued").length;
    const done   = data.downloads.filter(d => d.status === "completed").length;

    statActive.textContent = active;
    statQueued.textContent = queued;
    statDone.textContent   = done;

    const shown = data.downloads.slice(0, 8);
    if (shown.length === 0) {
      listEl.innerHTML = `<div class="dl-empty">No downloads yet</div>`;
    } else {
      listEl.innerHTML = shown.map(d => {
        const pct = fmtPct(d.downloaded, d.total_size);
        return `
          <li class="status-${d.status}">
            <div class="dl-name">${d.filename}</div>
            <div class="dl-bar"><div class="dl-bar-fill" style="width:${pct}%"></div></div>
            <div class="dl-meta">
              <span>${d.status}</span>
              <span>${d.status === "downloading" ? fmtSpeed(d.speed) : fmt(d.downloaded)}</span>
            </div>
          </li>`;
      }).join("");
    }
  } catch {
    dot.className = "disconnected";
    statusTxt.textContent = "App not running";
    statsEl.classList.add("hidden");
    listEl.innerHTML = `<div class="dl-empty">Start the desktop app to use SDM</div>`;
  }
}

refresh();
setInterval(refresh, 1500);
