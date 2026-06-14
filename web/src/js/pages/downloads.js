import { getDownloads, clearDownloads } from "../store.js";
import { escapeHtml } from "../ui.js";

export async function render(view) {
    const downloads = getDownloads();

    view.innerHTML = `
        <div class="container-fluid p-4">
            <div class="d-flex justify-content-between align-items-center mb-3">
                <h1 class="h3 mb-0">Downloads</h1>
                ${downloads.length ? `<button class="btn btn-outline-danger btn-sm" data-clear>Clear list</button>` : ""}
            </div>
            <div class="alert alert-info">
                Live progress and status are not available yet — the backend does not expose a download-status
                endpoint. This list only reflects downloads started from this browser.
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
}

function table(downloads) {
    const rows = downloads.map((entry) => `
        <tr>
            <td>${escapeHtml(String(entry.id))}</td>
            <td>${escapeHtml(entry.output_file)}</td>
            <td>${escapeHtml(entry.indexer)}</td>
            <td>${escapeHtml(entry.quality || "—")}</td>
            <td>${escapeHtml(new Date(entry.ts).toLocaleString())}</td>
        </tr>`).join("");

    return `
        <div class="table-responsive">
            <table class="table table-dark table-hover align-middle">
                <thead><tr><th>ID</th><th>Output file</th><th>Indexer</th><th>Quality</th><th>Started</th></tr></thead>
                <tbody>${rows}</tbody>
            </table>
        </div>`;
}
