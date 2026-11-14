// Drives a running relay over HTTP and WebSocket and checks it behaves as the
// protocol says. Node 22 or later (built-in fetch and WebSocket), no packages.
//
//   node smoke.mjs <base-url> [phase] [state-file]
//
// Phases, run in this order by smoke.sh:
//   rooms     create a room, two members, play, seek, ping, errors, bye
//   reload    after the relay restarted: the room came back from storage and
//             the host's rejoin key still gives the same member
//   timers    the host drops and, after the 120 s grace, the other member
//             becomes host (the Durable Object alarm); takes about two minutes
//   limits    the 11th room creation in a minute gets 429, a socket is held to
//             the rate before hello and closed after 10 s without one, and the
//             61st join page in a minute gets 429
//   close     a room nobody joins closes after 10 minutes (the alarm)
//   gone      after a restart that room is still gone: its storage was deleted

import { readFileSync, writeFileSync } from "node:fs";
import { request } from "node:http";

const base = (process.argv[2] ?? "http://127.0.0.1:8797").replace(/\/$/, "");
const phase = process.argv[3] ?? "rooms";
const stateFile = process.argv[4] ?? "smoke-state.json";
const wsBase = base.replace(/^http/, "ws");

const episode = {
  anilist_id: 457,
  episode_key: "s1e3",
  number: 3,
  series_title: "Mushishi",
  episode_title: "Tender Horns",
  duration: 1440,
};

let failures = 0;
function check(ok, what) {
  console.log(`${ok ? "ok  " : "FAIL"} ${what}`);
  if (!ok) failures++;
}

/** A socket with a queue of parsed frames and a wait for the next match. */
class Client {
  constructor(name) {
    this.name = name;
    this.frames = [];
    this.waiters = [];
    this.closed = false;
  }

  open(code) {
    return new Promise((resolve, reject) => {
      this.ws = new WebSocket(`${wsBase}/rooms/${code}/ws`);
      this.ws.addEventListener("open", () => resolve(this));
      this.ws.addEventListener("error", (e) => reject(new Error(`${this.name}: ${e.message ?? "socket error"}`)));
      this.ws.addEventListener("close", () => {
        this.closed = true;
        this.flush();
      });
      this.ws.addEventListener("message", (e) => {
        this.frames.push(JSON.parse(e.data));
        this.flush();
      });
    });
  }

  send(msg) {
    this.ws.send(typeof msg === "string" ? msg : JSON.stringify(msg));
  }

  flush() {
    for (const w of [...this.waiters]) {
      const i = this.frames.findIndex(w.match);
      if (i >= 0) {
        const [frame] = this.frames.splice(0, i + 1).slice(-1);
        this.waiters.splice(this.waiters.indexOf(w), 1);
        clearTimeout(w.timer);
        w.resolve(frame);
      } else if (w.untilClose && this.closed) {
        this.waiters.splice(this.waiters.indexOf(w), 1);
        clearTimeout(w.timer);
        w.resolve(null);
      }
    }
  }

  /** The next frame matching `match`, skipping others; null on timeout. */
  next(match, ms = 5000) {
    return new Promise((resolve) => {
      const w = { match, resolve, timer: null };
      w.timer = setTimeout(() => {
        this.waiters.splice(this.waiters.indexOf(w), 1);
        resolve(null);
      }, ms);
      this.waiters.push(w);
      this.flush();
    });
  }

  waitClosed(ms = 5000) {
    if (this.closed) return Promise.resolve(true);
    return new Promise((resolve) => {
      const t = setTimeout(() => resolve(false), ms);
      this.ws.addEventListener("close", () => {
        clearTimeout(t);
        resolve(true);
      });
    });
  }

  close() {
    try {
      this.ws.close();
    } catch {}
  }
}

/** The status a WebSocket upgrade request to `path` gets. fetch refuses to send one. */
function upgradeStatus(path) {
  return new Promise((resolve, reject) => {
    const req = request(`${base}${path}`, {
      headers: {
        connection: "Upgrade",
        upgrade: "websocket",
        "sec-websocket-version": "13",
        // Any 16 bytes, base64: the handshake only needs the shape.
        "sec-websocket-key": Buffer.from("smoke test nonce").toString("base64"),
      },
    });
    req.on("response", (res) => {
      res.resume();
      resolve(res.statusCode);
    });
    req.on("upgrade", (res, socket) => {
      socket.destroy();
      resolve(res.statusCode);
    });
    req.on("error", reject);
    req.end();
  });
}

const isT = (t) => (f) => f.t === t;

async function createRoom() {
  const res = await fetch(`${base}/rooms`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ episode }),
  });
  return { status: res.status, body: res.status === 201 ? await res.json() : await res.text() };
}

async function rooms() {
  const health = await fetch(`${base}/health`);
  check(health.status === 200 && (await health.text()) === "ok", "GET /health answers 200 ok");

  const bad = await fetch(`${base}/rooms`, { method: "POST", body: "{}" });
  check(bad.status === 400, `POST /rooms without an episode is 400 (got ${bad.status})`);

  const made = await createRoom();
  check(made.status === 201, `POST /rooms is 201 (got ${made.status})`);
  const { code, host_key } = made.body;
  check(/^[A-HJ-NP-Z2-9]{6}$/.test(code), `the code ${code} uses the alphabet`);
  check(/^[0-9a-f]{32}$/.test(host_key), "the host key is 32 hex");

  const page = await fetch(`${base}/${code.toLowerCase()}`);
  const html = await page.text();
  check(page.status === 200, `GET /{code} is 200 (got ${page.status})`);
  check(html.includes('<meta property="og:title" content="Watch Mushishi together">'), "the page names the series for previews");
  check(html.includes(`Episode 3: Tender Horns. Room ${code}`), "the page names the episode for previews");
  check(html.includes(`anibeam://room/${code}`), "the page opens the app");

  const find = await fetch(`${base}/`);
  check(find.status === 200 && (await find.text()).includes("Join a room"), "GET / is the find page");
  const typed = await fetch(`${base}/?code=${code.slice(0, 3).toLowerCase()}-${code.slice(3)}`, { redirect: "manual" });
  check(typed.status === 303 && typed.headers.get("location") === `/${code}`, "GET /?code= goes to that room's page");

  const missing = await fetch(`${base}/ZZZZZZ`);
  check(missing.status === 404 && (await missing.text()).includes("Room not found"), "a page for no room is 404 and says so");
  const nonsense = await fetch(`${base}/not-a-code`);
  check(nonsense.status === 404, "a page for a malformed code is 404");
  const noRoomWs = await upgradeStatus("/rooms/ZZZZZZ/ws");
  check(noRoomWs === 404, `a socket to no room is 404 (got ${noRoomWs})`);
  const notUpgrade = await fetch(`${base}/rooms/${code}/ws`);
  check(notUpgrade.status === 426, `a plain GET on a room socket is 426 (got ${notUpgrade.status})`);

  const a = await new Client("host").open(code);
  a.send({ t: "ping", c: 1 });
  const early = await a.next(isT("error"));
  check(early?.code === "proto", "a message before hello gets proto");
  check(await a.waitClosed(), "and the relay closes that socket");

  const host = await new Client("host").open(code);
  host.send({ t: "hello", name: "Ann", proto: 1, client: "smoke/1", host_key });
  const hw = await host.next(isT("welcome"));
  check(hw?.room?.members?.[0]?.host === true, "the host key makes the host");

  const guest = await new Client("guest").open(code);
  guest.send({ t: "hello", name: "Ben", proto: 1, client: "smoke/1" });
  const gw = await guest.next(isT("welcome"));
  check(gw?.room?.members?.length === 2, "the guest sees two members");
  const joined = await host.next((f) => f.t === "members" && f.members.length === 2);
  check(!!joined, "the host hears the guest join");

  host.send({ t: "cmd", action: "play", position: 12.5 });
  const s1 = await host.next(isT("state"));
  const s2 = await guest.next(isT("state"));
  check(s1?.state?.playing === true && s1.state.position === 12.5 && s1.by === hw.you, "play reaches the host");
  check(s2?.state?.playing === true && s2.state.position === 12.5 && s2.state.rev === s1.state.rev, "play reaches the guest");

  guest.send({ t: "cmd", action: "seek", position: 30 });
  const s3 = await host.next(isT("state"));
  const s4 = await guest.next(isT("state"));
  check(s3?.state?.position === 30 && s3.state.playing === true && s3.by === gw.you, "the guest's seek reaches the host and keeps playing");
  check(s4?.state?.rev === s1.state.rev + 1, "and the guest, one revision on");

  guest.send({ t: "ping", c: 42 });
  const pong = await guest.next(isT("pong"));
  check(pong?.c === 42 && pong.s > 1.7e12, "ping gets pong with the relay clock");

  guest.send("not json");
  const badMsg = await guest.next(isT("error"));
  check(badMsg?.code === "bad_message" && !guest.closed, "bad JSON gets bad_message and the socket stays");

  guest.send({ t: "episode", episode: { ...episode, episode_key: "s1e4" } });
  const notHost = await guest.next(isT("error"));
  check(notHost?.code === "not_allowed", "only the host changes the episode");

  guest.send({ t: "bye" });
  const bye = await guest.next(isT("closed"));
  check(bye?.reason === "bye", "bye gets closed bye");
  check(await guest.waitClosed(), "and the relay closes the guest");
  const left = await host.next((f) => f.t === "members" && f.members.length === 1);
  check(!!left, "the host hears the guest leave");

  writeFileSync(stateFile, JSON.stringify({ code, host_key, rejoin_key: hw.rejoin_key, you: hw.you }));
  // The host stays connected until the relay is stopped under it.
}

async function reload() {
  const { code, rejoin_key, you } = JSON.parse(readFileSync(stateFile, "utf8"));
  const page = await fetch(`${base}/${code}`);
  check(page.status === 200, `after a restart the room is still there (got ${page.status})`);
  const host = await new Client("host").open(code);
  host.send({ t: "hello", name: "Ann", proto: 1, client: "smoke/1", rejoin_key });
  const w = await host.next(isT("welcome"));
  check(w?.you === you, "the rejoin key gives the same member back");
  check(w?.room?.state?.position === 30 && w.room.state.playing === true, "the play state survived");
  check(w?.room?.host === you, "and the host role");
  host.close();
}

async function timers() {
  const made = await createRoom();
  const { code, host_key } = made.body;
  const host = await new Client("host").open(code);
  host.send({ t: "hello", name: "Ann", proto: 1, client: "smoke/1", host_key });
  const hw = await host.next(isT("welcome"));
  const guest = await new Client("guest").open(code);
  guest.send({ t: "hello", name: "Ben", proto: 1, client: "smoke/1" });
  const gw = await guest.next(isT("welcome"));
  host.close();
  const dropped = await guest.next((f) => f.t === "members" && f.members.length === 1);
  check(!!dropped && dropped.members[0].host === false, "the host dropping leaves the guest without a host");
  const started = Date.now();
  const handed = await guest.next((f) => f.t === "members" && f.members[0]?.host === true, 150_000);
  const secs = Math.round((Date.now() - started) / 1000);
  check(!!handed && handed.members[0].id === gw.you && hw.you !== gw.you, `the alarm hands the host role over after the grace (${secs} s)`);
  guest.close();
}

async function limits() {
  const made = await createRoom();
  check(made.status === 201, `a room for the limit checks (got ${made.status})`);
  const { code } = made.body;

  const statuses = [];
  for (let i = 0; i < 12; i++) statuses.push((await createRoom()).status);
  check(statuses.includes(429), `room creations past 10 a minute get 429 (${statuses.join(" ")})`);

  // A socket that has not said hello is held to the rate all the same.
  const junk = await new Client("junk").open(code);
  for (let i = 0; i < 25; i++) junk.send("junk");
  const limited = await junk.next((f) => f.t === "error" && f.code === "rate_limited");
  check(!!limited && !junk.closed, "a socket before hello gets rate_limited past 20 a second, and stays");
  junk.close();

  // One that never says hello is closed after 10 s.
  const silent = await new Client("silent").open(code);
  const started = Date.now();
  const late = await silent.next((f) => f.t === "error" && f.code === "proto", 15_000);
  const shut = await silent.waitClosed(5_000);
  const secs = Math.round((Date.now() - started) / 1000);
  check(!!late && shut && secs >= 9, `a socket that never says hello is closed (${secs} s)`);

  const pages = [];
  for (let i = 0; i < 62; i++) pages.push((await fetch(`${base}/${code}`)).status);
  check(pages.includes(429), `join pages past 60 a minute get 429 (${pages.filter((s) => s === 429).length} refused)`);
  const nonsense = await fetch(`${base}/not-a-code`);
  check(nonsense.status === 404, `a page for a malformed code is still 404 (got ${nonsense.status})`);
}

async function close() {
  // The limits phase may have used up this minute's creations.
  let made = await createRoom();
  for (let i = 0; made.status === 429 && i < 15; i++) {
    await new Promise((r) => setTimeout(r, 5_000));
    made = await createRoom();
  }
  check(made.status === 201, `a room for the close check (got ${made.status})`);
  const { code } = made.body;
  writeFileSync(stateFile, JSON.stringify({ code }));
  const started = Date.now();
  let status = 200;
  while (status === 200 && Date.now() - started < 700_000) {
    await new Promise((r) => setTimeout(r, 15_000));
    status = (await fetch(`${base}/${code}`)).status;
  }
  const secs = Math.round((Date.now() - started) / 1000);
  check(status === 404 && secs >= 600, `an empty room closes after 10 minutes (${secs} s, then ${status})`);
}

async function gone() {
  const { code } = JSON.parse(readFileSync(stateFile, "utf8"));
  const status = (await fetch(`${base}/${code}`)).status;
  check(status === 404, `after a restart the closed room is still gone (got ${status})`);
}

const phases = { rooms, reload, timers, limits, close, gone };
if (!phases[phase]) {
  console.error(`unknown phase ${phase}`);
  process.exit(2);
}
await phases[phase]();
console.log(failures === 0 ? `${phase}: all passed` : `${phase}: ${failures} failed`);
process.exit(failures === 0 ? 0 : 1);
