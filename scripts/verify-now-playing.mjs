import assert from 'node:assert/strict';

const { broadcastShowTitle, isRealEpisodeTitle, nowPlayingLines } =
  await import('../src/shared/nowPlaying.ts');

// These lines are what Linux MPRIS widgets show while the in-window player
// runs: `title` is xesam:title, `artist` is xesam:artist. Nothing is ever
// truncated and the show name is never repeated on both lines.

const SHOW = 'Sousou no Frieren';
const SEP = ' \u00b7 ';

// --- real episode title: episode name on top, show plus number below ---
assert.deepEqual(
  nowPlayingLines({ showTitle: SHOW, episodeNumber: 1, episodeTitle: "The Journey's End" }),
  { title: "The Journey's End", artist: `${SHOW}${SEP}Episode 1` },
);
// Long titles pass through untouched.
const LONG = 'A Very Long Episode Title That Would Get Cut Off By A Widget If We Let It, But We Never Truncate';
assert.equal(nowPlayingLines({ showTitle: SHOW, episodeNumber: 12, episodeTitle: LONG }).title, LONG);
// Surrounding whitespace on the title is trimmed, the words are not.
assert.equal(nowPlayingLines({ showTitle: SHOW, episodeNumber: 3, episodeTitle: '  Killing Magic  ' }).title, 'Killing Magic');

// --- no episode title: show on top, number only below (no show repeat) ---
assert.deepEqual(
  nowPlayingLines({ showTitle: SHOW, episodeNumber: 5, episodeTitle: null }),
  { title: SHOW, artist: 'Episode 5' },
);
assert.deepEqual(
  nowPlayingLines({ showTitle: SHOW, episodeNumber: 5, episodeTitle: undefined }),
  { title: SHOW, artist: 'Episode 5' },
);
// Decimal episode numbers print as given.
assert.equal(nowPlayingLines({ showTitle: SHOW, episodeNumber: 6.5, episodeTitle: null }).artist, 'Episode 6.5');

// --- placeholder titles are treated as no title ---
const placeholders = [
  '',
  '   ',
  SHOW,
  'sousou no frieren',            // case-folded
  '  Sousou   no Frieren ',       // whitespace collapsed
  'Episode 5',
  'episode 5',
  'Ep 5',
  'Ep. 5',
  'E05',
  'e5',
  '5',
  '05',
  '#5',
  `${SHOW} - 05`,
  `${SHOW} \u2013 5`,             // U+2013
  `${SHOW} \u2014 5`,             // U+2014
  `${SHOW}: Episode 5`,
  `${SHOW}_Ep05`,
  `${SHOW} Episode 5`,
  `${SHOW} E05`,
  `${SHOW} #5`,
  `${SHOW} 5`,
];
for (const placeholder of placeholders) {
  assert.equal(isRealEpisodeTitle(placeholder, SHOW, 5), false, `placeholder: ${JSON.stringify(placeholder)}`);
  assert.deepEqual(
    nowPlayingLines({ showTitle: SHOW, episodeNumber: 5, episodeTitle: placeholder }),
    { title: SHOW, artist: 'Episode 5' },
    `lines for placeholder: ${JSON.stringify(placeholder)}`,
  );
}

// --- titles that are real and must survive ---
const real = [
  "The Journey's End",
  'Episode of the Mage',           // "Episode" as a word, not a token
  `${SHOW} Returns`,               // show name plus real words
  `${SHOW} - The Final Battle`,
  'Part 5',                        // not one of the accepted number tokens
  '5 Years Later',
  'Episode 7',                     // a different number is not this episode's token
];
for (const title of real) {
  assert.equal(isRealEpisodeTitle(title, SHOW, 5), true, `real: ${JSON.stringify(title)}`);
}
assert.deepEqual(
  nowPlayingLines({ showTitle: SHOW, episodeNumber: 5, episodeTitle: `${SHOW} - The Final Battle` }),
  { title: `${SHOW} - The Final Battle`, artist: `${SHOW}${SEP}Episode 5` },
);

// --- isRealEpisodeTitle edge cases ---
assert.equal(isRealEpisodeTitle(null, SHOW, 5), false);
assert.equal(isRealEpisodeTitle(undefined, SHOW, 5), false);
// Numeric comparison: "05" is episode 5, but not episode 50.
assert.equal(isRealEpisodeTitle('05', SHOW, 50), true);
assert.equal(isRealEpisodeTitle('Episode 5', SHOW, 5), false);
// No episode number to compare against: any bare token is still not a name.
assert.equal(isRealEpisodeTitle('Episode 5', SHOW, null), false);
// Empty show name: only the bare-token rules apply.
assert.equal(isRealEpisodeTitle('E05', '', 5), false);
assert.equal(isRealEpisodeTitle('A Name', '', 5), true);

// --- extras: label on top, show below, no episode number ---
assert.deepEqual(
  nowPlayingLines({ showTitle: SHOW, episodeNumber: 1, episodeTitle: "The Journey's End", extraLabel: 'Opening 1' }),
  { title: 'Opening 1', artist: SHOW },
);
assert.deepEqual(
  nowPlayingLines({ showTitle: SHOW, episodeNumber: null, episodeTitle: null, extraLabel: 'Ending 2' }),
  { title: 'Ending 2', artist: SHOW },
);
// An empty extra label is not an extra.
assert.deepEqual(
  nowPlayingLines({ showTitle: SHOW, episodeNumber: 5, episodeTitle: null, extraLabel: '' }),
  { title: SHOW, artist: 'Episode 5' },
);

// --- no episode number and no extra: show alone ---
assert.deepEqual(
  nowPlayingLines({ showTitle: SHOW, episodeNumber: null, episodeTitle: 'Ignored' }),
  { title: SHOW, artist: '' },
);

// --- empty show name never leaves a dangling separator ---
assert.deepEqual(
  nowPlayingLines({ showTitle: '', episodeNumber: 2, episodeTitle: 'A Name' }),
  { title: 'A Name', artist: 'Episode 2' },
);

// --- broadcastShowTitle: romaji, then English, then title, then folder ---
assert.equal(broadcastShowTitle({ titleRomaji: SHOW, titleEnglish: "Frieren: Beyond Journey's End", title: 'Frieren' }), SHOW);
assert.equal(broadcastShowTitle({ titleRomaji: '', titleEnglish: "Frieren: Beyond Journey's End", title: 'Frieren' }), "Frieren: Beyond Journey's End");
assert.equal(broadcastShowTitle({ titleRomaji: null, titleEnglish: null, title: 'Frieren' }), 'Frieren');
assert.equal(broadcastShowTitle({ titleRomaji: '   ', titleEnglish: undefined, title: '' , folderPath: '/mnt/anime/Sousou no Frieren' }), 'Sousou no Frieren');
assert.equal(broadcastShowTitle({ folderPath: '/mnt/anime/Sousou no Frieren/' }), 'Sousou no Frieren');
assert.equal(broadcastShowTitle({ folderPath: 'C:\\Anime\\Frieren' }), 'Frieren');
assert.equal(broadcastShowTitle({ titleRomaji: null, titleEnglish: null, title: null, folderPath: null }), '');
assert.equal(broadcastShowTitle({}), '');
assert.equal(broadcastShowTitle(null), '');
assert.equal(broadcastShowTitle(undefined), '');

console.log('verify-now-playing: ok');
