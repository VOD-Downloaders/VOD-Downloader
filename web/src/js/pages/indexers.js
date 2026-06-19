import {
    getIndexers, getSpecifications, refetchSpecifications, reloadSpecifications, reloadIndexers, createIndexer, deleteIndexer,
} from "../api.js";
import { escapeHtml, escapeAttr, errorAlert, spinner, toast } from "../ui.js";

let specs = [];
let indexers = [];

// Template carried into the form for the current edit/create. The non-tunable parts of an indexer
// (algorithm_name, search, stream, based_on, segment headers) come straight from the chosen
// specification or the indexer being edited — the form only exposes the server choice, cloudflare
// flag and download tuning.
let formBase = null;

export async function render(view) {
    view.innerHTML = spinner("Loading indexers...");

    try {
        [indexers, specs] = await Promise.all([getIndexers(), getSpecifications()]);
    } catch (error) {
        view.innerHTML = errorAlert(error.message);
        return;
    }

    view.innerHTML = `
        <div class="container-fluid p-4">
            <div class="d-flex justify-content-between align-items-center mb-4">
                <h1 class="h3 mb-0">Indexers</h1>
                <div class="d-flex gap-2">
                    <button class="btn btn-outline-light" data-reload-disk>Reload indexers</button>
                    <button class="btn btn-outline-light" data-refresh-specs>Update specifications</button>
                </div>
            </div>
            <div class="row g-4">
                <div class="col-lg-5">
                    <div class="card">
                        <div class="card-body">
                            <h2 class="h5 mb-3" data-form-title>Create indexer</h2>
                            <div data-form-error></div>
                            ${formHtml()}
                        </div>
                    </div>
                </div>
                <div class="col-lg-7">
                    <h2 class="h5 mb-3">Active indexers</h2>
                    <div data-table></div>
                </div>
            </div>
        </div>`;

    wireRefresh(view);
    wireReload(view);
    wireForm(view);
    populateSpecSelect(view);
    renderTable(view);
}

function formHtml() {
    return `
        <form data-indexer-form>
            <div class="mb-3">
                <label class="form-label" for="spec-select">Based on specification</label>
                <select class="form-select" id="spec-select" name="spec"></select>
            </div>
            <div class="mb-3">
                <label class="form-label" for="indexer-name">Name</label>
                <input class="form-control" id="indexer-name" name="name" required>
            </div>
            <div class="mb-3">
                <label class="form-label" for="indexer-server">Server</label>
                <select class="form-select" id="indexer-server" name="server"></select>
                <div class="form-text" data-server-description></div>
            </div>
            <dl class="row small text-secondary mb-3" data-spec-info hidden>
                <dt class="col-sm-4">Algorithm</dt><dd class="col-sm-8" data-info-algorithm></dd>
                <dt class="col-sm-4">Stream type</dt><dd class="col-sm-8" data-info-stream-type></dd>
                <dt class="col-sm-4">Search URL</dt><dd class="col-sm-8 text-truncate" data-info-search-url></dd>
            </dl>
            <div class="form-check form-switch mb-3">
                <input class="form-check-input" type="checkbox" id="indexer-cf" name="uses_cloudflare">
                <label class="form-check-label" for="indexer-cf">Uses Cloudflare</label>
            </div>
            <div class="row g-2 mb-3">
                <div class="col">
                    <label class="form-label" for="seg-timeout">Segment timeout (s)</label>
                    <input type="number" min="0" class="form-control" id="seg-timeout" name="segment_timeout" value="5">
                </div>
                <div class="col">
                    <label class="form-label" for="seg-attempts">Segment attempts</label>
                    <input type="number" min="0" class="form-control" id="seg-attempts" name="segment_attempts" value="5">
                </div>
            </div>
            <div class="row g-2 mb-3">
                <div class="col">
                    <label class="form-label" for="rm-front">Remove front bytes</label>
                    <input type="number" min="0" class="form-control" id="rm-front" name="remove_front_bytes" value="0">
                </div>
                <div class="col">
                    <label class="form-label" for="rm-back">Remove back bytes</label>
                    <input type="number" min="0" class="form-control" id="rm-back" name="remove_back_bytes" value="0">
                </div>
            </div>
            <div class="d-flex gap-2">
                <button type="submit" class="btn btn-primary">Save indexer</button>
                <button type="button" class="btn btn-secondary" data-reset-form>Reset</button>
            </div>
        </form>`;
}

function populateSpecSelect(view) {
    const select = view.querySelector("#spec-select");
    select.innerHTML = `<option value="">— select —</option>`
        + specs.map((spec, index) => `<option value="${index}">${escapeHtml(spec.name)}</option>`).join("");

    select.addEventListener("change", () => {
        if (select.value === "") {
            return;
        }
        applySpec(view, specs[Number(select.value)]);
    });
}

function applySpec(view, spec) {
    const form = view.querySelector("[data-indexer-form]");

    formBase = {
        algorithm_name: spec.algorithm_name,
        search: spec.search,
        stream: spec.stream,
        based_on: spec.name,
        servers: spec.server_list,
        headers: spec.download.segment_download.headers || {},
    };

    form.name.value = spec.name;
    form.uses_cloudflare.checked = spec.uses_cloudflare;

    populateServerSelect(view, spec.server_list, spec.server_list[0]?.name);

    const download = spec.download;
    form.segment_timeout.value = download.segment_download.segment_timeout;
    form.segment_attempts.value = download.segment_download.segment_attempts;
    form.remove_front_bytes.value = download.segment_post_download.remove_front_bytes;
    form.remove_back_bytes.value = download.segment_post_download.remove_back_bytes;

    updateServerInfo(view);
}

function populateServerSelect(view, servers, selectedName) {
    const serverSelect = view.querySelector("#indexer-server");
    serverSelect.innerHTML = servers
        .map((server, index) => `<option value="${index}">${escapeHtml(server.name)}</option>`)
        .join("");

    const selectedIndex = servers.findIndex((server) => server.name === selectedName);
    serverSelect.value = String(selectedIndex >= 0 ? selectedIndex : 0);
}

function updateServerInfo(view) {
    if (!formBase) {
        return;
    }

    const serverSelect = view.querySelector("#indexer-server");
    const server = formBase.servers[Number(serverSelect.value)];

    view.querySelector("[data-server-description]").textContent = server?.description || "";

    const info = view.querySelector("[data-spec-info]");
    info.hidden = false;
    view.querySelector("[data-info-algorithm]").textContent = formBase.algorithm_name;
    view.querySelector("[data-info-stream-type]").textContent = formBase.stream.type;
    view.querySelector("[data-info-search-url]").textContent = server?.search_url || "";
}

function wireRefresh(view) {
    const button = view.querySelector("[data-refresh-specs]");
    button.addEventListener("click", async () => {
        const original = button.textContent;
        button.disabled = true;
        button.textContent = "Refreshing...";

        try {
            await refetchSpecifications();
            specs = await getSpecifications();
            populateSpecSelect(view);
            toast("Specifications refreshed.");
        } catch (error) {
            view.querySelector("[data-form-error]").innerHTML = errorAlert(error.message);
        } finally {
            button.disabled = false;
            button.textContent = original;
        }
    });
}

function wireReload(view) {
    const button = view.querySelector("[data-reload-disk]");
    button.addEventListener("click", async () => {
        const original = button.textContent;
        button.disabled = true;
        button.textContent = "Reloading...";

        try {
            await Promise.all([reloadIndexers(), reloadSpecifications()]);
            [indexers, specs] = await Promise.all([getIndexers(), getSpecifications()]);
            renderTable(view);
            populateSpecSelect(view);
            toast("Reloaded indexers from disk.");
        } catch (error) {
            view.querySelector("[data-form-error]").innerHTML = errorAlert(error.message);
        } finally {
            button.disabled = false;
            button.textContent = original;
        }
    });
}

function wireForm(view) {
    const form = view.querySelector("[data-indexer-form]");

    view.querySelector("#indexer-server").addEventListener("change", () => updateServerInfo(view));

    view.querySelector("[data-reset-form]").addEventListener("click", () => resetForm(view));

    form.addEventListener("submit", async (event) => {
        event.preventDefault();
        const errorBox = view.querySelector("[data-form-error]");
        errorBox.innerHTML = "";

        if (!formBase) {
            errorBox.innerHTML = errorAlert("Pick a specification first.");
            return;
        }

        const indexer = buildIndexer(view);
        if (!indexer.name) {
            errorBox.innerHTML = errorAlert("Name is required.");
            return;
        }
        if (!indexer.server) {
            errorBox.innerHTML = errorAlert("A server is required.");
            return;
        }

        try {
            await createIndexer(indexer);
            toast(`Saved indexer "${indexer.name}".`);
            indexers = await getIndexers();
            renderTable(view);
            resetForm(view);
        } catch (error) {
            errorBox.innerHTML = errorAlert(error.message);
        }
    });
}

function buildIndexer(view) {
    const form = view.querySelector("[data-indexer-form]");
    const serverSelect = view.querySelector("#indexer-server");
    const server = formBase.servers[Number(serverSelect.value)];

    return {
        name: form.name.value.trim(),
        algorithm_name: formBase.algorithm_name,
        server,
        uses_cloudflare: form.uses_cloudflare.checked,
        search: formBase.search,
        stream: formBase.stream,
        download: {
            segment_download: {
                segment_timeout: Number(form.segment_timeout.value),
                segment_attempts: Number(form.segment_attempts.value),
                headers: formBase.headers,
            },
            segment_post_download: {
                remove_front_bytes: Number(form.remove_front_bytes.value),
                remove_back_bytes: Number(form.remove_back_bytes.value),
            },
        },
        based_on: formBase.based_on,
    };
}

function resetForm(view) {
    const form = view.querySelector("[data-indexer-form]");
    form.reset();
    formBase = null;
    view.querySelector("#spec-select").value = "";
    view.querySelector("#indexer-server").innerHTML = "";
    view.querySelector("[data-server-description]").textContent = "";
    view.querySelector("[data-spec-info]").hidden = true;
    view.querySelector("[data-form-title]").textContent = "Create indexer";
}

function renderTable(view) {
    const container = view.querySelector("[data-table]");

    if (indexers.length === 0) {
        container.innerHTML = `<p class="text-secondary">No indexers yet. Create one from a specification.</p>`;
        return;
    }

    const rows = indexers.map((indexer) => `
        <tr>
            <td>${escapeHtml(indexer.name)}</td>
            <td class="text-truncate" style="max-width: 160px">${escapeHtml(indexer.server?.name || "")}</td>
            <td>${escapeHtml(indexer.algorithm_name || "")}</td>
            <td>${indexer.uses_cloudflare ? "Yes" : "No"}</td>
            <td>${escapeHtml(indexer.based_on || "")}</td>
            <td class="text-end text-nowrap">
                <button class="btn btn-sm btn-outline-light" data-edit="${escapeAttr(indexer.name)}">Edit</button>
                <button class="btn btn-sm btn-outline-danger" data-delete="${escapeAttr(indexer.name)}">Delete</button>
            </td>
        </tr>`).join("");

    container.innerHTML = `
        <div class="table-responsive">
            <table class="table table-dark table-hover align-middle">
                <thead><tr><th>Name</th><th>Server</th><th>Algorithm</th><th>Cloudflare</th><th>Based on</th><th></th></tr></thead>
                <tbody>${rows}</tbody>
            </table>
        </div>`;

    container.querySelectorAll("[data-edit]").forEach((button) => {
        button.addEventListener("click", () => loadForEdit(view, button.dataset.edit));
    });
    container.querySelectorAll("[data-delete]").forEach((button) => {
        button.addEventListener("click", () => removeIndexer(view, button.dataset.delete));
    });
}

function loadForEdit(view, name) {
    const indexer = indexers.find((item) => item.name === name);
    if (!indexer) {
        return;
    }

    const form = view.querySelector("[data-indexer-form]");
    view.querySelector("[data-form-title]").textContent = `Edit indexer: ${indexer.name}`;

    // Prefer the source specification's server list so other servers stay selectable; fall back to
    // the indexer's own server when the spec is gone. Always make sure the saved server is present.
    const spec = specs.find((item) => item.name === indexer.based_on);
    const servers = spec ? spec.server_list.slice() : [];
    if (!servers.some((server) => server.name === indexer.server.name)) {
        servers.unshift(indexer.server);
    }

    formBase = {
        algorithm_name: indexer.algorithm_name,
        search: indexer.search,
        stream: indexer.stream,
        based_on: indexer.based_on,
        servers,
        headers: indexer.download.segment_download.headers || {},
    };

    form.name.value = indexer.name;
    form.uses_cloudflare.checked = indexer.uses_cloudflare;

    populateServerSelect(view, servers, indexer.server.name);

    form.segment_timeout.value = indexer.download.segment_download.segment_timeout;
    form.segment_attempts.value = indexer.download.segment_download.segment_attempts;
    form.remove_front_bytes.value = indexer.download.segment_post_download.remove_front_bytes;
    form.remove_back_bytes.value = indexer.download.segment_post_download.remove_back_bytes;

    const specIndex = specs.findIndex((item) => item.name === indexer.based_on);
    view.querySelector("#spec-select").value = specIndex >= 0 ? String(specIndex) : "";

    updateServerInfo(view);
    form.scrollIntoView({ behavior: "smooth" });
}

async function removeIndexer(view, name) {
    if (!window.confirm(`Delete indexer "${name}"?`)) {
        return;
    }

    try {
        await deleteIndexer(name);
        toast(`Deleted "${name}".`);
        indexers = await getIndexers();
        renderTable(view);
    } catch (error) {
        view.querySelector("[data-table]").insertAdjacentHTML("afterbegin", errorAlert(error.message));
    }
}
