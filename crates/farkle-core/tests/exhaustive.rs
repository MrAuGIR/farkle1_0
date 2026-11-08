//! Tests d'intégration : propriétés vérifiées sur les 46 656 lancers de 6 dés.

use farkle_core::combos::find_combos;
use farkle_core::dice::{Die, dice_from};
use farkle_core::scoring::{is_farkle, score_selection};

fn all_rolls(n: usize) -> Vec<Vec<Die>> {
    let mut out = vec![vec![]];
    for _ in 0..n {
        out = out
            .into_iter()
            .flat_map(|r| {
                (1..=6).map(move |v| {
                    let mut r2 = r.clone();
                    r2.push(Die::new(v).unwrap());
                    r2
                })
            })
            .collect();
    }
    out
}

#[test]
fn farkle_iff_no_combo() {
    for roll in all_rolls(6) {
        assert_eq!(is_farkle(&roll), find_combos(&roll).is_empty(), "{roll:?}");
    }
}

#[test]
fn every_combo_is_a_valid_scoring_selection() {
    for roll in all_rolls(6) {
        for c in find_combos(&roll) {
            assert_eq!(score_selection(&c.dice), Some(c.score));
            assert!(c.score > 0 && c.score % 50 == 0);
            assert_eq!(c.patterns.iter().map(|p| p.points()).sum::<u32>(), c.score);
        }
    }
}

#[test]
fn farkle_probability_with_six_dice_is_about_3_1_percent() {
    let farkles = all_rolls(6).iter().filter(|r| is_farkle(r)).count();
    // Sans 1, sans 5 et sans brelan : faces {2,3,4,6}, chacune au plus 2 fois.
    // Répartitions (2,2,2,0) : 4 × 90 = 360 ; (2,2,1,1) : 6 × 180 = 1080.
    // Total 1440 / 46656 ≈ 3,09 % (pas de combinaison « trois paires » en KCD).
    assert_eq!(farkles, 1440);
}

#[test]
fn farkle_probability_with_one_die_is_two_thirds() {
    let farkles = all_rolls(1).iter().filter(|r| is_farkle(r)).count();
    assert_eq!(farkles, 4);
    assert!(!is_farkle(&dice_from(&[1]).unwrap()));
}
