import {
    getIndexers, getSpecifications, refreshSpecifications, createIndexer, deleteIndexer,
} from "../api.js";
import { escapeHtml, escapeAttr, errorAlert, spinner, toast } from "../ui.js";

let specs = [];
let indexers = [];

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
                <button class="btn btn-outline-light" data-refresh-specs>Refresh specifications</button>
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
    wireForm(view);
    populateSpecSelect(view);
    renderTable(view);
}

function formHtml() {
    return `
        <form data-indexer-form>
            <input type="hidden" name="based_on">
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
            </div>
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
            <div class="mb-3">
                <label class="form-label">Segment headers</label>
                <div data-headers></div>
                <button type="button" class="btn btn-sm btn-outline-light mt-2" data-add-header>Add header</button>
            </div>
            <div class="d-flex gap-2">
                <button type="submit" class="btn btn-primary">Save indexer</button>
                <button type="button" class="btn btn-secondary" data-reset-form>Reset</button>
            </div>
        </form>`;
}

function headerRowHtml(key, value) {
    return `
        <div class="input-group input-group-sm mb-2" data-header-row>
            <input class="form-control" placeholder="Header" value="${escapeAttr(key)}" data-header-key>
            <input class="form-control" placeholder="Value" value="${escapeAttr(value)}" data-header-value>
            <button class="btn btn-outline-danger" type="button" data-remove-header aria-label="Remove header">&times;</button>
        </div>`;
}

function populateSpecSelect(view) {
    const select = view.querySelector("#spec-select");
    select.innerHTML = `<option value="">— none —</option>`
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
    form.based_on.value = spec.name;
    form.name.value = spec.name;
    form.uses_cloudflare.checked = spec.uses_cloudflare;

    const serverSelect = view.querySelector("#indexer-server");
    serverSelect.innerHTML = spec.servers.map((server) => `<option value="${escapeAttr(server)}">${escapeHtml(server)}</option>`).join("");

    const download = spec.download;
    form.segment_timeout.value = download.segment_download.segment_timeout;
    form.segment_attempts.value = download.segment_download.segment_attempts;
    form.remove_front_bytes.value = download.segment_post_download.remove_front_bytes;
    form.remove_back_bytes.value = download.segment_post_download.remove_back_bytes;

    setHeaders(view, download.segment_download.headers || {});
}

function setHeaders(view, headers) {
    const container = view.querySelector("[data-headers]");
    const entries = Object.entries(headers);
    container.innerHTML = entries.map(([key, value]) => headerRowHtml(key, value)).join("");
}

function wireRefresh(view) {
    const button = view.querySelector("[data-refresh-specs]");
    button.addEventListener("click", async () => {
        const original = button.textContent;
        button.disabled = true;
        button.textContent = "Refreshing...";

        try {
            specs = await refreshSpecifications();
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

function wireForm(view) {
    const form = view.querySelector("[data-indexer-form]");
    const headers = view.querySelector("[data-headers]");

    view.querySelector("[data-add-header]").addEventListener("click", () => {
        headers.insertAdjacentHTML("beforeend", headerRowHtml("", ""));
    });

    headers.addEventListener("click", (event) => {
        const remove = event.target.closest("[data-remove-header]");
        if (remove) {
            remove.closest("[data-header-row]").remove();
        }
    });

    view.querySelector("[data-reset-form]").addEventListener("click", () => resetForm(view));

    form.addEventListener("submit", async (event) => {
        event.preventDefault();
        const errorBox = view.querySelector("[data-form-error]");
        errorBox.innerHTML = "";

        const indexer = buildIndexer(view);
        if (!indexer.name) {
            errorBox.innerHTML = errorAlert("Name is required.");
            return;
        }
        if (!indexer.server) {
            errorBox.innerHTML = errorAlert("A server is required. Pick a specification first.");
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
    const headers = {};
    view.querySelectorAll("[data-header-row]").forEach((row) => {
        const key = row.querySelector("[data-header-key]").value.trim();
        if (key) {
            headers[key] = row.querySelector("[data-header-value]").value;
        }
    });

    return {
        name: form.name.value.trim(),
        server: form.server.value,
        uses_cloudflare: form.uses_cloudflare.checked,
        based_on: form.based_on.value,
        download: {
            segment_download: {
                segment_timeout: Number(form.segment_timeout.value),
                segment_attempts: Number(form.segment_attempts.value),
                headers,
            },
            segment_post_download: {
                remove_front_bytes: Number(form.remove_front_bytes.value),
                remove_back_bytes: Number(form.remove_back_bytes.value),
            },
        },
    };
}

function resetForm(view) {
    const form = view.querySelector("[data-indexer-form]");
    form.reset();
    form.based_on.value = "";
    view.querySelector("#spec-select").value = "";
    view.querySelector("#indexer-server").innerHTML = "";
    view.querySelector("[data-headers]").innerHTML = "";
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
            <td class="text-truncate" style="max-width: 200px">${escapeHtml(indexer.server)}</td>
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
                <thead><tr><th>Name</th><th>Server</th><th>Cloudflare</th><th>Based on</th><th></th></tr></thead>
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
    form.based_on.value = indexer.based_on || "";
    form.name.value = indexer.name;
    form.uses_cloudflare.checked = indexer.uses_cloudflare;

    const spec = specs.find((item) => item.name === indexer.based_on);
    const servers = new Set(spec ? spec.servers : []);
    servers.add(indexer.server);
    const serverSelect = view.querySelector("#indexer-server");
    serverSelect.innerHTML = [...servers].map((server) => `<option value="${escapeAttr(server)}">${escapeHtml(server)}</option>`).join("");
    serverSelect.value = indexer.server;

    form.segment_timeout.value = indexer.download.segment_download.segment_timeout;
    form.segment_attempts.value = indexer.download.segment_download.segment_attempts;
    form.remove_front_bytes.value = indexer.download.segment_post_download.remove_front_bytes;
    form.remove_back_bytes.value = indexer.download.segment_post_download.remove_back_bytes;
    setHeaders(view, indexer.download.segment_download.headers || {});

    const specIndex = specs.findIndex((item) => item.name === indexer.based_on);
    view.querySelector("#spec-select").value = specIndex >= 0 ? String(specIndex) : "";

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
