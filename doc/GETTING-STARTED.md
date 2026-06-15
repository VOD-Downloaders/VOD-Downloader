# Getting Started

This guide walks you through downloading your first VOD with FMHY Downloader, from
starting the container to a finished file in your `./output` directory.

> [!NOTE]
> FMHY Downloader does **not** scrape sites itself. Stream discovery is delegated to the
> companion [`fmhy_bridge`](https://github.com/ggjorven/fmhy-bridge) service, and title
> metadata comes from TMDB. Make sure both this app and `fmhy_bridge` are running (see the
> [Installation](../README.md#installation) section of the README).

## 1. Start the stack

Follow the [Installation](../README.md#installation) instructions to create a `compose.yaml` and start the services:

```sh
docker compose up -d
```

Once it is up, open the WebUI at [`http://localhost:8080`](http://localhost:8080) (or the port you set with `WEBUI_PORT`).

## 2. Add an indexer

Before you can find streams you need at least one **indexer**. An indexer is your
configured instance of a site, built from an *indexer specification*.

Specifications are fetched automatically on first run from the [VOD-Downloaders/FMHY-Indexers](https://github.com/VOD-Downloaders/FMHY-Indexers)
repository. The specifications are version-matched to the app and saved under `/config/indexers/specifications/`.

To create an indexer:

1. Open the **Indexers** page from the sidebar.
2. Under **Create indexer**, pick a specification from the **Based on specification**
   dropdown. This pre-fills the name, the available **Servers**, the Cloudflare flag, and
   the download tuning.
3. Pick a **Server** from the list.
4. Optionally adjust:
   - **Uses Cloudflare** - enable for Cloudflare-protected sites. Requires a
     `flaresolverr` service and the `FLARESOLVERR_URL` environment variable
     (see [CONFIGURATION.md](./CONFIGURATION.md)).
   - **Segment timeout / Segment attempts** - per-segment download tuning.
   - **Remove front/back bytes** - trim bytes off each downloaded segment.
   - **Segment headers** - extra HTTP headers sent with each segment request.
5. Click **Save indexer**. It now appears under **Active indexers**, where you can later
   **Edit** or **Delete** it.

> [!TIP]
> If a site is missing or out of date, click **Refresh specifications** to re-fetch the
> latest specs from the [FMHY-Indexers](https://github.com/VOD-Downloaders/FMHY-Indexers)
> repository. Want to contribute a new site? Open a PR there.

Active indexers are stored as JSON under `/config/indexers/` (filename = lowercased name),
so they survive container restarts.

## 3. Search for a title

1. Open the **Search** page from the sidebar.
2. Choose **Movies** or **Series**.
3. Type a title and click **Search**. Results come from **TMDB**.
4. Click a result to open its detail page. Use the **Previous** / **Next** buttons to page through results.

## 4. Find streams

On the detail page:

1. (Series) Pick the season and episode you want.
2. Choose one of your **indexers** or select **all**.
3. FMHY Downloader queries `fmhy_bridge` through that indexer and lists the available
   **streams**, with their quality where known.

If no streams come back, the title may not be available on that site - try another
indexer.

## 5. Download

1. Select a stream from the list.
2. Review the suggested **output file** name and adjust it if you like.
3. Start the download.

The app spawns a background task that downloads its segments to `/output/<output_file>` - which maps to your `./output` directory on the host.

## 6. Track downloads

Open the **Downloads** page to see the downloads you have started from this browser.

> [!NOTE]
> Live progress and status are not available yet 
> The list only reflects downloads started from the current
> browser. Check the `./output` directory for the finished file.

## Next steps

- [CONFIGURATION.md](./CONFIGURATION.md) - all environment variables.
- [ENDPOINTS.md](./ENDPOINTS.md) - the REST API, if you want to script downloads.
- [FMHY-Indexers](https://github.com/VOD-Downloaders/FMHY-Indexers) - the indexer specification repository.
