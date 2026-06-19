// Thin API client. One function per backend endpoint. Throws an Error (with .status) on failure,
// using the backend's { error } message when present.

async function request(path, options) {
    const res = await fetch(path, options);

    if (!res.ok) {
        let message = `Request failed (${res.status}).`;
        try {
            const body = await res.json();
            if (body && body.error) {
                message = body.error;
            }
        } catch {
            // Non-JSON or empty error body — keep the default message.
        }

        const error = new Error(message);
        error.status = res.status;
        throw error;
    }

    if (res.status === 204) {
        return null;
    }

    const text = await res.text();
    return text ? JSON.parse(text) : null;
}

function jsonPost(body) {
    return {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
    };
}

// Indexers
export async function getIndexers() {
    const data = await request("/api/indexers");
    return data.indexers;
}

export async function getSpecifications() {
    const data = await request("/api/indexers/specifications");
    return data.indexers;
}

// Re-fetch specifications from GitHub, then reload them from disk.
export function refetchSpecifications() {
    return request("/api/indexers/specifications/refetch", { method: "POST" });
}

// Reload specifications from disk (no GitHub fetch).
export function reloadSpecifications() {
    return request("/api/indexers/specifications/refresh", { method: "POST" });
}

export function reloadIndexers() {
    return request("/api/indexers/refresh", { method: "POST" });
}

export function createIndexer(indexer) {
    return request("/api/indexers/create", jsonPost({ indexer }));
}

export function updateIndexer(oldName, indexer) {
    return request("/api/indexers/update", jsonPost({ old_name: oldName, indexer }));
}

export function deleteIndexer(name) {
    return request("/api/indexers/delete", jsonPost({ name }));
}

// TMDB metadata
export function searchMovies(name, page = 1) {
    return request(`/api/info/movie/search?name=${encodeURIComponent(name)}&page=${page}`);
}

export function searchSeries(name, page = 1) {
    return request(`/api/info/series/search?name=${encodeURIComponent(name)}&page=${page}`);
}

export function getMovie(id) {
    return request(`/api/info/movie/${id}`);
}

export function getSeries(id) {
    return request(`/api/info/series/${id}`);
}

export function getSeason(id, seasonNumber) {
    return request(`/api/info/series/${id}/season/${seasonNumber}`);
}

export function getEpisode(id, seasonNumber, episodeNumber) {
    return request(`/api/info/series/${id}/season/${seasonNumber}/episode/${episodeNumber}`);
}

// Streams
export function getMovieStreams(indexer, id) {
    return request(`/api/streams/indexer/${encodeURIComponent(indexer)}/movie/${id}`);
}

export function getEpisodeStreams(indexer, id, seasonNumber, episodeNumber) {
    return request(`/api/streams/indexer/${encodeURIComponent(indexer)}/series/${id}/season/${seasonNumber}/episode/${episodeNumber}`);
}

// Download
export function startDownload(indexer, payload) {
    return request(`/api/download/indexer/${encodeURIComponent(indexer)}`, jsonPost(payload));
}

export function getDownloadInfo(id) {
    return request(`/api/download/${encodeURIComponent(id)}`);
}
