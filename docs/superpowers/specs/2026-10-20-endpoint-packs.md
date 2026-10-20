# Endpoint packs: shipping AniBeam with no endpoints

Issue #97. Written 2026-10-20.

## Why

The owner wants AniBeam to earn something: paid signed builds for Linux, macOS
and Windows, with donations beside them. The source stays GPL. A paid app must
not ship pointed at release sites, because that is what turns a neutral player
with a torrent client into a product sold for infringement. Most real users
will still want the anirss side, the owner and Liam first, so adding endpoints
has to take one step and then work exactly as it does today.

The split: **AniBeam ships the machinery, users bring the endpoints.** This is
how Sonarr, Prowlarr and Kodi stay on the right side of the line.

## What ships

| Piece | Today | After |
| --- | --- | --- |
| AniBeam endpoints | whatever anirss's config holds | none on a fresh install |
| anirss bootstrap config | a nyaa endpoint, a release group ranking | no endpoints, no group names |
| Bundled anirss and qbittorrent-nox | yes | yes, unchanged |
| Store page, README, screenshots | | the library, the player, tracking; Downloads as "your own RSS feeds" |
| Test fixtures | real site names | stay; they are parser fixtures, not configuration |

Existing installs change nothing. anirss keeps the config it already has, so
the owner's and Liam's endpoints and subscriptions stay where they are.

## The user's path

1. Fresh install, Downloads page: an empty state says "No endpoints yet" with
   two buttons, **Paste a search URL** and **Import a pack**.
2. **Paste a search URL** takes the address a browser shows, plus an optional
   name. The core reads it by shape (`pack::from_url`): a nyaa-style site
   becomes a `nyaa` endpoint at its root, a feed gets `{query}` in its search
   parameter (`s` when it has none), a torznab API gets `t=search&q={query}`,
   and a URL that already has `{query}` is used as it is. No site is named in
   the UI; the shapes are quiet support, so a person sets up without knowing
   anirss's terms. A token in the address stays in it.
3. **Import a pack** takes a `.json` file, or pasted text. The dialog shows the
   pack's name, its endpoints in order and, if it carries them, its best-fit
   preferences with a checkbox to apply them. Import adds after the existing
   endpoints; a name already in use is replaced in place.
4. Settings, Downloads tab: the endpoint list with Remove on each row and
   **Export pack** at the bottom, which writes the current list to a file.

Export reads anirss's own `config.toml`, so URLs come out whole, tokens
included (owner's call, 2026-10-20): a pack is a backup or a hand-over the
owner chooses to make. On-demand endpoints are left out. The best-fit ranking
travels in the pack, and import shows it but does not apply it yet; that needs
a `[bestfit]` write route in anirss. Liam sets up from scratch by pasting
URLs, which is the path that has to be easy. AniBeam never hosts, lists or
links a pack.

## The pack

```json
{
  "format": "anibeam-endpoints",
  "version": 1,
  "name": "Marcus's endpoints",
  "endpoints": [
    {"name": "main", "kind": "nyaa", "url": "https://site.example/", "category": "1_0", "filter": "0"},
    {"name": "fallback", "kind": "rss", "url": "https://other.example/rss?q={query}"}
  ],
  "best_fit": {"preferred_groups": ["GroupA", "GroupB"], "preferred_resolution": "1080"}
}
```

`kind` is anirss's: `nyaa` for nyaa-style software, `rss` for any search URL
with `{query}`. On-demand endpoints (Jackett pods) stay hand-written in
anirss's config for now; their start and stop commands are shell commands,
and a pack that runs commands is a pack someone can use against you.

## Layers

- **Core** (this skeleton): `downloads::pack` reads, checks, merges and writes
  packs. The contract has `ImportEndpointPack { text }`, `RemoveEndpoint { name }`
  and `ExportEndpointPack`, replying `EndpointPack { pack }`. Import checks the
  pack today and then answers `Unsupported` until anirss can take the write.
- **anirss**: needs `POST /endpoints` and `DELETE /endpoints/<name>` on the
  serve API that rewrite the `[[endpoint]]` tables in `config.toml`, and an
  unmasked read for export. Its bootstrap config drops the nyaa endpoint and the
  group list. Tracked in marcusbandit/anirss#14.
- **Shell**: the empty state on the Downloads page, the import dialog, the
  list in Settings. Not in this skeleton.
- **CLI**: `anibeam-cli call ImportEndpointPack --json '{"text": "..."}'`
  works as soon as the core does.

## Money

- Paid: signed AppImage, dmg and a Windows build, sold through a merchant of
  record (Lemon Squeezy or Paddle) so EU VAT is handled for us. No licence
  check in the code; the GPL makes one pointless and it annoys paying users.
- Free: the source, the AUR package, donations through GitHub Sponsors and
  Ko-fi.
- Before charging: confirm AniList's and MyAnimeList's API terms allow a paid
  client, and credit TVmaze as its licence asks.

## Order of work

1. anirss: endpoint write API and an empty bootstrap.
2. Core: wire the three calls through `AnirssClient`.
3. Shell: empty state, import dialog, Settings list.
4. Owner exports a pack, Liam imports it on a fresh install: the acceptance test.
5. Store page and the first paid build.
