// Reusable, modular sidebar. Used on the single-page shell via <component-sidebar></component-sidebar>.
// Active link is derived from the current hash route, so no per-page wiring is needed.

class Sidebar extends HTMLElement {
    static PAGES = [
        { href: "#/", label: "Home" },
        { href: "#/search", label: "Search" },
        { href: "#/indexers", label: "Indexers" },
        { href: "#/downloads", label: "Downloads" },
    ];

    connectedCallback() {
        this.render();
        this.onHashChange = () => this.render();
        window.addEventListener("hashchange", this.onHashChange);
    }

    disconnectedCallback() {
        window.removeEventListener("hashchange", this.onHashChange);
    }

    render() {
        const current = window.location.hash || "#/";

        const items = Sidebar.PAGES.map((page) => {
            const active = Sidebar.isActive(page.href, current) ? " active" : "";

            return `
                <li class="nav-item">
                    <a class="nav-link${active}" href="${page.href}">${page.label}</a>
                </li>`;
        }).join("");

        this.innerHTML = `
            <nav id="sidebar" class="d-flex flex-column h-100 py-3 px-2">
                <a class="sidebar-brand mb-3 px-1" href="#/">FMHY Downloader</a>
                <ul class="nav nav-pills flex-column gap-1">
                    ${items}
                </ul>
            </nav>
        `;
    }

    static isActive(href, current) {
        if (href === "#/") {
            return current === "#/" || current === "#" || current === "";
        }

        return current === href || current.startsWith(`${href}/`);
    }
}

customElements.define("component-sidebar", Sidebar);
