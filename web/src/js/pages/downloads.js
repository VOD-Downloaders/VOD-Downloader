import { getDownloads, clearDownloads } from "../store.js";
import { getDownloadInfo } from "../api.js";
import { escapeHtml } from "../ui.js";

const POLL_INTERVAL_MS = 2000;

export async function render(view) {
    const downloads = getDownloads();

    view.innerHTML = `
        <div class="container-fluid p-4">
            <div class="d-flex justify-content-between align-items-center mb-3">
                <h1 class="h3 mb-0">Downloads</h1>
                ${downloads.length ? `<button class="btn btn-outline-danger btn-sm" data-clear>Clear list</button>` : ""}
            </div>
            ${downloads.length ? table(downloads) : `<p class="text-secondary">No downloads started yet.</p>`}
        </div>`;

    const clear = view.querySelector("[data-clear]");
    if (clear) {
        clear.addEventListener("click", () => {
            clearDownloads();
            render(view);
        });
    }

    if (downloads.length) {
        startPolling(view, downloads);
    }
}

function table(downloads) {
    const rows = downloads.map((entry) => `
        <tr data-row="${escapeHtml(String(entry.id))}">
            <td>${escapeHtml(String(entry.id))}</td>
            <td>${escapeHtml(entry.output_file)}</td>
            <td>${escapeHtml(entry.indexer)}</td>
            <td>${escapeHtml(entry.quality || "—")}</td>
            <td>${escapeHtml(new Date(entry.ts).toLocaleString())}</td>
            <td data-status><span class="text-secondary">Loading…</span></td>
        </tr>`).join("");

    return `
        <div class="table-responsive">
            <table class="table table-dark table-hover align-middle">
                <thead><tr><th>ID</th><th>Output file</th><th>Indexer</th><th>Quality</th><th>Started</th><th>Status</th></tr></thead>
                <tbody data-table>${rows}</tbody>
            </table>
        </div>`;
}

function startPolling(view, downloads) {
    const root = view.querySelector("[data-table]");

    const tick = async () => {
        // Router replaced the view (navigated away) — stop polling.
        if (!document.contains(root)) {
            clearInterval(timer);
            return;
        }

        const results = await Promise.allSettled(downloads.map((entry) => getDownloadInfo(entry.id)));

        let allTerminal = true;
        results.forEach((result, index) => {
            const cell = root.querySelector(`[data-row="${cssEscape(String(downloads[index].id))}"] [data-status]`);
            if (!cell) {
                return;
            }

            if (result.status === "rejected") {
                // 400 = id unknown (downloads are in-memory and lost on server restart).
                const expired = result.reason && result.reason.status === 400;
                cell.innerHTML = expired
                    ? `<span class="badge text-bg-secondary">Unknown / expired</span>`
                    : `<span class="badge text-bg-warning">Unavailable</span>`;
                return;
            }

            const status = result.value.download_status;
            cell.innerHTML = statusHtml(status);
            if (!isTerminal(status)) {
                allTerminal = false;
            }
        });

        if (allTerminal) {
            clearInterval(timer);
        }
    };

    const timer = setInterval(tick, POLL_INTERVAL_MS);
    tick();
}

function statusHtml(status) {
    if (typeof status === "string") {
        switch (status) {
            case "Pending":
                return `<span class="badge text-bg-secondary">Pending</span>`;
            case "Converting":
                return `<span class="badge text-bg-info">Converting</span>`;
            case "Finished":
                return `<span class="badge text-bg-success">Finished</span>`;
            default:
                return `<span class="badge text-bg-secondary">${escapeHtml(status)}</span>`;
        }
    }

    if (status && typeof status === "object") {
        if (status.DownloadingSegments) {
            const { amount, total } = status.DownloadingSegments;
            const percent = total > 0 ? Math.round((amount / total) * 100) : 0;
            return `
                <div class="d-flex align-items-center gap-2">
                    <div class="progress flex-grow-1" style="min-width: 120px; height: 1rem;">
                        <div class="progress-bar" role="progressbar" style="width: ${percent}%;"
                            aria-valuenow="${percent}" aria-valuemin="0" aria-valuemax="100"></div>
                    </div>
                    <span class="small text-secondary">${escapeHtml(String(amount))}/${escapeHtml(String(total))}</span>
                </div>`;
        }
        if ("Failed" in status) {
            return `<span class="badge text-bg-danger" title="${escapeHtml(String(status.Failed))}">Failed</span>
                <span class="small text-danger ms-1">${escapeHtml(String(status.Failed))}</span>`;
        }
    }

    return `<span class="badge text-bg-secondary">Unknown</span>`;
}

function isTerminal(status) {
    if (typeof status === "string") {
        return status === "Finished";
    }
    return Boolean(status && typeof status === "object" && "Failed" in status);
}

function cssEscape(value) {
    return window.CSS && window.CSS.escape ? window.CSS.escape(value) : value.replace(/["\\]/g, "\\$&");
}
