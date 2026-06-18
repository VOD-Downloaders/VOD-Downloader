# Endpoints

The docker container exposes an HTTP server with callable API functions, listed below:

| Type | Endpoint | Description | Query | Input | Output |
|---|---|---|---|---|---|
| `GET` | `/health` | Healthcheck endpoint | - | - | `{ health }` |
| `GET` | `/api/indexers` | Retrieve active indexers | - | - | `{ indexers }` |
| `POST` | `/api/indexers/create` | Create an active indexer from a specification | - | `{ indexer }` | - |
| `POST` | `/api/indexers/update` | Update an active indexer (rename + overwrite) | - | `{ old_name, indexer }` | - |
| `POST` | `/api/indexers/delete` | Delete an active indexer | - | `{ name }` | - |
| `POST` | `/api/indexers/refresh` | Reload active indexers from disk | - | - | - |
| `GET` | `/api/indexers/specifications` | Retrieve usable indexer specifications | - | - | `{ indexers }` |
| `POST` | `/api/indexers/specifications/refresh` | Refetch indexer specifications from GitHub | - | - | - |
| `GET` | `/api/info/movie/search` | Search for a movie | `name, page` | - | `{ response }` |
| `GET` | `/api/info/series/search` | Search for a series | `name, page` | - | `{ response }` |
| `GET` | `/api/info/movie/{movie_id}` | Retrieve movie information | - | - | `{ response }` |
| `GET` | `/api/info/movie/{movie_id}/external_ids` | Retrieve external IDs for a movie | - | - | `{ response }` |
| `GET` | `/api/info/series/{series_id}` | Retrieve series information | - | - | `{ response }` |
| `GET` | `/api/info/series/{series_id}/external_ids` | Retrieve external IDs for a series | - | - | `{ response }` |
| `GET` | `/api/info/series/{series_id}/season/{season_number}` | Retrieve season information | - | - | `{ response }` |
| `GET` | `/api/info/series/{series_id}/season/{season_number}/episode/{episode_number}` | Retrieve episode information | - | - | `{ response }` |
| `GET` | `/api/streams/indexer/{indexer_name}/movie/{movie_id}` | List available streams for a movie | - | - | `{ streams }` |
| `GET` | `/api/streams/indexer/{indexer_name}/series/{series_id}/season/{season_number}/episode/{episode_number}` | List available streams for an episode | - | - | `{ streams }` |
| `POST` | `/api/download/indexer/{indexer_name}` | Start downloading a stream | - | `{ stream, output_file }` | `{ id }` |
| `GET` | `/api/download/{id}` | Retrieve download status/info | - | - | `{ start_time, end_time, output_file, download_status }` |

