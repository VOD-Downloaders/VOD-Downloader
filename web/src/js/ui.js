// Shared UI helpers: escaping, TMDB images, spinners/alerts/toasts, the indexer stream-search
// toolbar, and the stream-results + download modals.
import { startDownload } from "./api.js";
import { addDownload, getLastIndexer, setLastIndexer } from "./store.js";

const TMDB_IMG = "https://image.tmdb.org/t/p";
const PLACEHOLDER = `data:image/svg+xml;charset=UTF-8,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="200" height="300"><rect width="100%" height="100%" fill="#2b3035"/>` +
    `<text x="50%" y="50%" fill="#6c757d" font-family="sans-serif" font-size="16" text-anchor="middle" dominant-baseline="middle">No image</text></svg>`,
)}`;

// --- Escaping ---------------------------------------------------------------
export function escapeHtml(value) {
    return String(value ?? "").replace(/[&<>"']/g, (char) => ({
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        "\"": "&quot;",
        "'": "&#39;",
    }[char]));
}

export function escapeAttr(value) {
    return escapeHtml(value);
}

// --- Images / formatting ----------------------------------------------------
export function posterImg(path) {
    return path ? `${TMDB_IMG}/w342${path}` : PLACEHOLDER;
}

export function stillImg(path) {
    return path ? `${TMDB_IMG}/w300${path}` : PLACEHOLDER;
}

export function backdropStyle(path) {
    if (!path) {
        return "";
    }
    return `background-image: linear-gradient(to right, rgb(13 17 23 / 92%), rgb(13 17 23 / 55%)), url('${TMDB_IMG}/w1280${path}');`;
}

export function yearOf(dateStr) {
    if (!dateStr) {
        return "";
    }
    const match = String(dateStr).match(/^(\d{4})/);
    return match ? match[1] : "";
}

export function pad(value) {
    return String(value).padStart(2, "0");
}

// --- Feedback ---------------------------------------------------------------
export function spinner(text = "Loading...") {
    return `<div class="d-flex align-items-center gap-2 p-4 text-secondary">
        <div class="spinner-border spinner-border-sm" role="status"></div><span>${escapeHtml(text)}</span></div>`;
}

export function errorAlert(message) {
    return `<div class="alert alert-danger">${escapeHtml(message)}</div>`;
}

export function toast(message) {
    let container = document.getElementById("toast-container");
    if (!container) {
        container = document.createElement("div");
        container.id = "toast-container";
        container.className = "toast-container position-fixed bottom-0 end-0 p-3";
        document.body.appendChild(container);
    }

    const element = document.createElement("div");
    element.className = "toast align-items-center text-bg-primary border-0";
    element.setAttribute("role", "alert");
    element.innerHTML = `<div class="d-flex">
        <div class="toast-body">${escapeHtml(message)}</div>
        <button type="button" class="btn-close btn-close-white me-2 m-auto" data-bs-dismiss="toast" aria-label="Close"></button>
    </div>`;
    container.appendChild(element);

    const instance = new bootstrap.Toast(element, { delay: 4000 });
    element.addEventListener("hidden.bs.toast", () => element.remove());
    instance.show();
}

function makeModal(title, bodyHtml) {
    const wrapper = document.createElement("div");
    wrapper.className = "modal fade";
    wrapper.tabIndex = -1;
    wrapper.innerHTML = `
        <div class="modal-dialog modal-lg modal-dialog-scrollable">
            <div class="modal-content">
                <div class="modal-header">
                    <h5 class="modal-title">${escapeHtml(title)}</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">${bodyHtml}</div>
            </div>
        </div>`;
    document.body.appendChild(wrapper);

    const modal = new bootstrap.Modal(wrapper);
    wrapper.addEventListener("hidden.bs.modal", () => wrapper.remove());
    return { wrapper, modal };
}

// --- Indexer stream-search toolbar -----------------------------------------
export function indexerActionsHtml(indexers) {
    if (!indexers || indexers.length === 0) {
        return `<div class="alert alert-warning mb-0">No indexers configured.
            <a href="#/indexers" class="alert-link">Add one</a> to search for streams.</div>`;
    }

    const options = indexers.map((indexer) => `<option value="${escapeAttr(indexer.name)}">${escapeHtml(indexer.name)}</option>`).join("");

    return `
        <div class="d-flex flex-wrap gap-2 align-items-center">
            <button type="button" class="btn btn-primary btn-sm" data-search-all>Search all indexers</button>
            <div class="input-group input-group-sm w-auto">
                <select class="form-select" data-indexer-select aria-label="Indexer">${options}</select>
                <button type="button" class="btn btn-outline-light" data-search-one>Search this indexer</button>
            </div>
        </div>`;
}

export function attachStreamSearch(scope, { indexers, titleHint, fetcher }) {
    const searchAll = scope.querySelector("[data-search-all]");
    const searchOne = scope.querySelector("[data-search-one]");
    const select = scope.querySelector("[data-indexer-select]");

    if (select) {
        const last = getLastIndexer();
        if (last && indexers.some((indexer) => indexer.name === last)) {
            select.value = last;
        }
    }

    if (searchAll) {
        searchAll.addEventListener("click", () => openStreamSearch({ indexers, selected: "all", titleHint, fetcher }));
    }

    if (searchOne) {
        searchOne.addEventListener("click", () => {
            setLastIndexer(select.value);
            openStreamSearch({ indexers, selected: select.value, titleHint, fetcher });
        });
    }
}

// --- Stream results + download ---------------------------------------------
export async function openStreamSearch({ indexers, selected, titleHint, fetcher }) {
    const { wrapper, modal } = makeModal(`Streams — ${titleHint}`, spinner("Searching for streams..."));
    modal.show();
    const body = wrapper.querySelector(".modal-body");

    const targets = selected === "all" ? indexers.map((indexer) => indexer.name) : [selected];

    const settled = await Promise.allSettled(
        targets.map((name) => fetcher(name).then((streams) => ({ name, streams }))),
    );

    const rows = [];
    const subtitles = [];
    const statuses = [];
    settled.forEach((result, index) => {
        const name = targets[index];

        if (result.status !== "fulfilled") {
            statuses.push({ indexer: name, state: "failed", detail: result.reason?.message || "request failed" });
            return;
        }

        const { streams } = result.value;
        const streamList = streams.streams || [];
        for (const stream of streamList) {
            rows.push({ indexer: name, stream });
        }
        for (const subtitle of streams.subtitles || []) {
            subtitles.push({ indexer: name, subtitle });
        }
        statuses.push({ indexer: name, state: streamList.length ? "ok" : "empty", detail: `${streamList.length} stream(s)` });
    });

    body.innerHTML = renderStreamTable(rows, subtitles, statuses);
    body.querySelectorAll("[data-stream-row]").forEach((button) => {
        button.addEventListener("click", () => {
            const { indexer, stream } = rows[Number(button.dataset.streamRow)];
            openDownloadModal({ indexer, stream, titleHint });
        });
    });
}

function renderStreamTable(rows, subtitles, statuses) {
    const statusBlock = renderStatusBlock(statuses);

    if (rows.length === 0) {
        return `${statusBlock}<div class="alert alert-warning mb-0">No streams found.</div>`;
    }

    const streamRows = rows.map((row, index) => `
        <tr>
            <td>${escapeHtml(row.stream.quality || "unknown")}</td>
            <td>${escapeHtml(row.indexer)}</td>
            <td class="text-truncate" style="max-width: 320px"><span class="small text-secondary">${escapeHtml(row.stream.url)}</span></td>
            <td class="text-end"><button type="button" class="btn btn-sm btn-primary" data-stream-row="${index}">Download</button></td>
        </tr>`).join("");

    const subtitleBlock = subtitles.length === 0 ? "" : `
        <h6 class="mt-4">Subtitles</h6>
        <ul class="list-unstyled small text-secondary mb-0">
            ${subtitles.map((item) => `<li>${escapeHtml(item.subtitle.language)} — ${escapeHtml(item.subtitle.url)} (${escapeHtml(item.indexer)})</li>`).join("")}
        </ul>`;

    return `
        ${statusBlock}
        <div class="table-responsive">
            <table class="table table-dark table-hover align-middle">
                <thead><tr><th>Quality</th><th>Indexer</th><th>URL</th><th></th></tr></thead>
                <tbody>${streamRows}</tbody>
            </table>
        </div>
        ${subtitleBlock}`;
}

const STATUS_BADGE = {
    ok: "text-bg-success",
    empty: "text-bg-secondary",
    failed: "text-bg-danger",
};

function renderStatusBlock(statuses) {
    if (!statuses || statuses.length <= 1) {
        return "";
    }

    const badges = statuses.map((status) => `
        <span class="badge ${STATUS_BADGE[status.state]} d-inline-flex gap-1" title="${escapeAttr(status.detail)}">
            ${escapeHtml(status.indexer)}<span class="opacity-75">${escapeHtml(status.detail)}</span>
        </span>`).join("");

    return `<div class="d-flex flex-wrap gap-2 mb-3">${badges}</div>`;
}

export function openDownloadModal({ indexer, stream, titleHint }) {
    const suggested = suggestFilename(titleHint, stream.quality);
    const { wrapper, modal } = makeModal("Start download", `
        <dl class="row mb-3">
            <dt class="col-sm-3 text-secondary">Indexer</dt><dd class="col-sm-9">${escapeHtml(indexer)}</dd>
            <dt class="col-sm-3 text-secondary">Quality</dt><dd class="col-sm-9">${escapeHtml(stream.quality || "unknown")}</dd>
        </dl>
        <div class="mb-3">
            <label class="form-label" for="download-filename">Output file</label>
            <input id="download-filename" type="text" class="form-control" value="${escapeAttr(suggested)}">
        </div>
        <div data-download-error></div>
        <div class="d-flex justify-content-end gap-2">
            <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">Cancel</button>
            <button type="button" class="btn btn-primary" data-download-start>Download</button>
        </div>`);
    modal.show();

    const input = wrapper.querySelector("#download-filename");
    const startButton = wrapper.querySelector("[data-download-start]");
    const errorBox = wrapper.querySelector("[data-download-error]");

    startButton.addEventListener("click", async () => {
        const outputFile = input.value.trim();
        if (!outputFile) {
            input.classList.add("is-invalid");
            return;
        }

        startButton.disabled = true;
        errorBox.innerHTML = "";

        try {
            const result = await startDownload(indexer, { stream, output_file: outputFile });
            addDownload({ id: result.id, output_file: outputFile, indexer, quality: stream.quality || "", ts: Date.now() });
            toast(`Download started (id ${result.id}).`);

            wrapper.querySelector(".modal-body").innerHTML = `
                <div class="alert alert-success mb-3">Download started (id ${escapeHtml(String(result.id))}).</div>
                <div class="d-flex justify-content-end gap-2">
                    <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">Close</button>
                    <a href="#/downloads" class="btn btn-primary" data-bs-dismiss="modal">View downloads</a>
                </div>`;
        } catch (error) {
            startButton.disabled = false;
            errorBox.innerHTML = errorAlert(error.message);
        }
    });
}

function suggestFilename(titleHint, quality) {
    const base = String(titleHint || "download")
        .replace(/[^\w.-]+/g, ".")
        .replace(/\.+/g, ".")
        .replace(/^\.|\.$/g, "");
    const qualityPart = quality ? `.${String(quality).replace(/[^\w]+/g, "")}` : "";
    return `${base}${qualityPart}.ts`;
}
