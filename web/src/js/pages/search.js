import { searchMovies, searchSeries } from "../api.js";
import { posterImg, yearOf, escapeHtml, spinner, errorAlert } from "../ui.js";

const state = { kind: "movie", query: "", page: 1 };

export async function render(view) {
    view.innerHTML = `
        <div class="container-fluid p-4">
            <h1 class="h3 mb-4">Search</h1>
            <form class="row g-2 align-items-center mb-4" data-search-form>
                <div class="col-auto">
                    <div class="btn-group" role="group" aria-label="Type">
                        <input type="radio" class="btn-check" name="kind" id="kind-movie" value="movie" checked>
                        <label class="btn btn-outline-light" for="kind-movie">Movies</label>
                        <input type="radio" class="btn-check" name="kind" id="kind-series" value="series">
                        <label class="btn btn-outline-light" for="kind-series">Series</label>
                    </div>
                </div>
                <div class="col">
                    <input type="text" class="form-control" name="query" placeholder="Search for a title..." autocomplete="off">
                </div>
                <div class="col-auto">
                    <button type="submit" class="btn btn-primary">Search</button>
                </div>
            </form>
            <div data-results></div>
        </div>`;

    const form = view.querySelector("[data-search-form]");
    const results = view.querySelector("[data-results]");

    form.querySelector(`#kind-${state.kind}`).checked = true;
    form.querySelector('input[name="query"]').value = state.query;

    form.addEventListener("submit", (event) => {
        event.preventDefault();
        state.kind = form.querySelector('input[name="kind"]:checked').value;
        state.query = form.querySelector('input[name="query"]').value.trim();
        state.page = 1;
        runSearch(results);
    });

    if (state.query) {
        runSearch(results);
    }
}

async function runSearch(results) {
    if (!state.query) {
        results.innerHTML = "";
        return;
    }

    results.innerHTML = spinner("Searching...");

    try {
        const data = state.kind === "movie"
            ? await searchMovies(state.query, state.page)
            : await searchSeries(state.query, state.page);
        results.innerHTML = renderResults(data);
        wirePagination(results);
    } catch (error) {
        results.innerHTML = errorAlert(error.message);
    }
}

function renderResults(data) {
    if (!data.results || data.results.length === 0) {
        return `<p class="text-secondary">No results.</p>`;
    }

    const cards = data.results.map((item) => {
        const title = state.kind === "movie" ? item.title : item.name;
        const date = state.kind === "movie" ? item.release_date : item.first_air_date;
        const route = state.kind === "movie" ? `#/movie/${item.id}` : `#/series/${item.id}`;

        return `
            <div class="col">
                <a class="card h-100 result-card text-decoration-none" href="${route}">
                    <img class="card-img-top poster" src="${posterImg(item.poster_path)}" alt="${escapeHtml(title)}" loading="lazy">
                    <div class="card-body">
                        <div class="fw-semibold text-truncate">${escapeHtml(title)}</div>
                        <div class="small text-secondary">${yearOf(date) || "—"}</div>
                    </div>
                </a>
            </div>`;
    }).join("");

    return `
        <div class="row row-cols-2 row-cols-md-4 row-cols-xl-6 g-3">${cards}</div>
        ${renderPagination(data)}`;
}

function renderPagination(data) {
    if (!data.total_pages || data.total_pages <= 1) {
        return "";
    }

    return `
        <nav class="mt-4 d-flex justify-content-center align-items-center gap-3">
            <button class="btn btn-outline-light btn-sm" data-page-prev ${data.page <= 1 ? "disabled" : ""}>Previous</button>
            <span class="small text-secondary">Page ${data.page} of ${data.total_pages}</span>
            <button class="btn btn-outline-light btn-sm" data-page-next ${data.page >= data.total_pages ? "disabled" : ""}>Next</button>
        </nav>`;
}

function wirePagination(results) {
    const prev = results.querySelector("[data-page-prev]");
    const next = results.querySelector("[data-page-next]");

    if (prev) {
        prev.addEventListener("click", () => {
            state.page -= 1;
            runSearch(results);
        });
    }

    if (next) {
        next.addEventListener("click", () => {
            state.page += 1;
            runSearch(results);
        });
    }
}
