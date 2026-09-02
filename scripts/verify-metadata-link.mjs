// Run: bun --bun scripts/verify-metadata-link.mjs
//
// The match modal accepts a pasted AniList, MyAnimeList or TMDB URL in the
// same box as a search query. This pins the split: what counts as a URL at
// all (null means "search for it"), which URLs resolve to a provider id, and
// which look like a link but carry nothing we can look up (unknown).

import assert from 'node:assert/strict';

const { parseMetadataLink, describeMetadataLink } = await import('../src/shared/metadataLink.ts');

const anilist = (id) => ({ provider: 'anilist', id });
const mal = (id) => ({ provider: 'mal', id });
const tmdb = (kind, id) => ({ provider: 'tmdb', kind, id });
const unknown = { provider: 'unknown' };

let cases = 0;
const check = (text, expected) => {
  cases += 1;
  assert.deepEqual(parseMetadataLink(text), expected, `parse ${JSON.stringify(text)}`);
};

// --- AniList: /anime/{id}, slug and trailing slash optional ---
check('https://anilist.co/anime/21/ONE-PIECE/', anilist(21));
check('https://anilist.co/anime/21', anilist(21));
// People paste from the address bar without the scheme.
check('anilist.co/anime/21', anilist(21));
check('www.anilist.co/anime/21', anilist(21));
// Scheme and host are case-insensitive.
check('HTTPS://ANILIST.CO/anime/21', anilist(21));
// Query and fragment carry nothing for the known forms.
check('https://anilist.co/anime/21?foo=bar#x', anilist(21));
// Surrounding whitespace comes free with a paste.
check('  https://anilist.co/anime/21  ', anilist(21));
// Manga has no place in a video library.
check('https://anilist.co/manga/30013/ONE-PIECE/', unknown);
check('https://anilist.co/user/somebody', unknown);

// --- MyAnimeList: /anime/{id} and the legacy anime.php?id= form ---
check('https://myanimelist.net/anime/5114/Fullmetal_Alchemist__Brotherhood', mal(5114));
check('https://myanimelist.net/anime/5114', mal(5114));
check('https://myanimelist.net/anime.php?id=5114', mal(5114));
check('myanimelist.net/anime/5114', mal(5114));
check('https://myanimelist.net/manga/25/Fullmetal_Alchemist', unknown);
check('https://myanimelist.net/anime.php?id=abc', unknown);
check('https://myanimelist.net/anime.php', unknown);

// --- TMDB: /movie/{id} or /tv/{id}, id may carry a slug, trailing segments allowed ---
check('https://www.themoviedb.org/movie/550-fight-club', tmdb('movie', 550));
check('https://www.themoviedb.org/movie/550', tmdb('movie', 550));
check('https://www.themoviedb.org/tv/1399-game-of-thrones', tmdb('tv', 1399));
check('https://www.themoviedb.org/tv/1399/season/2', tmdb('tv', 1399));
check('https://www.themoviedb.org/tv/1399-game-of-thrones/watch?locale=US', tmdb('tv', 1399));
check('themoviedb.org/tv/1399', tmdb('tv', 1399));
// People, collections and searches are not something we can match a series to.
check('https://www.themoviedb.org/person/287-brad-pitt', unknown);
check('https://www.themoviedb.org/collection/10-star-wars-collection', unknown);
check('https://www.themoviedb.org/search?query=fight+club', unknown);

// --- Bad ids read as a link with nothing to look up ---
check('https://anilist.co/anime/0', unknown);
check('https://anilist.co/anime/abc', unknown);
check('https://anilist.co/anime/', unknown);
check('https://anilist.co/anime', unknown);
check('https://anilist.co/anime/21abc', unknown);
check('https://anilist.co/anime/-21', unknown);
check('https://www.themoviedb.org/movie/abc-fight-club', unknown);
check('https://www.themoviedb.org/movie/0-nothing', unknown);

// --- Any other host is a link we do not understand ---
check('https://example.com/anime/21', unknown);
check('http://example.com', unknown);

// --- Not a URL at all: hand it to search ---
check('One Piece', null);
check('', null);
check('   ', null);
check('anime/21', null);
// Only the three known hosts get the scheme-less treatment.
check('example.com/anime/21', null);
check('anilist.co', null);
check('anilist.com/anime/21', null);
// A scheme alone with nothing usable behind it is not a URL.
check('https://', null);
check('http://', null);

// --- describeMetadataLink ---
const describe = (link, expected) => {
  cases += 1;
  assert.equal(describeMetadataLink(link), expected);
};
describe(anilist(21), 'AniList #21');
describe(mal(5114), 'MyAnimeList #5114');
describe(tmdb('movie', 550), 'TMDB film #550');
describe(tmdb('tv', 1399), 'TMDB show #1399');
describe(unknown, 'Link');

console.log(`verify-metadata-link: ${cases} cases passed`);
