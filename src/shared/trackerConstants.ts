// The OAuth redirect URI is pinned to a fixed loopback port so the URL you
// register with AniList / MAL once keeps working forever. Renderer and main
// both import this value — keep it in sync.
export const LOOPBACK_HOST = '127.0.0.1';
export const LOOPBACK_PORT = 53682;
export const LOOPBACK_REDIRECT_URI = `http://${LOOPBACK_HOST}:${LOOPBACK_PORT}/callback`;

// Bundled OAuth credentials are read from build-time env vars (ANIBEAM_*),
// inlined by Vite. To bundle creds for distribution: copy .env.example to
// .env.local, fill in the values from the developer panels below, build.
//
//   AniList:  https://anilist.co/settings/developer
//   MAL:      https://myanimelist.net/apiconfig   (App Type: "Web")
//
// .env.local is gitignored. NEVER commit real values to this file or .env.
// When env vars are absent (e.g. fresh clone, CI without secrets) the
// Trackers UI falls back to per-user client-id/secret inputs which are
// stored locally encrypted via Electron safeStorage.
export const DEFAULT_CLIENT_IDS: { anilist: string; mal: string } = {
  anilist: import.meta.env.ANIBEAM_ANILIST_CLIENT_ID ?? '',
  mal: import.meta.env.ANIBEAM_MAL_CLIENT_ID ?? '',
};

export const DEFAULT_CLIENT_SECRETS: { anilist: string; mal: string } = {
  anilist: '',
  mal: import.meta.env.ANIBEAM_MAL_CLIENT_SECRET ?? '',
};
