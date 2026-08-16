# Musix2.0 Design Document

## High Level Design

- **Purpose**: Musix2.0 is the second iteration of a YouTube Music audio downloader. It downloads songs from YouTube and SoundCloud playlists, tags and renames the resulting MP3 files, and persists download state so re-runs skip already-downloaded songs.
- **Core flow**: A CLI command resolves a playlist URL to a music source (YouTube or SoundCloud), fetches per-song metadata, downloads each song through the yt-dlp CLI, post-processes it (ID3 tags + `artist - title.mp3` rename), and records the result in SQLite.
- **Idempotency**: Downloaded songs are stored in the `downloaded_songs` table with a `failed` flag. Runs skip songs that already succeeded and mark download failures so they can be retried later.
- **Playlist management**: Playlists are registered/deleted in the database and listed on demand. A playlist can only map to one source type, inferred from the URL origin.
- **Apple playlist bridging**: Local Apple playlist files (`.txt` in `data/apple_playlists`) can be registered as new public YouTube playlists, with the mapping stored in the database. A separate command syncs the Apple playlist songs into the corresponding YouTube playlist.
- **Anti-detection**: The downloader sleeps a random 2–7 seconds between downloads to avoid tripping the music provider's abuse heuristics.
- **Deployment**: A Dockerfile is provided to keep the environment portable and platform-independent; the database is intended to eventually live on S3 so state survives container recreation.
- **YouTube 403 mitigation**: YouTube's default clients now serve SABR-only streams that return HTTP 403 without a Proof-of-Origin (PO) token. yt-dlp must be kept current and paired with a PO-token provider plugin for downloads to succeed.
- **Style mandate** (per `how_to_write_rust_tips.txt`): functions are written input-to-output (no mutation of inputs), and code favors flat/horizontal structure over nested blocks.

## Technical Specifications

- **Language/Runtime**: Rust 2021 edition.
- **Core dependencies**: `clap` (CLI), `rusqlite` (SQLite), `ureq` (HTTP), `serde_json` (JSON), `regex` (URL/title parsing), `audiotags` (ID3 tags), `getset` (getters), `rand` (sleep jitter), `aws-sdk-s3`/`aws-config` (planned S3 database), `rustube` (reserved for YouTube extraction).
- **SQLite schema** (at `data/database/sqlite.db`):
  - `playlists (playlist_url)`: registered playlist URLs, unique.
  - `downloaded_songs (song_url, playlist_url, failed)`: per-playlist song download state, unique on the pair, `failed` updated on conflict.
  - `apple_youtube_playlist_map (apple_playlist_path, youtube_playlist_url)`: Apple file-to-YouTube playlist mapping, path unique.
  - `youtube_playlist_content (youtube_playlist_url, song_youtube_url)`: YouTube playlist contents, unique on the pair.
- **CLI commands**: `CreatePlaylist`, `DeletePlaylist`, `ListPlaylists`, `Run`, `RegisterApplePlaylists`, `UpdateApplePlaylists`.
- **External integrations**:
  - `yt-dlp`: invoked as a subprocess (`./yt-dlp`, `--audio-format mp3`, `-x`, output `tmp`); the executable is auto-downloaded per OS (Windows/macOS/Linux) with `chmod 755` on Unix. Must be kept current — YouTube now 403s stale versions (SABR/PO-token enforcement).
  - `bgutil-ytdlp-pot-provider` (PO token provider): plugin bundled at `yt-dlp-plugins/` (portable plugin dir next to the binary) with a generation-script provider at `~/bgutil-ytdlp-pot-provider/server` (deno, npm deps installed via `deno install`). Supplies GVS PO tokens to the default YouTube client so media downloads do not 403. The HTTP-server variant (port 4416) is optional and not currently run.
  - YouTube Data API v3: playlist metadata, paginated `playlistItems`, playlist creation, and playlist item insertion, keyed by `resources/api_key.txt`.
  - SoundCloud: page scraping of the `window.__sc_hydration` JSON plus cross-origin script scraping for the `client_id`, with an API-v2 track lookup fallback when `permalink_url` is missing.
- **Playlist URL format**: only canonical `https://www.youtube.com/playlist?list=...` URLs parse in `get_playlist_song_information`; watch-form URLs (`watch?v=...&list=...`) fail the parser.
- **Genre**: per-song genre is set to the YouTube playlist title fetched from the Data API; there is no CLI override for a custom genre.
- **File environment**: `data/` (Apple playlists, database), `downloaded/` (fresh downloads), `archive/` (previously downloaded songs moved before each run), `yt-dlp-plugins/` (PO-token provider plugin). Files are moved `downloaded/ -> archive/` at startup.
- **Post-processing**: ID3 metadata (title, artist, genre) is appended, then the file is renamed to `{artist} - {title}.mp3` with Windows-invalid filename characters (`< > : " / \ | ? *`) replaced by spaces.
- **Settings**: `settings.json` (SoundCloud credentials) exists but is currently unused.

## Running the Program

- **Prerequisites**:
  - Rust toolchain (`cargo`).
  - `ffmpeg`/`ffprobe` — required by yt-dlp for `-x --audio-format mp3` conversion.
  - `deno` — JS runtime for yt-dlp nsig decryption and the PO-token provider script.
  - YouTube Data API key at `resources/api_key.txt` (gitignored).
  - PO-token provider script at `~/bgutil-ytdlp-pot-provider/server` (deps: `cd ~/bgutil-ytdlp-pot-provider/server && deno install --allow-scripts=npm:canvas --frozen`); the plugin itself is committed in the repo at `yt-dlp-plugins/`.
- **Build**: `cargo build` (binary at `target/debug/Musix2`).
- **Commands**:
  - `./target/debug/Musix2 create-playlist <url>` — register a playlist. Use the canonical `https://www.youtube.com/playlist?list=...` form; watch-form URLs fail the parser.
  - `./target/debug/Musix2 delete-playlist <url>` — unregister a playlist.
  - `./target/debug/Musix2 list-playlists` — print registered playlists.
  - `./target/debug/Musix2 run` — download all not-yet-downloaded songs across all registered playlists.
  - `./target/debug/Musix2 register-apple-playlists` / `update-apple-playlists` — Apple playlist bridging.
- **Notes**:
  - On startup `run` creates the file environment, auto-downloads the `yt-dlp` executable if missing, and moves leftover `downloaded/*` into `archive/`.
  - Keep yt-dlp current (`./yt-dlp -U`); stale versions return HTTP 403 from YouTube.

## Technical Architecture

- **`main.rs`**: Entry point; loads environment variables, initializes the file environment, builds the `Database`, and delegates to CLI parsing.
- **`command_line_runtime.rs`**: clap definitions and all command handlers; orchestrates the download pipeline in `handle_run`.
- **`music_sources.rs`**: defines the `MusicSource` trait (`download_song`, `get_playlist_song_information`), the shared `SongInformation`/`DownloadedSong` structs, and the factory functions that map a URL origin (`soundcloud`, `youtube`) to a source instance.
- **`music_sources/soundcloud_service.rs`**: SoundCloud playlist track extraction (script + `__sc_hydration` scraping, per-track JSON parsing) and download delegation.
- **`music_sources/youtube_service.rs`**: YouTube playlist metadata and song listing via the Data API, YouTube playlist creation, song insertion, and download delegation. Uses `title_extractor` to derive artist/title from video titles.
- **`yt_dlp_caller.rs`**: thin wrapper that spawns the `yt-dlp` CLI for a given `SongInformation` and returns a `DownloadedSong`.
- **`database.rs`**: SQLite wrapper modeled as a state machine (`UninitializedDatabase -> InitializedDatabase`) with lazy initialization; all public methods delegate to the initialized state.
- **`environment_extractor.rs`**: environment variable extraction; the S3 database URI field is currently commented out (planned).
- **`environment_initializer.rs`**: ensures `data/`, `database/`, `downloaded/`, `archive/` exist, downloads the `yt-dlp` executable if absent, and moves leftover downloads to the archive.
- **`post_processor.rs`**: orchestration of tag appending and file rename, returning the new file path.
- **`audio_tag_appender.rs`**: writes title/artist/genre ID3 tags to the downloaded file.
- **`title_extractor.rs`**: parses artist and track name out of YouTube video titles using regex separators; falls back to the channel name as artist.
- **`url_enforcer.rs`**: validates that a URL has `https` plus sub-origin/origin/root-origin structure.
- **`settings_parser.rs`**: reads and parses `settings.json` (unused by the current pipeline).
- **Documented limitations / TODOs**:
  - S3-backed database hosting is planned but not implemented (environment extraction commented out in `main.rs`).
  - `handle_update_apple_playlists` treats each Apple playlist line as a YouTube URL directly; real title-to-video search is a placeholder.
  - Migration to `anyhow` error handling is planned.
  - Unit tests exist only in `title_extractor.rs`; parsing tests with real sample data are pending.
  - PO-token provider is a per-machine setup (deno + provider script at `~/bgutil-ytdlp-pot-provider`); the Docker image must bundle the plugin + provider for parity.
