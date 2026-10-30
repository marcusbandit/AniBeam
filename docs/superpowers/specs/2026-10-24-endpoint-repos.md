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
| `index.min.json` | `repo.json`, format `anibeam-endpoint-repo` v1 |
| APK per extension, signed | JSON definitions inside one signed file |
| Signing key in the repo store | ed25519 public key, pinned when the repo is added |
| `code` decides updates | `version` decides updates |
| Extension settings screen for keys and logins | `auth` fields in the definition, asked for at install |
| Kotlin source API | Endpoint kinds in the core |

## The repo

`marcusbandit/anibeam-endpoints`, laid out like this:

```
repo.json           built and signed by CI from endpoints/
endpoints/
  <id>.json         one definition per endpoint, the source CI reads
schema/
  endpoint.schema.json
scripts/check.sh
scripts/build-index.sh
.github/workflows/publish.yml
```

The URL a user pastes is the raw URL of `repo.json`. It is the only file the app
ever fetches.

### repo.json

One file, so a CDN that caches each file on its own can never hand the app a new
index with an old signature:

```json
{
  "format": "anibeam-endpoint-repo",
  "version": 1,
  "payload": "<base64 of the index JSON>",
  "signature": "<base64 of the ed25519 signature over the payload's bytes>"
}
```

The payload is base64 so the signed bytes reach the app exactly as CI wrote them,
with no JSON canonicalisation between jq and serde. Decoded, it is the index:

```json
{
  "name": "Bandit's endpoints",
  "key": "ed25519:9d2f...base64",
  "serial": 1793050000,
  "endpoints": [ { "id": "example-rss", "version": 3, "...": "the full definition" } ]
}
```

- The index carries every definition whole. A repo of a few dozen endpoints is a
  few kilobytes, and installing needs no second fetch.
- `serial` is the publish time in seconds and only goes up. The core refuses an
  index whose serial is lower than the last one it accepted from that repo, so a
  stale mirror or a replayed old file cannot hide an update or bring back an
  endpoint the owner pulled. There is no expiry: an offline phone must keep
  working with the index it has.
- A fetch that fails, or fails its checks, keeps the last good index and says so
  on the repo's row. It never removes anything.

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
  at request time, so a stored URL never holds a secret. Values are
  percent-encoded when filled. Errors, the activity log and anything else that
  prints a URL print the template, never the filled URL. Recognising a URL as
  belonging to an endpoint (`Endpoint::owns_url`) compares against the template
  with its placeholders filled, so an auth placeholder before `{query}` works.
- `auth` is empty for an endpoint that needs nothing. A login with username and
  password is two fields; a flow that needs a browser (OAuth, a cookie behind a
  captcha) is out of scope until one endpoint actually needs it.
- `front` is the fronting host from `core/src/net/front.rs`, moved out of the
  code and into data. The store build then names no site anywhere. A fronted
  connection still goes to the URL host's own address (`FrontResolve`), so a
  front can only change which certificate name is checked there. The install
  screen shows both hosts.
- `rate` can only slow the core down. The limiter takes the lower of the
  definition's rate and the core's own floor for that host (#135), and the
  budget is per host, so two endpoints on one site share it. Every call still
  goes through `net/limiter.rs`.
- `min_core` hides the endpoint from a core too old for its kind or fields.
- `category` and `filter` are for `nyaa` only, as in `releases::Endpoint`.

### The `json` kind

A `json` endpoint is a search API that answers in JSON, giving per result a title,
an info-hash or a link, a size and maybe a page. The kind knows no field names;
the definition maps them in a `json` object, which this kind requires and every
other kind refuses:

```json
{
  "id": "example-json",
  "name": "Example search",
  "version": 1,
  "kind": "json",
  "url": "https://search.example.org/api?q={query}&limit=50",
  "json": {
    "items": "data.hits",
    "title": "label",
    "hash": "torrent.infohash",
    "link": "torrent.url",
    "size": "torrent.bytes",
    "page": "https://example.org/view/{ref.post}",
    "skip": "flags.hidden",
    "error": "status.failed"
  }
}
```

- Every path is object keys joined by dots, with no array indexing. `items` is
  read from the reply and is empty when the reply itself is the results array;
  `error` is read from the reply's top; the rest from each result.
- `items` and `title` are required, and so is one of `hash` and `link`.
- `hash` becomes `magnet:?xt=urn:btih:<hash>&dn=<title>` only when it is 40 hex
  or 32 base32 characters. `link` is a ready `magnet:` or `https://` link, used
  when there is no usable hash. A result with neither is left out.
- `size` is bytes, as a number.
- `page` is an `https://` template with a literal host and `{path}` placeholders
  in its path or query only. It is filled, each value percent-encoded, only when
  every placeholder's value is present and not null, empty, zero or false;
  otherwise the result has no page.
- `skip` drops a result whose value there is set: this is how a definition
  leaves out results its site flags, such as adult ones.
- A set value at `error` makes the reply an error, with the value as the message
  when it is a non-empty string and a generic one otherwise.
- The query's `-word` exclusions are applied after the fetch rather than sent,
  since the kind cannot know which search syntax a site takes.
- A `json` source cannot be added by hand or saved as a phone source: its map
  only comes from a repo.

## Trust

- Adding a repo fetches `index.json` and `index.json.sig` and checks the
  signature against the `key` in the index. The app shows the repo name, the URL
  and the key's fingerprint, and the user accepts it. The key is pinned from then
  on: an index signed by a different key is refused with a plain message, and the
  user has to remove and re-add the repo to accept a new key.
- Installing an endpoint takes its definition from the verified index; there is
  nothing else to fetch or check.
- Trusting a repo means trusting where its endpoints send requests. The owner's
  repo is the only one this design is built for; a third-party repo gets the same
  fingerprint screen and nothing more.
- A definition is data. Nothing in it runs: no commands, no scripts. That is also
  why on-demand endpoints (Jackett pods with start and stop commands) stay out of
  repos, as #97 already decided for packs.

## Updates

- The core refreshes repo indexes at launch and when the user pulls to refresh
  the list. No intervals, the same rule as the library watcher.
- An installed endpoint with a higher `version` in the index is shown as having
  an update. Updating keeps the stored credentials only when the `auth` keys are
  unchanged and the hosts of `url` and `front` are unchanged. A new key, or a
  credential that would now go to a different host, asks the user again and
  shows the new host.
- An endpoint removed from the index stays installed and is marked as no longer
  offered. The user removes it.

## Core

- A store table, `endpoint_repos` (url, name, key, serial, last good index,
  fetched at), and `installed_endpoints` (install id, repo url, endpoint id,
  version, definition JSON, enabled, order). An installed endpoint is identified
  by repo plus endpoint id, so two repos that both offer `example` never collide;
  the install id is a UUID the rest of the core uses. Installed endpoints join the
  phone's ordered endpoint list in `phone::sources`.
- Desktop (built 2026-10-24, changed from the first draft at the owner's ask): the
  desktop searches through anirss, which reads only its own `config.toml`. The
  core writes each enabled installed endpoint into AniBeam's own anirss sidecar
  through anirss 0.6's endpoint API, filled URL included, so a credential does
  land in `config.toml` the way a hand-written one already does; the secret store
  stays the record AniBeam reads. Only names AniBeam wrote are changed or
  removed (`anirss_synced_endpoints` in settings); an endpoint anirss already has
  from its config keeps its place and the catalog says so. A kind anirss cannot
  search (`tosho`) stays out and says so. A remote anirss, a sandbox root and a
  drive are never written to. Searching through `releases::native` on the
  desktop would remove the credential from `config.toml`, and stays the goal.
- Endpoints added by hand (`AddCustomEndpoint`, owner's ask 2026-10-24): a name
  and an address, beside whatever repos there are, for a site no repo carries
  (1337x through Jackett). A secret in the address (`apikey=`, `token=`,
  `passkey=`) moves to the secret store and the address keeps a placeholder.
  Plain http is allowed for these, for a Jackett on the same machine. They are
  stored with an empty repo url and behave like installed endpoints otherwise.
- Credentials go to `secrets.json` under `endpoint/<install id>/<key>`. They never
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
4. Merge. CI rebuilds `repo.json`, signs it with the key stored as a repo
   secret, and commits it.

The repo has to be public for the app to read it: a raw URL of a private repo
needs a GitHub token. It starts private while the format settles. Until then the
core and `anibeam-cli` also accept a local path to `repo.json`, which is how the
fixture tests and step 2 above work, and the owner's builds preload nothing.

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
