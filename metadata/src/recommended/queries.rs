//! The GraphQL the recommender sends. The batches use aliases, so one request
//! carries several lookups: AniList counts a request against its limit, not
//! what is inside it.

/// What the recommender reads for every show it knows: what the models
/// learn from (genres, tags with their spoiler flags, studios, the staff who
/// shape the look, relations) and what a tile and its hover card draw.
const MEDIA_FIELDS: &str = "
  id idMal status format episodes duration seasonYear averageScore popularity isAdult genres
  description(asHtml: false) siteUrl bannerImage
  title { romaji english native }
  coverImage { extraLarge large color }
  tags { name rank category isMediaSpoiler isGeneralSpoiler isAdult }
  studios(isMain: true) { nodes { name } }
  staff(sort: [RELEVANCE, ID], perPage: 12) { edges { role node { id name { full } } } }
  relations { edges { relationType node { id type } } }
";

/// Up to `n` shows by id, one request.
pub fn shows(n: usize) -> String {
    format!(
        "query ($ids: [Int]) {{ Page(perPage: {n}) {{ media(id_in: $ids, type: ANIME) {{ {MEDIA_FIELDS} }} }} }}"
    )
}

/// The latest finishers of each favourite, which is where people with the
/// user's taste are found.
pub fn raters(shows: &[u64]) -> String {
    let parts: Vec<String> = shows
        .iter()
        .map(|id| {
            format!(
                "m{id}: Page(perPage: 50) {{ mediaList(mediaId: {id}, status: COMPLETED, sort: UPDATED_TIME_DESC) \
                 {{ userId score(format: POINT_100) }} }}"
            )
        })
        .collect();
    format!("query {{ {} }}", parts.join(" "))
}

/// Other users' whole lists, scores out of 100.
pub fn user_lists(users: &[u64]) -> String {
    let parts: Vec<String> = users
        .iter()
        .map(|id| {
            format!(
                "u{id}: MediaListCollection(userId: {id}, type: ANIME) \
                 {{ lists {{ isCustomList entries {{ mediaId status score(format: POINT_100) repeat }} }} }}"
            )
        })
        .collect();
    format!("query {{ {} }}", parts.join(" "))
}

/// AniList's community recommendations for each show: "if you liked this".
pub fn similar(shows: &[u64]) -> String {
    let parts: Vec<String> = shows
        .iter()
        .map(|id| {
            format!(
                "m{id}: Media(id: {id}) {{ recommendations(perPage: 10, sort: RATING_DESC) \
                 {{ nodes {{ rating mediaRecommendation {{ id type }} }} }} }}"
            )
        })
        .collect();
    format!("query {{ {} }}", parts.join(" "))
}

/// The show page's own read: everything a tile knows, plus the trailer,
/// every studio, the relations by name and AniList's recommendations.
pub fn page() -> String {
    format!(
        "query ($id: Int) {{ Media(id: $id, type: ANIME) {{ {MEDIA_FIELDS}
           trailer {{ id site }}
           allStudios: studios {{ nodes {{ name isAnimationStudio }} }}
           named: relations {{ edges {{ relationType node {{ id type format
             title {{ romaji english native }} coverImage {{ large }} }} }} }}
           recommendations(perPage: 8, sort: RATING_DESC) {{ nodes {{ rating mediaRecommendation {{
             id type title {{ romaji english native }} coverImage {{ large }} }} }} }}
         }} }}"
    )
}
