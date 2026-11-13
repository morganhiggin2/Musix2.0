#!/usr/bin/env bash

# Automate the local playlist download flow: start the bgutil PO token
# provider, refresh local playlists, and run Musix2. Only the provider
# started by this script is stopped when the script exits.

set -euo pipefail

PLAYLIST_ID="${1:-/apple_favorites}"
PROVIDER_DIRECTORY="${HOME}/bgutil-ytdlp-pot-provider/server"
PROVIDER_PORT=4416
PROVIDER_LOG="${TMPDIR:-/tmp}/musix2-bgutil-provider.log"
SCRIPT_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

PROVIDER_PID=""
PROVIDER_STARTED="false"

cleanup() {
    if [[ "${PROVIDER_STARTED}" == "true" && -n "${PROVIDER_PID}" ]]; then
        echo "Stopping bgutil PO token provider (pid ${PROVIDER_PID})..."
        kill "${PROVIDER_PID}" 2>/dev/null || true
        wait "${PROVIDER_PID}" 2>/dev/null || true
    fi
}

# always stop the provider we started, including on ctrl-c
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

echo "Using playlist: ${PLAYLIST_ID}"

# reuse an already running provider instead of starting a second one
if curl -fsS "http://127.0.0.1:${PROVIDER_PORT}/ping" >/dev/null 2>&1; then
    echo "PO token provider already running on port ${PROVIDER_PORT}."
else
    if [[ ! -f "${PROVIDER_DIRECTORY}/build/main.js" ]]; then
        echo "PO token provider not found at ${PROVIDER_DIRECTORY}/build/main.js" >&2
        echo "See DESIGN.md for setup instructions." >&2
        exit 1
    fi

    echo "Starting bgutil PO token provider..."
    (
        cd "${PROVIDER_DIRECTORY}"
        exec node build/main.js
    ) >"${PROVIDER_LOG}" 2>&1 &
    PROVIDER_PID=$!
    PROVIDER_STARTED="true"

    # wait for the provider to answer ping, up to 15 seconds
    provider_ready="false"
    for _ in $(seq 1 30); do
        if curl -fsS "http://127.0.0.1:${PROVIDER_PORT}/ping" >/dev/null 2>&1; then
            provider_ready="true"
            break
        fi
        sleep 0.5
    done

    if [[ "${provider_ready}" != "true" ]]; then
        echo "PO token provider did not become ready; see ${PROVIDER_LOG}" >&2
        exit 1
    fi

    echo "PO token provider ready (pid ${PROVIDER_PID})."
fi

cd "${SCRIPT_DIRECTORY}"

# keep yt-dlp current; stale versions return HTTP 403 from youtube
if [[ -x ./yt-dlp ]]; then
    ./yt-dlp -U || echo "Could not update yt-dlp; continuing with the installed version."
fi

cargo build
./target/debug/Musix2 refresh-local-playlists
./target/debug/Musix2 run --playlist "${PLAYLIST_ID}"

echo "Done. Downloads are in ${SCRIPT_DIRECTORY}/downloaded."
