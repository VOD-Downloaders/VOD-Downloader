# Configuration

The container is configured through environment variables:

| Variable | Default | Description |
|---|---|---|
| `PUID` | `1000` | The user id the container runs as |
| `PGID` | `1000` | The user group id the container runs as |
| `TZ` | `UTC` | Timezone of the container |
| `LOG_LEVEL` | `info` | Log verbosity: `debug` / `info` / `warning` / `error` |
| `BRIDGE_URL` | `http://fmhy_bridge:3000/` | FMHY Bridge container url |
| `FLARESOLVERR_URL` | - | FlareSolverr endpoint (may be empty) |
| `WEBUI_PORT` | `8080` | Port the WebUI/API listens on |
| `CHOWN_CONFIG` | `true` | Changes ownership of the /config directory to `PUID:PGID` |
| `CHOWN_OUTPUT` | `true` | Changes ownership of the /output directory to `PUID:PGID` |
