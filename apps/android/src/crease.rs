//! Where the crease runs, as the phone reports it: Android's FoldingFeature, read by
//! `Crease.java` and packed into one number on the way over JNI. A book foldable's crease
//! runs top to bottom at rotation 0 and a clamshell's left to right, so the layout reads
//! this rather than guessing from the rotation, and only guesses on a phone that says
//! nothing.

/// A fold that crosses the window, in the window's pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Crease {
    /// The crease runs from left to right, as a clamshell's does held upright.
    pub across: bool,
    /// The phone is half open, by the platform's own reckoning.
    pub half: bool,
    /// Where the crease starts and ends across the axis it crosses: its top and bottom
    /// when it runs across, its left and right edges otherwise. The same for a fold with
    /// no width, and a hinge's gap between them.
    pub start: f32,
    pub end: f32,
}

/// What the phone said about its crease.
#[derive(Clone, Copy, Debug, PartialEq)]
// Off the phone nothing is ever reported, so only Silent is made there.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub enum Reading {
    /// The phone has no window extensions, so it never will say: lay out by the guess.
    Silent,
    /// No fold crosses the window now: folded shut, beside another app, or not said yet.
    None,
    Fold(Crease),
}

/// Unpacks `Crease.at`'s answer: -2 for a phone without extensions, -1 for no fold, and
/// otherwise bit 40 for across, bit 41 for half open, bits 20 to 39 the start and bits 0
/// to 19 the end.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub fn decode(packed: i64) -> Reading {
    match packed {
        -2 => Reading::Silent,
        p if p < 0 => Reading::None,
        p => Reading::Fold(Crease {
            across: p & (1 << 40) != 0,
            half: p & (1 << 41) != 0,
            start: ((p >> 20) & 0xF_FFFF) as f32,
            end: (p & 0xF_FFFF) as f32,
        }),
    }
}

/// The crease as the layout takes it, in the window's logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    /// The phone said where the crease is; otherwise the layout guesses a book fold.
    pub known: bool,
    /// The crease runs from left to right.
    pub across: bool,
    /// Half open by the platform's own reckoning, for a phone with no hinge sensor.
    pub half: bool,
    pub start: f32,
    pub end: f32,
}

/// Where the layout puts the crease: the phone's own report when it gave one, scaled from
/// pixels by `scale`, or else a book fold's, whose crease runs top to bottom until the
/// display is turned a quarter (`rotated`).
pub fn layout(reading: Reading, rotated: bool, scale: f32) -> Layout {
    match reading {
        Reading::Fold(c) => Layout {
            known: true,
            across: c.across,
            half: c.half,
            start: c.start / scale,
            end: c.end / scale,
        },
        Reading::None | Reading::Silent => Layout {
            known: false,
            across: rotated,
            half: false,
            start: 0.0,
            end: 0.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack(across: bool, half: bool, start: i64, end: i64) -> i64 {
        (start << 20) | end | (i64::from(across) << 40) | (i64::from(half) << 41)
    }

    #[test]
    fn a_clamshell_held_upright_creases_across() {
        let r = decode(pack(true, true, 1318, 1318));
        assert_eq!(
            r,
            Reading::Fold(Crease {
                across: true,
                half: true,
                start: 1318.0,
                end: 1318.0
            })
        );
    }

    #[test]
    fn a_book_fold_creases_top_to_bottom_with_its_hinge_gap() {
        let r = decode(pack(false, false, 1070, 1110));
        assert_eq!(
            r,
            Reading::Fold(Crease {
                across: false,
                half: false,
                start: 1070.0,
                end: 1110.0
            })
        );
    }

    #[test]
    fn a_crease_at_the_left_edge_is_still_a_crease() {
        assert!(matches!(decode(pack(false, false, 0, 0)), Reading::Fold(_)));
    }

    #[test]
    fn a_clamshell_creases_across_even_at_rotation_zero() {
        let reading = decode(pack(true, true, 2400, 2400));
        let l = layout(reading, false, 2.0);
        assert!(l.known && l.across && l.half);
        assert_eq!((l.start, l.end), (1200.0, 1200.0));
    }

    #[test]
    fn a_book_fold_turned_a_quarter_creases_across_by_its_own_report() {
        let l = layout(decode(pack(true, false, 900, 940)), true, 2.0);
        assert!(l.known && l.across && !l.half);
        assert_eq!((l.start, l.end), (450.0, 470.0));
    }

    #[test]
    fn a_phone_that_says_nothing_is_taken_for_a_book_fold() {
        for reading in [Reading::None, Reading::Silent] {
            let upright = layout(reading, false, 2.0);
            assert!(!upright.known && !upright.across);
            assert!(layout(reading, true, 2.0).across);
        }
    }

    #[test]
    fn no_fold_and_no_extensions_read_apart() {
        assert_eq!(decode(-1), Reading::None);
        assert_eq!(decode(-2), Reading::Silent);
    }
}
