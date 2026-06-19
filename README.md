# FMHY Downloader

A docker container for downloading VODs off of certain [freemediaheckyeah](https://fmhy.net/video) sites.

> [!WARNING]
> This software is currently in alpha stages, there may be bugs and breaking changes to the API.

## Features

- VOD Indexing from various [freemediaheckyeah](https://fmhy.net/video) sites (actually their backends).
- (Bulk) VOD Downloading from [freemediaheckyeah](https://fmhy.net/video) sites.
- Easy navigatable WebUI
- VOD Validation // TODO
- Prowlarr indexer API // TODO

## Installation

Pre-built images are published to the GitHub Container Registry at
[`ghcr.io/vod-downloaders/fmhy-downloader`](https://github.com/VOD-Downloaders/FMHY-Downloader/pkgs/container/fmhy-downloader).
Available tags: `latest` (newest release), `nightly` (latest `dev` build), and per-version tags (e.g. `0.1.0-alpha2`).

The recommended way to run is with Docker Compose. Create a `compose.yaml`:

```yaml
services:
  fmhy_downloader:
    image: ghcr.io/vod-downloaders/fmhy-downloader:latest
    container_name: fmhy_downloader
    volumes:
      - ./config:/config
      - ./output:/output
    environment:
      - LOG_LEVEL=info
      - BRIDGE_URL=http://fmhy_bridge:3000/
      - FLARESOLVERR_URL=http://flaresolverr:8191/v1 # Optional
    ports:
      - 8080:8080
    restart: unless-stopped

  # This container contains the actual decryption logic for all
  # the backends and is seperate to keep the reverse-engineered code secret
  fmhy_bridge:
    image: ghcr.io/ggjorven/fmhy-bridge:latest
    container_name: fmhy_bridge
    environment:
      - LOG_LEVEL=info
    restart: unless-stopped

  flaresolverr: # Optional
    image: ghcr.io/flaresolverr/flaresolverr:latest
    container_name: flaresolverr
    environment:
      - LOG_LEVEL=info
    restart: unless-stopped
```

Then start it:

```sh
docker compose up -d
```

> [!NOTE]
> `flaresolverr` is only required for Cloudflare-protected sites. If you don't need it, remove the service
> and the `FLARESOLVERR_URL` environment variable.

## Configuration

The container is configured through environment variables:

| Variable | Default | Description |
|---|---|---|
| `LOG_LEVEL` | `info` | Log verbosity: `debug` / `info` / `warning` / `error` |
| `BRIDGE_URL` | `http://fmhy_bridge:3000/` | FMHY Bridge container url |
| `FLARESOLVERR_URL` | - | FlareSolverr endpoint (may be empty) |
| `WEBUI_PORT` | `8080` | Port the WebUI/API listens on |

For more configuration options check out [CONFIGURATION.md](./doc/CONFIGURATION.md).

## Usage

The WebUI is served on [`http://localhost:8080`](http://localhost:8080) (or the port set with `WEBUI_PORT`). Use the sidebar to navigate. A typical run:

1. **Indexers** - Create an indexer from a specification and pick a server. Specifications are fetched from the [VOD-Downloaders/FMHY-Indexers](https://github.com/VOD-Downloaders/FMHY-Indexers)
   repository. Hit **Update specifications** to pull the latest.
2. **Search** - Search TMDB for a movie or series and open its details page.
3. **Streams** - On the details page, pick an indexer to list available streams, then start a download. Finished files land in the `/output` directory.
4. **Downloads** - Shows the downloads you've started from this browser.

For a full walkthrough see [GETTING-STARTED.md](./doc/GETTING-STARTED.md).

## Contributing

Contributions are highly appreciated, please follow the [CONTRIBUTING GUIDELINES](./CONTRIBUTING.md) to make a quality contribution.

## Third-Party Libraries

This project uses quite a lot of dependencies, these can be found under [THIRD-PARTY](./THIRD-PARTY.md).

## License

This project is licensed under the **GNU Affero General Public License v3.0**. See [LICENSE](LICENSE.txt) for the full license text.
