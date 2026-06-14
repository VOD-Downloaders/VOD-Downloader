import { getIndexers } from "../api.js";
import { getDownloads } from "../store.js";

export async function render(view) {
    view.innerHTML = `
        <div class="container-fluid p-4">
            <h1 class="h3 mb-4">FMHY Downloader</h1>
            <div class="row g-3 mb-4">
                <div class="col-sm-6 col-lg-4">
                    <div class="card h-100"><div class="card-body">
                        <div class="text-secondary small text-uppercase">Active indexers</div>
                        <div class="display-6" data-stat="indexers">—</div>
                    </div></div>
                </div>
                <div class="col-sm-6 col-lg-4">
                    <div class="card h-100"><div class="card-body">
                        <div class="text-secondary small text-uppercase">Downloads started</div>
                        <div class="display-6" data-stat="downloads">—</div>
                    </div></div>
                </div>
            </div>
            <div class="d-flex gap-2">
                <a class="btn btn-primary" href="#/search">Search titles</a>
                <a class="btn btn-outline-light" href="#/indexers">Manage indexers</a>
            </div>
        </div>`;

    view.querySelector('[data-stat="downloads"]').textContent = getDownloads().length;

    try {
        const indexers = await getIndexers();
        view.querySelector('[data-stat="indexers"]').textContent = indexers.length;
    } catch {
        view.querySelector('[data-stat="indexers"]').textContent = "!";
    }
}
