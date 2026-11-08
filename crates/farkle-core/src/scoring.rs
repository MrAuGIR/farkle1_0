//! Calcul des points d'une sélection de dés (règles KCD, voir `docs/REGLES.md`).

use crate::dice::Die;
use std::fmt;

/// Motif de scoring élémentaire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Pattern {
    /// Un 1 (100) ou un 5 (50) isolé.
    Single(u8),
    /// `count` dés identiques (3 à 6) de valeur `face`.
    OfAKind { face: u8, count: u8 },
    /// Suite 1-2-3-4-5 (500).
    LowStraight,
    /// Suite 2-3-4-5-6 (750).
    HighStraight,
    /// Suite 1-2-3-4-5-6 (1500).
    FullStraight,
}

impl Pattern {
    /// Points rapportés par ce motif.
    pub fn points(self) -> u32 {
        todo!()
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = f;
        todo!()
    }
}

/// Meilleur découpage de **toute** la sélection en motifs.
/// `None` si la sélection est vide ou contient un dé qui ne marque pas.
pub fn best_partition(selection: &[Die]) -> Option<(u32, Vec<Pattern>)> {
    let _ = selection;
    todo!()
}

/// Points d'une sélection, `None` si elle est vide ou contient un dé qui ne marque pas.
pub fn score_selection(selection: &[Die]) -> Option<u32> {
    best_partition(selection).map(|(points, _)| points)
}

/// Vrai si le lancer ne contient aucun dé qui marque.
pub fn is_farkle(roll: &[Die]) -> bool {
    let _ = roll;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::dice_from;

    fn score(values: &[u8]) -> Option<u32> {
        score_selection(&dice_from(values).unwrap())
    }

    fn farkle(values: &[u8]) -> bool {
        is_farkle(&dice_from(values).unwrap())
    }

    #[test]
    fn singles() {
        assert_eq!(score(&[1]), Some(100));
        assert_eq!(score(&[5]), Some(50));
        assert_eq!(score(&[1, 5]), Some(150));
        assert_eq!(score(&[1, 1]), Some(200));
        assert_eq!(score(&[5, 5]), Some(100));
    }

    #[test]
    fn non_scoring_selections_are_rejected() {
        assert_eq!(score(&[]), None);
        assert_eq!(score(&[2]), None);
        assert_eq!(score(&[6]), None);
        assert_eq!(score(&[1, 2]), None);
        assert_eq!(score(&[2, 3, 4, 6]), None);
        assert_eq!(score(&[1, 2, 3, 4]), None);
        assert_eq!(score(&[2, 2]), None);
        assert_eq!(score(&[2, 3, 4, 5, 6, 6]), None);
    }

    #[test]
    fn three_of_a_kind() {
        assert_eq!(score(&[1, 1, 1]), Some(1000));
        assert_eq!(score(&[2, 2, 2]), Some(200));
        assert_eq!(score(&[3, 3, 3]), Some(300));
        assert_eq!(score(&[4, 4, 4]), Some(400));
        assert_eq!(score(&[5, 5, 5]), Some(500));
        assert_eq!(score(&[6, 6, 6]), Some(600));
    }

    #[test]
    fn each_extra_identical_die_doubles() {
        assert_eq!(score(&[4, 4, 4, 4]), Some(800));
        assert_eq!(score(&[1, 1, 1, 1]), Some(2000));
        assert_eq!(score(&[5, 5, 5, 5, 5]), Some(2000));
        assert_eq!(score(&[3, 3, 3, 3, 3]), Some(1200));
        assert_eq!(score(&[2, 2, 2, 2, 2, 2]), Some(1600));
        assert_eq!(score(&[6, 6, 6, 6, 6, 6]), Some(4800));
        assert_eq!(score(&[1, 1, 1, 1, 1, 1]), Some(8000));
    }

    #[test]
    fn straights() {
        assert_eq!(score(&[1, 2, 3, 4, 5]), Some(500));
        assert_eq!(score(&[2, 3, 4, 5, 6]), Some(750));
        assert_eq!(score(&[1, 2, 3, 4, 5, 6]), Some(1500));
        assert_eq!(score(&[6, 4, 1, 3, 5, 2]), Some(1500));
        assert_eq!(score(&[5, 3, 1, 4, 2]), Some(500));
    }

    #[test]
    fn straight_plus_extra_scoring_die() {
        assert_eq!(score(&[1, 2, 3, 4, 5, 5]), Some(550));
        assert_eq!(score(&[1, 2, 3, 4, 5, 1]), Some(600));
        assert_eq!(score(&[2, 3, 4, 5, 6, 5]), Some(800));
        assert_eq!(score(&[2, 3, 4, 5, 6, 1]), Some(1500));
    }

    #[test]
    fn combinations_add_up() {
        assert_eq!(score(&[1, 1, 1, 5]), Some(1050));
        assert_eq!(score(&[2, 2, 2, 3, 3, 3]), Some(500));
        assert_eq!(score(&[1, 1, 1, 1, 5, 5]), Some(2100));
        assert_eq!(score(&[4, 4, 4, 4, 1, 5]), Some(950));
        assert_eq!(score(&[6, 6, 6, 1, 1, 5]), Some(850));
        assert_eq!(score(&[5, 5, 5, 5, 1]), Some(1100));
    }

    #[test]
    fn examples_from_rules_document() {
        assert_eq!(score(&[1, 5, 2, 3, 4, 6]), Some(1500));
        assert_eq!(score(&[1, 1, 1, 5]), Some(1050));
        assert_eq!(score(&[4, 4, 4, 4]), Some(800));
        assert_eq!(score(&[5, 5, 5, 5, 5]), Some(2000));
        assert_eq!(score(&[1, 1, 1, 1, 1, 1]), Some(8000));
        assert_eq!(score(&[2, 2, 2, 3, 3, 3]), Some(500));
        assert_eq!(score(&[1, 2, 3, 4, 5, 5]), Some(550));
        assert!(farkle(&[2, 3, 4, 6, 6, 3]));
    }

    #[test]
    fn partition_describes_the_patterns() {
        let (points, mut patterns) = best_partition(&dice_from(&[1, 1, 1, 5]).unwrap()).unwrap();
        patterns.sort();
        assert_eq!(points, 1050);
        assert_eq!(
            patterns,
            vec![Pattern::Single(5), Pattern::OfAKind { face: 1, count: 3 }]
        );
        let (_, patterns) = best_partition(&dice_from(&[6, 5, 4, 3, 2, 1]).unwrap()).unwrap();
        assert_eq!(patterns, vec![Pattern::FullStraight]);
    }

    #[test]
    fn pattern_points() {
        assert_eq!(Pattern::Single(1).points(), 100);
        assert_eq!(Pattern::Single(5).points(), 50);
        assert_eq!(Pattern::OfAKind { face: 1, count: 3 }.points(), 1000);
        assert_eq!(Pattern::OfAKind { face: 6, count: 4 }.points(), 1200);
        assert_eq!(Pattern::LowStraight.points(), 500);
        assert_eq!(Pattern::HighStraight.points(), 750);
        assert_eq!(Pattern::FullStraight.points(), 1500);
    }

    #[test]
    fn pattern_display_in_french() {
        assert_eq!(Pattern::Single(1).to_string(), "un 1");
        assert_eq!(
            Pattern::OfAKind { face: 4, count: 3 }.to_string(),
            "trois 4"
        );
        assert_eq!(Pattern::OfAKind { face: 2, count: 6 }.to_string(), "six 2");
        assert_eq!(Pattern::LowStraight.to_string(), "suite 1-5");
        assert_eq!(Pattern::HighStraight.to_string(), "suite 2-6");
        assert_eq!(Pattern::FullStraight.to_string(), "suite 1-6");
    }

    #[test]
    fn farkle_detection() {
        assert!(farkle(&[2, 3, 4, 6, 6, 3]));
        assert!(farkle(&[2, 2, 3, 3, 4, 6]));
        assert!(farkle(&[6]));
        assert!(farkle(&[2, 3]));
        assert!(!farkle(&[1]));
        assert!(!farkle(&[5, 2]));
        assert!(!farkle(&[2, 2, 2]));
        assert!(!farkle(&[2, 3, 4, 6, 6, 6]));
    }
}
