import { getSeries, getSeason, getIndexers, getEpisodeStreams } from "../api.js";
import {
    backdropStyle, posterImg, stillImg, yearOf, pad, escapeHtml, spinner, errorAlert,
    indexerActionsHtml, attachStreamSearch, openSeasonStreamSearch,
} from "../ui.js";

export async function render(view, params) {
    const id = params[0];
    view.innerHTML = spinner("Loading series...");

    let series;
    let indexers;
    try {
        [series, indexers] = await Promise.all([getSeries(id), getIndexers()]);
    } catch (error) {
        view.innerHTML = errorAlert(error.message);
        return;
    }

    const seasons = series.seasons || [];
    const meta = [
        series.genres.map((genre) => escapeHtml(genre.name)).join(" · "),
        `${series.number_of_seasons} season(s)`,
    ].filter(Boolean).join(" · ");

    view.innerHTML = `
        <div class="detail-hero" style="${backdropStyle(series.backdrop_path)}">
            <div class="detail-hero-overlay p-4">
                <div class="d-flex gap-4 flex-wrap">
                    <img class="detail-poster" src="${posterImg(series.poster_path)}" alt="${escapeHtml(series.name)}">
                    <div class="flex-grow-1">
                        <h1 class="h2">${escapeHtml(series.name)}
                            <span class="text-secondary fs-5">${yearOf(series.first_air_date)}</span></h1>
                        <div class="mb-2 text-secondary">${meta}</div>
                        <p class="detail-overview">${escapeHtml(series.overview || "")}</p>
                    </div>
                </div>
            </div>
        </div>
        <div class="container-fluid p-4">
            <h2 class="h4 mb-3">Seasons</h2>
            <div class="accordion" id="seasons-accordion">
                ${seasons.map((season) => seasonHeader(season)).join("")}
            </div>
        </div>`;

    view.querySelectorAll(".accordion-collapse").forEach((collapse) => {
        collapse.addEventListener("show.bs.collapse", () => {
            const body = collapse.querySelector(".accordion-body");
            if (body.dataset.loaded) {
                return;
            }
            body.dataset.loaded = "1";
            loadSeason(body, id, collapse.dataset.season, series.name, indexers);
        });
    });
}

function seasonHeader(season) {
    const collapseId = `season-${season.season_number}`;
    return `
        <div class="accordion-item">
            <h2 class="accordion-header">
                <button class="accordion-button collapsed" type="button" data-bs-toggle="collapse" data-bs-target="#${collapseId}">
                    ${escapeHtml(season.name)}
                    <span class="text-secondary ms-2 small">${season.episode_count} episodes</span>
                </button>
            </h2>
            <div id="${collapseId}" class="accordion-collapse collapse" data-season="${season.season_number}" data-bs-parent="#seasons-accordion">
                <div class="accordion-body"><div class="text-secondary">Loading...</div></div>
            </div>
        </div>`;
}

async function loadSeason(body, seriesId, seasonNumber, seriesName, indexers) {
    body.innerHTML = spinner("Loading episodes...");

    let season;
    try {
        season = await getSeason(seriesId, seasonNumber);
    } catch (error) {
        body.innerHTML = errorAlert(error.message);
        body.dataset.loaded = "";
        return;
    }

    const episodes = season.episodes || [];
    if (episodes.length === 0) {
        body.innerHTML = `<p class="text-secondary mb-0">No episodes.</p>`;
        return;
    }

    body.innerHTML = seasonToolbar(indexers) + episodes.map((episode) => episodeRow(episode, indexers)).join("");

    wireSeasonToolbar(body, { indexers, seriesName, seriesId, seasonNumber, episodes });

    body.querySelectorAll("[data-episode]").forEach((element) => {
        const episodeNumber = element.dataset.episode;
        const episode = episodes.find((item) => String(item.episode_number) === episodeNumber);
        const titleHint = `${seriesName} S${pad(seasonNumber)}E${pad(episodeNumber)} ${episode.name}`;
        attachStreamSearch(element, {
            indexers,
            titleHint,
            fetcher: (indexerName) => getEpisodeStreams(indexerName, seriesId, seasonNumber, episodeNumber),
        });
    });
}

function seasonToolbar(indexers) {
    if (!indexers || indexers.length === 0) {
        return "";
    }

    return `
        <div class="d-flex justify-content-end mb-3" data-season-toolbar>
            <button type="button" class="btn btn-primary btn-sm" data-season-search>Search</button>
        </div>`;
}

function wireSeasonToolbar(body, { indexers, seriesName, seriesId, seasonNumber, episodes }) {
    const button = body.querySelector("[data-season-search]");
    if (!button) {
        return;
    }

    button.addEventListener("click", () => {
        openSeasonStreamSearch({ indexers, seriesName, seriesId, seasonNumber, episodes });
    });
}

function episodeRow(episode, indexers) {
    return `
        <div class="episode-row d-flex gap-3 py-3 border-bottom" data-episode="${episode.episode_number}">
            <img class="episode-still" src="${stillImg(episode.still_path)}" alt="" loading="lazy">
            <div class="flex-grow-1">
                <div class="fw-semibold">${episode.episode_number}. ${escapeHtml(episode.name)}
                    <span class="small text-secondary ms-2">${escapeHtml(episode.air_date || "")}</span></div>
                <p class="small text-secondary mb-2">${escapeHtml(episode.overview || "")}</p>
                <div data-actions>${indexerActionsHtml(indexers)}</div>
            </div>
        </div>`;
}
