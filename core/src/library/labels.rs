//! Display codes and labels for extras and episode codes. Shared by the
//! series page and the player header through the records, so the two never
//! drift. Carried from src/shared/extraLabels.ts.

use crate::contract::ExtraKind;
use super::classifier::format_number;

pub fn extra_code(kind: ExtraKind) -> &'static str {
    match kind {
        ExtraKind::Op => "OP",
        ExtraKind::Ed => "ED",
        ExtraKind::Pv => "PV",
        ExtraKind::Sp => "SP",
        ExtraKind::Other => "EXTRA",
    }
}

/// `Extra.code`: the code plus the index, `OP1`; the bare code without one.
pub fn extra_code_with_index(kind: ExtraKind, index: Option<u32>) -> String {
    match index {
        Some(i) => format!("{}{i}", extra_code(kind)),
        None => extra_code(kind).to_string(),
    }
}

pub fn extra_label(kind: ExtraKind, index: Option<u32>, variant: Option<&str>, raw_label: Option<&str>) -> String {
    let idx_part = match (index, variant) {
        (Some(i), Some(v)) => format!(" {i}{v}"),
        (Some(i), None) => format!(" {i}"),
        (None, Some(v)) => format!(" {v}"),
        (None, None) => String::new(),
    };
    match kind {
        ExtraKind::Op => format!("Opening{idx_part}"),
        ExtraKind::Ed => format!("Ending{idx_part}"),
        ExtraKind::Pv => format!("Preview{idx_part}"),
        ExtraKind::Sp => format!("Special{idx_part}"),
        ExtraKind::Other => match raw_label {
            Some(raw) if !raw.is_empty() => {
                let mut chars = raw.chars();
                let first = chars.next().unwrap().to_uppercase().collect::<String>();
                format!("{first}{}", chars.as_str().to_lowercase())
            }
            _ => "Bonus".to_string(),
        },
    }
}

/// `S01E03` with a season, `EP 3` without, the number rendered canonically.
pub fn episode_code(season: Option<u32>, number: f64) -> String {
    match season {
        Some(s) => {
            let n = if number.fract() == 0.0 { format!("{:02}", number as i64) } else { format_number(number) };
            format!("S{s:02}E{n}")
        }
        None => format!("EP {}", format_number(number)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::ExtraKind;

    #[test]
    fn codes_and_labels() {
        assert_eq!(extra_code(ExtraKind::Op), "OP");
        assert_eq!(extra_code(ExtraKind::Other), "EXTRA");
        assert_eq!(extra_code_with_index(ExtraKind::Op, Some(1)), "OP1");
        assert_eq!(extra_code_with_index(ExtraKind::Sp, None), "SP");
        assert_eq!(extra_label(ExtraKind::Op, Some(4), Some("a"), Some("OP4a")), "Opening 4a");
        assert_eq!(extra_label(ExtraKind::Ed, Some(1), None, Some("ED1")), "Ending 1");
        assert_eq!(extra_label(ExtraKind::Pv, Some(12), None, Some("PV12")), "Preview 12");
        assert_eq!(extra_label(ExtraKind::Sp, None, None, Some("Special")), "Special");
        assert_eq!(extra_label(ExtraKind::Sp, Some(2), None, Some("SP2")), "Special 2");
        assert_eq!(extra_label(ExtraKind::Other, None, None, Some("BONUS")), "Bonus");
        assert_eq!(extra_label(ExtraKind::Other, None, None, Some("cm")), "Cm");
        assert_eq!(extra_label(ExtraKind::Other, None, None, None), "Bonus");
        assert_eq!(episode_code(Some(1), 3.0), "S01E03");
        assert_eq!(episode_code(Some(2), 12.5), "S02E12.5");
        assert_eq!(episode_code(None, 3.0), "EP 3");
        assert_eq!(episode_code(Some(0), 0.0), "S00E00");
    }
}
