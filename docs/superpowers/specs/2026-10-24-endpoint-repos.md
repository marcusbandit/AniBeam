# Endpoint repos: endpoints a user adds by URL, the way Mihon gets its sources

Issue #143. Written 2026-10-24.

## Why

The phone app is headed for the Play Store, and a store build must ship with no
endpoints (#122). Mihon solves the same problem for manga: the app ships empty, a
user pastes the URL of an extension repository, and installs sources from it one
at a time. The owner wants the same shape, with the owner hosting one such
repository and adding endpoints to it only after checking each by hand: whether it
needs an API key, a token, a login, or nothing.

One correction to the starting assumption: Mihon is not on Google Play. Its FAQ
(https://mihon.app/docs/faq/general) says "Mihon won't be on the Google Play
Store" because "APK-based extensions conflict with Google Play's content policy".
Play's Device and Network Abuse policy says an app "may not download executable
code (such as dex, JAR, .so files) from a source other than Google Play", with an
exception for "code that runs in a virtual machine or an interpreter".

So AniBeam copies Mihon's repository model and drops its extension model. A Mihon
extension is an APK with Kotlin in it. An AniBeam endpoint in a repo is a JSON
definition the core already knows how to run. The core holds every kind of
endpoint as Rust (`nyaa`, `rss`, `tosho`, later `torznab`), and a new kind ships
in an app update, through Play, like any other change.

## How Mihon does it, for reference

- A repo is an `index.min.json` on a raw GitHub URL. Each entry has `name, pkg,
  apk, lang, code, version, nsfw` and a `sources` list of `name, lang, id,
  baseUrl`.
- The repo declares a signing key. Mihon installs an APK only if the APK is
  signed with that key; anything else is "untrusted" until the user trusts that
  package, version and signature hash.
- An update is a higher `code` (version code) in the index than the installed
  one.
- Mihon ships with no repo since 2024.

## What AniBeam keeps and changes

| Mihon | AniBeam |
| --- | --- |
| Repo URL pasted in Settings | Same |
| `index.min.json` | `index.json`, format `anibeam-endpoint-repo` v1 |
| APK per extension, signed | JSON per endpoint, checked by a SHA-256 in a signed index |
| Signing key in the repo store | ed25519 public key, pinned when the repo is added |
| `code` decides updates | `version` decides updates |
| Extension settings screen for keys and logins | `auth` fields in the definition, asked for at install |
| Kotlin source API | Endpoint kinds in the core |

## The repo

`marcusbandit/anibeam-endpoints`, laid out like this:

```
index.json          built by CI from endpoints/
index.json.sig      ed25519 signature over the exact bytes of index.json
endpoints/
  <id>.json         one definition per endpoint
schema/
  endpoint.schema.json
scripts/build-index.sh
.github/workflows/publish.yml
```

The URL a user pastes is the raw URL of `index.json`. The app derives every other
URL from it, relative to the same directory.

### index.json

```json
{
  "format": "anibeam-endpoint-repo",
  "version": 1,
  "name": "Marcus's endpoints",
  "key": "ed25519:9d2f...base64",
  "endpoints": [
    {
      "id": "example-rss",
      "name": "Example",
      "version": 3,
      "lang": "en",
      "kind": "rss",
      "auth": ["api_key"],
      "nsfw": false,
      "file": "endpoints/example-rss.json",
      "sha256": "4b1c..."
    }
  ]
}
```

The index repeats `kind` and the names of the `auth` fields so the list can show
"needs an API key" before anything is installed.

### An endpoint definition

```json
{
  "id": "example-rss",
  "name": "Example",
  "version": 3,
  "lang": "en",
  "kind": "rss",
  "url": "https://site.example/rss?q={query}&apikey={api_key}",
  "auth": [
    {
      "key": "api_key",
      "label": "API key",
      "secret": true,
      "help": "Make one under Account, API on the site.",
      "help_url": "https://site.example/account/api"
    }
  ],
  "front": null,
  "rate": { "per_minute": 30 },
  "category": "1_0",
  "filter": "0",
  "min_core": "0.9.0"
}
```

- `id` is stable forever; renaming is a new `name`, not a new `id`.
- `url` takes `{query}` plus one placeholder per `auth` key. The core fills them
  at request time, so a stored URL never holds a secret.
- `auth` is empty for an endpoint that needs nothing. A login with username and
  password is two fields; a flow that needs a browser (OAuth, a cookie behind a
  captcha) is out of scope until one endpoint actually needs it.
- `front` is the fronting host from `core/src/net/front.rs`, moved out of the
  code and into data. The store build then names no site anywhere.
- `rate` feeds the limiter. Every call still goes through `net/limiter.rs`.
- `min_core` hides the endpoint from a core too old for its kind or fields.
- `category` and `filter` are for `nyaa` only, as in `releases::Endpoint`.

## Trust

- Adding a repo fetches `index.json` and `index.json.sig` and checks the
  signature against the `key` in the index. The app shows the repo name, the URL
  and the key's fingerprint, and the user accepts it. The key is pinned from then
  on: an index signed by a different key is refused with a plain message, and the
  user has to remove and re-add the repo to accept a new key.
- Installing an endpoint fetches its file and checks the SHA-256 the signed index
  gives for it. A mismatch is refused.
- A definition is data. Nothing in it runs: no commands, no scripts. That is also
  why on-demand endpoints (Jackett pods with start and stop commands) stay out of
  repos, as #97 already decided for packs.

## Updates

- The core refreshes repo indexes at launch and when the user pulls to refresh
  the list. No intervals, the same rule as the library watcher.
- An installed endpoint with a higher `version` in the index is shown as having
  an update. Updating fetches the new file, checks it, and keeps the stored
  credentials when the `auth` keys are unchanged; a new key asks the user again.
- An endpoint removed from the index stays installed and is marked as no longer
  offered. The user removes it.

## Core

- A store table, `endpoint_repos` (url, name, key, last index, fetched at), and
  `installed_endpoints` (id, repo url, version, definition JSON, enabled, order).
  Installed endpoints join the phone's ordered endpoint list in
  `phone::sources`; on the desktop they are written to anirss the way an
  imported pack is (#97).
- Credentials go to `secrets.json` under `endpoint:<repo>:<id>:<key>`. They never
  enter the database, an export or a pack, and the activity log never prints them.
- Contract calls: `AddEndpointRepo { url }` replies with the repo and its
  fingerprint before it is saved; `TrustEndpointRepo { url, key }` saves it;
  `RemoveEndpointRepo`, `ListRepoEndpoints`, `InstallEndpoint { repo, id,
  credentials }`, `UpdateEndpoint`, `RemoveEndpoint`, `SetEndpointCredentials`.
- The activity log gets a line for adding a repo, installing, updating and
  removing an endpoint. Nothing per fetch.

## Endpoint packs (#97)

A pack stays as the offline, hand-over form: a file someone imports. A repo is
the hosted, versioned form. Both produce the same `releases::Endpoint`, and a
pack's endpoint entry is a definition without `version`, `auth` or `rate`.

## The store build

- Ships with no repo and no endpoint, and names no site, also not in
  placeholder text. The fronting table leaves the code.
- Onboarding: what an endpoint is, a field for a repo URL, and the link to
  everythingmoe from #122 for people looking for one.
- The owner's GitHub builds can preload the owner's repo URL behind a build flag,
  the way they preload endpoints today.

## Adding an endpoint to the owner's repo

One at a time, by hand:

1. Check the site: does a search need a key, a token, a login, or nothing; what
   rate it tolerates; whether some networks block it.
2. Write `endpoints/<id>.json` and try it with `anibeam-cli` against a sandbox
   root.
3. Open a PR in `anibeam-endpoints`. CI validates every file against the schema
   and refuses a duplicate `id` or a `version` that did not go up.
4. Merge. CI rebuilds `index.json`, signs it with the key stored as a repo
   secret, and commits both.

The repo has to be public for the app to read it: a raw URL of a private repo
needs a GitHub token. It starts private while the format settles.

## Later, not now

If an endpoint ever needs real logic, such as a site with no feed that has to
be scraped, the route that stays inside Play's policy is code in a sandboxed
interpreter: a WASM module with no host access except an HTTP call through the
limiter. That is a separate decision, made when such an endpoint exists.

## Order of work

1. Owner's repo: format, schema, CI that validates and signs. No endpoints yet.
2. Core: types, index parsing and signature checks, the store tables, the
   contract calls, tests with a fixture repo.
3. Phone: the Sources screen and onboarding.
4. Fronting moves from `net/front.rs` into definitions.
5. Desktop: the same list in Settings, Downloads.
6. First endpoints, one at a time.
