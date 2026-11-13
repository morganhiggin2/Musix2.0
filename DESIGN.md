# Musix2.0 Design Document

## High Level Design

- **Purpose**: Musix2.0 is the second iteration of a YouTube Music audio downloader. It downloads songs from YouTube and SoundCloud playlists, tags and renames the resulting MP3 files, and persists download state so re-runs skip already-downloaded songs.
- **Core flow**: A CLI command resolves a playlist URL to a music source (YouTube or SoundCloud), fetches per-song metadata, downloads each song through the yt-dlp CLI, post-processes it (ID3 tags + `artist - title.mp3` rename), and records the result in SQLite.
- **Idempotency**: Downloaded songs are stored in the `downloaded_songs` table with a `failed` flag. Runs skip songs that already succeeded and mark download failures so they can be retried later.
- **Playlist management**: Playlists are registered/deleted in the database and listed on demand. A playlist can only map to one source type, inferred from the URL origin.
- **Local playlists**: A playlist id starting with `/` (e.g. `/apple_favorites`) resolves to a local file `local_playlists/<id_without_backslash>.txt` instead of a remote source. The file is a comma-separated list with a header row (`url,artist,title,genre`), one song per line, carrying full metadata — no remote metadata lookup is needed. `run` downloads those songs via yt-dlp exactly like a YouTube playlist. Local playlists are registered in bulk by `refresh-local-playlists`, which scans `local_playlists/*.txt` and registers (or removes) the corresponding `/`-prefixed playlists.
- **Anti-detection**: The downloader sleeps a random 2–7 seconds between downloads to avoid tripping the music provider's abuse heuristics.
- **Deployment**: A Dockerfile is provided to keep the environment portable and platform-independent; the database is intended to eventually live on S3 so state survives container recreation.
- **YouTube 403 mitigation**: YouTube's default clients now serve SABR-only streams that return HTTP 403 without a Proof-of-Origin (PO) token. yt-dlp must be kept current and paired with a PO-token provider plugin for downloads to succeed.
- **Style mandate** (per `how_to_write_rust_tips.txt`): functions are written input-to-output (no mutation of inputs), and code favors flat/horizontal structure over nested blocks.

## Technical Specifications

- **Language/Runtime**: Rust 2021 edition.
- **Core dependencies**: `clap` (CLI), `rusqlite` (SQLite), `ureq` (HTTP), `serde_json` (JSON), `regex` (URL/title parsing), `audiotags` (ID3 tags), `getset` (getters), `rand` (sleep jitter), `aws-sdk-s3`/`aws-config` (planned S3 database), `rustube` (reserved for YouTube extraction).
- **SQLite schema** (at `data/database/sqlite.db`):
  - `playlists (playlist_url)`: registered playlist identifiers, unique. Values are either remote URLs or `/`-prefixed local playlist ids.
  - `downloaded_songs (song_url, playlist_url, failed)`: per-playlist song download state, unique on the pair, `failed` updated on conflict.
  - `apple_youtube_playlist_map (apple_playlist_path, youtube_playlist_url)` / `youtube_playlist_content (youtube_playlist_url, song_youtube_url)`: legacy tables from the removed Apple playlist bridging feature; no longer written.
- **CLI commands**: `CreatePlaylist`, `DeletePlaylist`, `ListPlaylists`, `Run` (optional `--playlist <id-or-url>` to target a single playlist), `RefreshLocalPlaylists`.
- **External integrations**:
  - `yt-dlp`: invoked as a subprocess (`./yt-dlp`, `--audio-format mp3`, `-x`, output `tmp`); the executable is auto-downloaded per OS (Windows/macOS/Linux) with `chmod 755` on Unix. Must be kept current — YouTube now 403s stale versions (SABR/PO-token enforcement).
  - `bgutil-ytdlp-pot-provider` (PO token provider): plugin bundled at `yt-dlp-plugins/` (portable plugin dir next to the binary) with the provider at `~/bgutil-ytdlp-pot-provider/server`. Supplies GVS PO tokens to the default YouTube client so media downloads do not 403. The plugin exposes three provider variants — `bgutil:http` (always-on HTTP server, port 4416), `bgutil:script-node`, and `bgutil:script-deno` (auto-spawned generation script). When both methods are available (provider repo present + `node`/`deno` on PATH) the HTTP variant is **prioritized**, so downloads 403 unless the server is actually running (see "PO-token provider — download & launch").
  - YouTube Data API v3: playlist metadata, paginated `playlistItems`, playlist creation, and playlist item insertion, keyed by `resources/api_key.txt`.
  - SoundCloud: page scraping of the `window.__sc_hydration` JSON plus cross-origin script scraping for the `client_id`, with an API-v2 track lookup fallback when `permalink_url` is missing.
- **Playlist URL format**: only canonical `https://www.youtube.com/playlist?list=...` URLs parse in `get_playlist_song_information`; watch-form URLs (`watch?v=...&list=...`) fail the parser. A playlist id beginning with `/` instead denotes a local playlist file (see High Level Design); local playlist file lines must have at least 4 comma-separated columns (`url,artist,title,genre`), where fields containing commas must be wrapped in double quotes (`"Steve Aoki, Chris Lake"`) and literal quotes are escaped by doubling (`""`). Any extra columns are folded back into the title to tolerate unquoted commas in titles.
- **Genre**: per-song genre is set to the YouTube playlist title fetched from the Data API for YouTube playlists; for local playlists the genre comes from each file line. There is no CLI override for a custom genre.
- **File environment**: `data/` (database; `apple_playlists/` is legacy/unused), `downloaded/` (fresh downloads), `archive/` (previously downloaded songs moved before each run), `local_playlists/` (local playlist `.txt` files), `yt-dlp-plugins/` (PO-token provider plugin). Files are moved `downloaded/ -> archive/` at startup.
- **Post-processing**: ID3 metadata (title, artist, genre) is appended, then the file is renamed to `{artist} - {title}.mp3` with Windows-invalid filename characters (`< > : " / \ | ? *`) replaced by spaces.
- **Settings**: `settings.json` (SoundCloud credentials) exists but is currently unused.

## Running the Program

- **Prerequisites**:
  - Rust toolchain (`cargo`).
  - `ffmpeg`/`ffprobe` — required by yt-dlp for `-x --audio-format mp3` conversion.
  - `deno` — JS runtime for yt-dlp nsig decryption and the PO-token provider script.
  - YouTube Data API key at `resources/api_key.txt` (gitignored).
  - PO-token provider repo at `~/bgutil-ytdlp-pot-provider` (clone + install deps, then launch the server — full steps below); the plugin itself is committed in the repo at `yt-dlp-plugins/`.
- **Build**: `cargo build` (binary at `target/debug/Musix2`).
- **Commands**:
  - `./target/debug/Musix2 create-playlist <url-or-id>` — register a playlist. For remote playlists use the canonical `https://www.youtube.com/playlist?list=...` form; watch-form URLs fail the parser. For local playlists use `/`-prefixed ids (file must exist).
  - `./target/debug/Musix2 delete-playlist <url-or-id>` — unregister a playlist.
  - `./target/debug/Musix2 list-playlists` — print registered playlists.
  - `./target/debug/Musix2 run [--playlist <url-or-id>]` — download not-yet-downloaded songs across all registered playlists, or only the given one.
  - `./target/debug/Musix2 refresh-local-playlists` — register `/`-prefixed playlists from `local_playlists/*.txt` and remove any whose files no longer exist.
- **Notes**:
  - On startup `run` creates the file environment, auto-downloads the `yt-dlp` executable if missing, and moves leftover `downloaded/*` into `archive/`.
  - `./run_youtube_local.sh [playlist-id]` — automates the local playlist flow: starts the PO-token provider, updates `yt-dlp`, refreshes local playlists, and runs the download. The provider process it started is stopped on exit.
  - Keep yt-dlp current (`./yt-dlp -U`); stale versions return HTTP 403 from YouTube.

### PO-token provider (bgutil) — download & launch

- **Download**: the plugin is already committed at `yt-dlp-plugins/bgutil-ytdlp-pot-provider.zip` (version 1.3.1). The provider repo must be present at `~/bgutil-ytdlp-pot-provider` and pinned to the same version as the plugin:
  ```shell
  git clone --single-branch --branch 1.3.1 https://github.com/Brainicism/bgutil-ytdlp-pot-provider.git ~/bgutil-ytdlp-pot-provider
  cd ~/bgutil-ytdlp-pot-provider/server
  # Node runtime deps:
  npm ci && npx tsc
  # or Deno runtime deps:
  deno install --allow-scripts=npm:canvas --frozen
  ```
- **Launch**: because the plugin prioritizes the HTTP variant when the repo is present, the always-on server (default `http://127.0.0.1:4416`) must be running before any `run`:
  - Docker (recommended): `docker run --name bgutil-provider -d --init brainicism/bgutil-ytdlp-pot-provider` (or `:1.3.1-deno` for the Deno flavor; pass `-p 1234:4416` to change the published port, then point the plugin at it with `--extractor-args "youtubepot-bgutilhttp:base_url=http://127.0.0.1:1234"`).
  - Native Node: `cd ~/bgutil-ytdlp-pot-provider/server && node build/main.js`.
  - Native Deno: `cd ~/bgutil-ytdlp-pot-provider/server/node_modules && deno run --allow-env --allow-net --allow-ffi=. --allow-read=. ../src/main.ts`.
  - Custom port: append `--port 8080` to any native launch.
- **Verify**: `./yt-dlp -v <youtube-url>` should list the providers in debug output, e.g. `PO Token Providers: bgutil:http-1.3.1 (external), bgutil:script-node-1.3.1 (external), bgutil:script-deno-1.3.1 (external, unavailable)`. A download that fails with `HTTP Error 403: Forbidden` plus `[pot:bgutil:http] Error reaching GET http://127.0.0.1:4416/ping` means the server is not running.
- **Alternative (no server)**: the `bgutil:script-node` / `bgutil:script-deno` variants auto-spawn the generation script per download (no manual launch) and are used only when the HTTP server is unreachable.

## Technical Architecture

- **`main.rs`**: Entry point; loads environment variables, initializes the file environment, builds the `Database`, and delegates to CLI parsing.
- **`command_line_runtime.rs`**: clap definitions and all command handlers; orchestrates the download pipeline in `handle_run` (optionally scoped to a single playlist via `--playlist`) and the local playlist registration/cleanup in `handle_refresh_local_playlists`.
- **`music_sources.rs`**: defines the `MusicSource` trait (`download_song`, `get_playlist_song_information`), the shared `SongInformation`/`DownloadedSong` structs, the factory functions that map a URL origin (`soundcloud`, `youtube`) to a source instance, and `get_music_source_from_playlist_identifier`, which dispatches local `/`-prefixed playlists to the local playlist service.
- **`music_sources/local_playlist_service.rs`**: local playlist source. `get_playlist_song_information` parses `local_playlists/<id>.txt` (header row skipped, blank lines skipped, comma-separated `url,artist,title,genre` columns with quoted-field support, extra columns folded into the title); download delegates to yt-dlp. `validate_local_playlist` is used by `create-playlist` to require the file to exist.
- **`music_sources/soundcloud_service.rs`**: SoundCloud playlist track extraction (script + `__sc_hydration` scraping, per-track JSON parsing) and download delegation.
- **`music_sources/youtube_service.rs`**: YouTube playlist metadata and song listing via the Data API, YouTube playlist creation, song insertion, and download delegation. Uses `title_extractor` to derive artist/title from video titles.
- **`yt_dlp_caller.rs`**: thin wrapper that spawns the `yt-dlp` CLI for a given `SongInformation` and returns a `DownloadedSong`.
- **`database.rs`**: SQLite wrapper modeled as a state machine (`UninitializedDatabase -> InitializedDatabase`) with lazy initialization; all public methods delegate to the initialized state.
- **`environment_extractor.rs`**: environment variable extraction; the S3 database URI field is currently commented out (planned).
- **`environment_initializer.rs`**: ensures `data/`, `database/`, `downloaded/`, `archive/`, `local_playlists/` exist, downloads the `yt-dlp` executable if absent, and moves leftover downloads to the archive.
- **`post_processor.rs`**: orchestration of tag appending and file rename, returning the new file path.
- **`audio_tag_appender.rs`**: writes title/artist/genre ID3 tags to the downloaded file.
- **`title_extractor.rs`**: parses artist and track name out of YouTube video titles using regex separators; falls back to the channel name as artist.
- **`url_enforcer.rs`**: validates that a URL has `https` plus sub-origin/origin/root-origin structure.
- **`settings_parser.rs`**: reads and parses `settings.json` (unused by the current pipeline).
- **Documented limitations / TODOs**:
  - S3-backed database hosting is planned but not implemented (environment extraction commented out in `main.rs`).
  - Migration to `anyhow` error handling is planned.
  - Unit tests exist only in `title_extractor.rs`; parsing tests with real sample data are pending.
  - PO-token provider is a per-machine setup (repo at `~/bgutil-ytdlp-pot-provider` + running HTTP server); the Docker image must bundle the plugin + provider and launch the server for parity.
