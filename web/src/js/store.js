// localStorage-backed client state: tracked downloads (no backend status endpoint exists yet)
// and the last-used indexer for stream searches.

const DOWNLOADS_KEY = "fmhy.downloads";
const LAST_INDEXER_KEY = "fmhy.lastIndexer";

export function getDownloads() {
    try {
        return JSON.parse(localStorage.getItem(DOWNLOADS_KEY)) || [];
    } catch {
        return [];
    }
}

export function addDownload(entry) {
    const list = getDownloads();
    list.unshift(entry);
    localStorage.setItem(DOWNLOADS_KEY, JSON.stringify(list));
}

export function clearDownloads() {
    localStorage.removeItem(DOWNLOADS_KEY);
}

export function getLastIndexer() {
    return localStorage.getItem(LAST_INDEXER_KEY) || "";
}

export function setLastIndexer(name) {
    localStorage.setItem(LAST_INDEXER_KEY, name);
}
