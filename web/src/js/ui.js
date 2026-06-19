// Shared UI helpers: escaping, TMDB images, spinners/alerts/toasts, the indexer stream-search
// toolbar, and the stream-results + download modals.
import { startDownload, getEpisodeStreams } from "./api.js";
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
// Map a quality string ("1080p", "4K", "HD"...) to a numeric rank for sorting. Unknown -> 0.
export function qualityRank(quality) {
    const text = String(quality || "").toLowerCase();
    if (text.includes("2160") || text.includes("4k")) {
        return 2160;
    }
    if (text.includes("1440")) {
        return 1440;
    }
    const match = text.match(/(\d{3,4})/);
    return match ? Number(match[1]) : 0;
}

function qualitySortSelect(qualityDir) {
    return `
        <label class="small text-secondary">Quality</label>
        <select class="form-select form-select-sm w-auto" data-quality-sort aria-label="Sort by quality">
            <option value="desc"${qualityDir === "desc" ? " selected" : ""}>High → Low</option>
            <option value="asc"${qualityDir === "asc" ? " selected" : ""}>Low → High</option>
        </select>`;
}

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

    let qualityDir = "desc";
    const draw = () => {
        const sorted = [...rows].sort((a, b) => {
            const factor = qualityDir === "asc" ? 1 : -1;
            return factor * (qualityRank(a.stream.quality) - qualityRank(b.stream.quality));
        });
        body.innerHTML = renderStreamTable(sorted, subtitles, statuses, qualityDir);
        const sortSelect = body.querySelector("[data-quality-sort]");
        if (sortSelect) {
            sortSelect.addEventListener("change", (event) => {
                qualityDir = event.target.value;
                draw();
            });
        }
        body.querySelectorAll("[data-stream-row]").forEach((button) => {
            button.addEventListener("click", () => {
                const { indexer, stream } = sorted[Number(button.dataset.streamRow)];
                openDownloadModal({ indexer, stream, titleHint });
            });
        });
    };
    draw();
}

function renderStreamTable(rows, subtitles, statuses, qualityDir) {
    const statusBlock = renderStatusBlock(statuses);

    if (rows.length === 0) {
        return `${statusBlock}<div class="alert alert-warning mb-0">No streams found.</div>`;
    }

    const toolbar = `<div class="d-flex flex-wrap gap-2 align-items-center mb-3">${qualitySortSelect(qualityDir)}</div>`;

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
        ${toolbar}
        <div class="table-responsive">
            <table class="table table-dark table-hover align-middle">
                <thead><tr><th>Quality</th><th>Indexer</th><th>URL</th><th></th></tr></thead>
                <tbody>${streamRows}</tbody>
            </table>
        </div>
        ${subtitleBlock}`;
}

// --- Season-wide stream search (all episodes at once) ----------------------
export async function openSeasonStreamSearch({ indexers, selected, seriesName, seriesId, seasonNumber, episodes }) {
    const { wrapper, modal } = makeModal(`Season ${seasonNumber} — ${seriesName}`, spinner("Searching all episodes for streams..."));
    modal.show();
    const body = wrapper.querySelector(".modal-body");

    const targets = selected === "all" ? indexers.map((indexer) => indexer.name) : [selected];

    // One request per (episode, indexer); keep a parallel meta array so rejected promises stay attributable.
    const meta = [];
    const jobs = [];
    for (const episode of episodes) {
        for (const name of targets) {
            meta.push({ episode, name });
            jobs.push(getEpisodeStreams(name, seriesId, seasonNumber, episode.episode_number));
        }
    }

    const settled = await Promise.allSettled(jobs);

    const rows = [];
    const statusMap = new Map();
    settled.forEach((result, index) => {
        const { episode, name } = meta[index];
        const status = statusMap.get(name) || { ok: 0, failed: 0 };

        if (result.status !== "fulfilled") {
            status.failed += 1;
        } else {
            const streamList = result.value.streams || [];
            status.ok += streamList.length;
            for (const stream of streamList) {
                rows.push({ episodeNumber: episode.episode_number, episodeName: episode.name, indexer: name, stream });
            }
        }
        statusMap.set(name, status);
    });

    const statuses = [...statusMap.entries()].map(([name, status]) => ({
        indexer: name,
        state: status.ok ? "ok" : status.failed ? "failed" : "empty",
        detail: `${status.ok} stream(s)${status.failed ? `, ${status.failed} error(s)` : ""}`,
    }));

    let qualityDir = "desc";
    let episodeDir = "asc";

    const draw = () => {
        if (rows.length === 0) {
            body.innerHTML = `${renderStatusBlock(statuses)}<div class="alert alert-warning mb-0">No streams found for this season.</div>`;
            return;
        }

        const sorted = [...rows].sort((a, b) => {
            if (a.episodeNumber !== b.episodeNumber) {
                return (episodeDir === "asc" ? 1 : -1) * (a.episodeNumber - b.episodeNumber);
            }
            return (qualityDir === "asc" ? 1 : -1) * (qualityRank(a.stream.quality) - qualityRank(b.stream.quality));
        });

        body.innerHTML = `
            ${renderStatusBlock(statuses)}
            <div class="d-flex flex-wrap gap-2 align-items-center mb-3">
                <label class="small text-secondary">Episode</label>
                <select class="form-select form-select-sm w-auto" data-episode-sort aria-label="Sort by episode">
                    <option value="asc"${episodeDir === "asc" ? " selected" : ""}>Ascending</option>
                    <option value="desc"${episodeDir === "desc" ? " selected" : ""}>Descending</option>
                </select>
                ${qualitySortSelect(qualityDir)}
                <button type="button" class="btn btn-sm btn-success ms-auto" data-download-all>Download all…</button>
            </div>
            <div class="table-responsive">
                <table class="table table-dark table-hover align-middle">
                    <thead><tr><th>Episode</th><th>Quality</th><th>Indexer</th><th>URL</th><th></th></tr></thead>
                    <tbody>${sorted.map((row, index) => `
                        <tr>
                            <td>E${pad(row.episodeNumber)}</td>
                            <td>${escapeHtml(row.stream.quality || "unknown")}</td>
                            <td>${escapeHtml(row.indexer)}</td>
                            <td class="text-truncate" style="max-width: 280px"><span class="small text-secondary">${escapeHtml(row.stream.url)}</span></td>
                            <td class="text-end"><button type="button" class="btn btn-sm btn-primary" data-stream-row="${index}">Download</button></td>
                        </tr>`).join("")}</tbody>
                </table>
            </div>`;

        body.querySelector("[data-episode-sort]").addEventListener("change", (event) => {
            episodeDir = event.target.value;
            draw();
        });
        body.querySelector("[data-quality-sort]").addEventListener("change", (event) => {
            qualityDir = event.target.value;
            draw();
        });
        body.querySelector("[data-download-all]").addEventListener("click", () => {
            openBulkDownload({ rows, episodes, seriesName, seasonNumber });
        });
        body.querySelectorAll("[data-stream-row]").forEach((button) => {
            button.addEventListener("click", () => {
                const row = sorted[Number(button.dataset.streamRow)];
                const titleHint = `${seriesName} S${pad(seasonNumber)}E${pad(row.episodeNumber)} ${row.episodeName}`;
                openDownloadModal({ indexer: row.indexer, stream: row.stream, titleHint });
            });
        });
    };
    draw();
}

// Prompt for a single quality, then start that quality for every episode that has it.
function openBulkDownload({ rows, episodes, seriesName, seasonNumber }) {
    const qualities = [...new Set(rows.map((row) => row.stream.quality || "unknown"))]
        .sort((a, b) => qualityRank(b) - qualityRank(a));
    const options = qualities.map((quality) => `<option value="${escapeAttr(quality)}">${escapeHtml(quality)}</option>`).join("");

    const { wrapper, modal } = makeModal("Download all episodes", `
        <p class="text-secondary">Pick a quality. Every episode with a matching stream starts downloading; episodes without it are skipped.</p>
        <div class="mb-3">
            <label class="form-label" for="bulk-quality">Quality</label>
            <select id="bulk-quality" class="form-select">${options}</select>
        </div>
        <div data-bulk-result></div>
        <div class="d-flex justify-content-end gap-2">
            <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">Cancel</button>
            <button type="button" class="btn btn-success" data-bulk-start>Download all</button>
        </div>`);
    modal.show();

    const select = wrapper.querySelector("#bulk-quality");
    const startButton = wrapper.querySelector("[data-bulk-start]");
    const resultBox = wrapper.querySelector("[data-bulk-result]");

    startButton.addEventListener("click", async () => {
        const quality = select.value;
        startButton.disabled = true;
        resultBox.innerHTML = "";

        const episodeNumbers = [...new Set(episodes.map((episode) => episode.episode_number))].sort((a, b) => a - b);
        const started = [];
        const missing = [];
        const failed = [];

        for (const episodeNumber of episodeNumbers) {
            const match = rows.find((row) => row.episodeNumber === episodeNumber && (row.stream.quality || "unknown") === quality);
            if (!match) {
                missing.push(episodeNumber);
                continue;
            }

            const titleHint = `${seriesName} S${pad(seasonNumber)}E${pad(episodeNumber)} ${match.episodeName}`;
            const outputFile = suggestFilename(titleHint, match.stream.quality);
            try {
                const result = await startDownload(match.indexer, { stream: match.stream, output_file: outputFile });
                addDownload({ id: result.id, output_file: outputFile, indexer: match.indexer, quality: match.stream.quality || "", ts: Date.now() });
                started.push(episodeNumber);
            } catch {
                failed.push(episodeNumber);
            }
        }

        toast(`Started ${started.length} download(s).`);
        let html = `<div class="alert alert-success">Started ${started.length} download(s) at ${escapeHtml(quality)}.</div>`;
        if (missing.length) {
            html += `<div class="alert alert-warning mb-0">No ${escapeHtml(quality)} stream for episode(s): ${missing.map((number) => `E${pad(number)}`).join(", ")}.</div>`;
        }
        if (failed.length) {
            html += `<div class="alert alert-danger mb-0 mt-2">Failed to start episode(s): ${failed.map((number) => `E${pad(number)}`).join(", ")}.</div>`;
        }
        resultBox.innerHTML = html;
        startButton.disabled = false;
    });
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
