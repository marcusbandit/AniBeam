/// <reference types="vite/client" />

// Build-time env vars exposed via Vite's import.meta.env. Set in .env.local
// (gitignored) or in CI for release builds. All optional; UI falls back to
// per-user input when missing.
interface ImportMetaEnv {
  readonly ANIBEAM_ANILIST_CLIENT_ID?: string;
  readonly ANIBEAM_MAL_CLIENT_ID?: string;
  readonly ANIBEAM_MAL_CLIENT_SECRET?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
