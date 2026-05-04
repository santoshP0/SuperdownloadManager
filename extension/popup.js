const toggle = document.getElementById("toggle");
const dot    = document.getElementById("dot");
const msg    = document.getElementById("msg");

// Restore saved toggle state
chrome.storage.local.get({ enabled: true }, ({ enabled }) => {
  toggle.checked = enabled;
});
toggle.addEventListener("change", () => {
  chrome.storage.local.set({ enabled: toggle.checked });
  check();
});

async function check() {
  if (!toggle.checked) {
    dot.className = "";
    msg.textContent = "Capturing paused";
    return;
  }
  try {
    const res = await fetch("http://localhost:9876/status", {
      signal: AbortSignal.timeout(1500),
    });
    const { active = 0, queued = 0 } = await res.json();
    dot.className = "ok";
    const n = active + queued;
    msg.textContent = n > 0 ? `${n} download${n > 1 ? "s" : ""} in progress` : "Ready — app is running";
  } catch {
    dot.className = "err";
    msg.textContent = "App not running — open it first";
  }
}

check();
