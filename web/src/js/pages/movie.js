import { getMovie, getIndexers, getMovieStreams } from "../api.js";
import {
    backdropStyle, posterImg, yearOf, escapeHtml, spinner, errorAlert,
    indexerActionsHtml, attachStreamSearch,
} from "../ui.js";

export async function render(view, params) {
    const id = params[0];
    view.innerHTML = spinner("Loading movie...");

    let movie;
    let indexers;
    try {
        [movie, indexers] = await Promise.all([getMovie(id), getIndexers()]);
    } catch (error) {
        view.innerHTML = errorAlert(error.message);
        return;
    }

    const year = yearOf(movie.release_date);
    const titleHint = `${movie.title}${year ? ` ${year}` : ""}`;
    const meta = [
        movie.genres.map((genre) => escapeHtml(genre.name)).join(" · "),
        movie.runtime ? `${movie.runtime} min` : "",
    ].filter(Boolean).join(" · ");

    view.innerHTML = `
        <div class="detail-hero" style="${backdropStyle(movie.backdrop_path)}">
            <div class="detail-hero-overlay p-4">
                <div class="d-flex gap-4 flex-wrap">
                    <img class="detail-poster" src="${posterImg(movie.poster_path)}" alt="${escapeHtml(movie.title)}">
                    <div class="flex-grow-1">
                        <h1 class="h2">${escapeHtml(movie.title)}
                            <span class="text-secondary fs-5">${year}</span></h1>
                        <div class="mb-2 text-secondary">${meta}</div>
                        ${movie.tagline ? `<p class="fst-italic text-secondary">${escapeHtml(movie.tagline)}</p>` : ""}
                        <p class="detail-overview">${escapeHtml(movie.overview || "")}</p>
                        <div data-actions>${indexerActionsHtml(indexers)}</div>
                    </div>
                </div>
            </div>
        </div>`;

    attachStreamSearch(view.querySelector("[data-actions]"), {
        indexers,
        titleHint,
        fetcher: (indexerName) => getMovieStreams(indexerName, id),
    });
}
