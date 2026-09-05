//! The small pieces every query builder in the core needs: the `IN` list's
//! placeholders and the two casts between a `u64` id and the `i64` a SQLite
//! column holds.

/// `?,?,?` for an `IN` list of `n` values. Ids go in as bound parameters,
/// never formatted into the SQL. SQLite accepts an empty `IN ()` and reads
/// it as false, so a scope that matches no rows loads nothing rather than
/// failing to parse.
pub(crate) fn placeholders(n: usize) -> String {
    let mut out = String::with_capacity(n * 2);
    for i in 0..n {
        if i > 0 {
            out.push(',');
        }
        out.push('?');
    }
    out
}

/// An id into a column. Ids come from AniList and fit an i64 many times
/// over, so the saturation is a promise never to panic rather than a
/// number anything will ever see.
pub(crate) fn as_i64(v: u64) -> i64 {
    i64::try_from(v).unwrap_or(i64::MAX)
}

/// An id back out of a column. A negative id is not a thing the schema can
/// hold, so it reads as nought rather than wrapping around.
pub(crate) fn as_u64(v: i64) -> u64 {
    u64::try_from(v).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_in_list_is_one_placeholder_per_value_and_empty_for_none() {
        assert_eq!(placeholders(0), "");
        assert_eq!(placeholders(1), "?");
        assert_eq!(placeholders(3), "?,?,?");
    }

    #[test]
    fn the_casts_saturate_rather_than_wrapping() {
        assert_eq!(as_i64(7), 7);
        assert_eq!(as_i64(u64::MAX), i64::MAX);
        assert_eq!(as_u64(7), 7);
        assert_eq!(as_u64(-1), 0);
    }
}
