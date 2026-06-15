// Minimal hash router. Each route maps a path pattern to a page module exposing render(view, params).
import { errorAlert } from "./ui.js";
import * as home from "./pages/home.js";
import * as search from "./pages/search.js";
import * as movie from "./pages/movie.js";
import * as series from "./pages/series.js";
import * as indexers from "./pages/indexers.js";
import * as downloads from "./pages/downloads.js";

const ROUTES = [
    { pattern: /^\/?$/, page: home },
    { pattern: /^\/search$/, page: search },
    { pattern: /^\/movie\/(\d+)$/, page: movie },
    { pattern: /^\/series\/(\d+)$/, page: series },
    { pattern: /^\/indexers$/, page: indexers },
    { pattern: /^\/downloads$/, page: downloads },
];

function currentPath() {
    const hash = window.location.hash.replace(/^#/, "");
    return hash || "/";
}

async function handle() {
    const path = currentPath();
    const view = document.getElementById("view");

    for (const route of ROUTES) {
        const match = path.match(route.pattern);
        if (!match) {
            continue;
        }

        view.scrollTop = 0;
        view.innerHTML = "";

        try {
            await route.page.render(view, match.slice(1));
        } catch (error) {
            view.innerHTML = errorAlert(error.message);
        }

        return;
    }

    window.location.hash = "#/";
}

export function startRouter() {
    window.addEventListener("hashchange", handle);
    handle();
}
