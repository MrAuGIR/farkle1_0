//! Conseiller : probabilités de Farkle, espérance d'une relance, meilleure mise
//! de côté et recommandation « relancer / encaisser ».
//!
//! Modèle « à un coup » : relancer `n` dés avec `t` points en jeu rapporte en
//! moyenne `(1 - p(n)) × t + M(n)`, où `p(n)` est la probabilité de Farkle et
//! `M(n)` l'espérance du meilleur score d'un lancer de `n` dés (0 en cas de Farkle).
//! Relancer est intéressant tant que `M(n) > p(n) × t`, soit `t < M(n) / p(n)`.
//! Ces valeurs sont calculées exactement, par énumération des 6^n lancers.

use crate::combos::{Combo, find_combos};
use crate::dice::Die;
use crate::game::DICE_COUNT;
use std::sync::OnceLock;

/// Recommandation après une mise de côté.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Reroll,
    Bank,
}

/// Goût du risque : multiplie le gain espéré d'une relance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Risk(pub f64);

impl Risk {
    pub const CAUTIOUS: Risk = Risk(0.7);
    pub const NORMAL: Risk = Risk(1.0);
    pub const BOLD: Risk = Risk(1.5);
}

/// Conseil pour la meilleure mise de côté d'un lancer.
#[derive(Debug, Clone, PartialEq)]
pub struct KeepAdvice {
    pub combo: Combo,
    /// Points du tour après cette mise de côté.
    pub turn_points_after: u32,
    /// Dés à relancer ensuite (6 en cas de main chaude).
    pub dice_left_after: usize,
    /// Probabilité de Farkle si l'on relance ensuite.
    pub farkle_probability: f64,
    /// Points du tour espérés si l'on relance une fois ensuite.
    pub expected_reroll: f64,
    pub decision: Decision,
}

/// Conseil complet pour un lancer.
#[derive(Debug, Clone, PartialEq)]
pub struct Advice {
    /// Toutes les mises de côté possibles (voir [`crate::combos::find_combos`]).
    pub combos: Vec<Combo>,
    /// Meilleure mise de côté, `None` si le lancer est un Farkle.
    pub best: Option<KeepAdvice>,
}

/// Probabilité de Farkle en lançant `n` dés (0 si `n` vaut 0 ou plus de 6).
pub fn farkle_probability(n: usize) -> f64 {
    stats(n).map_or(0.0, |s| s.farkle_probability)
}

/// Espérance du meilleur score d'un lancer de `n` dés (Farkle compté 0).
pub fn expected_best_score(n: usize) -> f64 {
    stats(n).map_or(0.0, |s| s.expected_best)
}

/// Points du tour espérés en relançant `n` dés une fois avec `turn_points` en jeu.
pub fn expected_reroll(turn_points: u32, n: usize) -> f64 {
    expected_reroll_with(turn_points, n, Risk::NORMAL)
}

/// Points du tour en dessous desquels relancer `n` dés est rentable.
pub fn reroll_threshold(n: usize) -> f64 {
    match farkle_probability(n) {
        p if p > 0.0 => expected_best_score(n) / p,
        _ => f64::INFINITY,
    }
}

/// Relancer ou encaisser, une fois les dés mis de côté.
/// Encaisse toujours si cela fait gagner la partie.
pub fn decide(turn_points: u32, dice_left: usize, score: u32, target: u32, risk: Risk) -> Decision {
    if score + turn_points >= target || stats(dice_left).is_none() {
        return Decision::Bank;
    }
    if expected_reroll_with(turn_points, dice_left, risk) > turn_points as f64 {
        Decision::Reroll
    } else {
        Decision::Bank
    }
}

/// Conseil pour un lancer : toutes les combinaisons, la meilleure mise de côté
/// (celle qui maximise les points espérés du tour) et la décision qui suit.
pub fn suggest(roll: &[Die], turn_points: u32, score: u32, target: u32, risk: Risk) -> Advice {
    let combos = find_combos(roll);
    let mut best: Option<(f64, KeepAdvice)> = None;
    for combo in &combos {
        let turn_points_after = turn_points + combo.score;
        let dice_left_after = match roll.len() - combo.dice.len() {
            0 => DICE_COUNT,
            n => n,
        };
        let decision = decide(turn_points_after, dice_left_after, score, target, risk);
        let expected = expected_reroll_with(turn_points_after, dice_left_after, risk);
        let value = if score + turn_points_after >= target {
            f64::INFINITY
        } else {
            expected.max(turn_points_after as f64)
        };
        // Les combinaisons arrivent par score décroissant : à valeur égale,
        // on garde la plus rentable immédiatement.
        if best.as_ref().is_none_or(|(v, _)| value > *v) {
            let advice = KeepAdvice {
                combo: combo.clone(),
                turn_points_after,
                dice_left_after,
                farkle_probability: farkle_probability(dice_left_after),
                expected_reroll: expected_reroll(turn_points_after, dice_left_after),
                decision,
            };
            best = Some((value, advice));
        }
    }
    Advice {
        combos,
        best: best.map(|(_, a)| a),
    }
}

fn expected_reroll_with(turn_points: u32, n: usize, risk: Risk) -> f64 {
    (1.0 - farkle_probability(n)) * turn_points as f64 + risk.0 * expected_best_score(n)
}

#[derive(Debug, Clone, Copy)]
struct RollStats {
    farkle_probability: f64,
    expected_best: f64,
}

/// Statistiques exactes pour 1 à 6 dés, calculées une fois pour toutes.
fn stats(n: usize) -> Option<RollStats> {
    static TABLE: OnceLock<Vec<RollStats>> = OnceLock::new();
    let table = TABLE.get_or_init(|| (0..=DICE_COUNT).map(compute_stats).collect());
    (1..=DICE_COUNT).contains(&n).then(|| table[n])
}

/// Énumère les lancers distincts (multi-ensembles) de `n` dés avec leur poids.
fn compute_stats(n: usize) -> RollStats {
    fn rec(face: u8, left: usize, current: &mut Vec<Die>, weight: f64, acc: &mut (f64, f64)) {
        if left == 0 {
            match find_combos(current).first() {
                Some(best) => acc.1 += weight * best.score as f64,
                None => acc.0 += weight,
            }
            return;
        }
        if face > 6 {
            return;
        }
        // On choisit combien de dés montrent `face` (k), poids multinomial 1/k!.
        let mut inv_fact = 1.0;
        for k in 0..=left {
            if k > 0 {
                inv_fact /= k as f64;
                current.push(Die::new(face).unwrap());
            }
            rec(face + 1, left - k, current, weight * inv_fact, acc);
        }
        // On a poussé exactement `left` dés pendant la boucle : on les retire.
        current.truncate(current.len() - left);
    }
    let mut acc = (0.0, 0.0);
    let n_fact: f64 = (1..=n).map(|i| i as f64).product();
    let total = 6f64.powi(n as i32);
    rec(1, n, &mut Vec::new(), n_fact / total, &mut acc);
    RollStats {
        farkle_probability: acc.0,
        expected_best: acc.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combos::find_combos;
    use crate::dice::dice_from;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn values(c: &Combo) -> Vec<u8> {
        c.dice.iter().map(|d| d.value()).collect()
    }

    #[test]
    fn exact_farkle_probabilities() {
        assert!(close(farkle_probability(1), 4.0 / 6.0));
        assert!(close(farkle_probability(2), 16.0 / 36.0));
        assert!(close(farkle_probability(3), 60.0 / 216.0));
        assert!(close(farkle_probability(4), 204.0 / 1296.0));
        assert!(close(farkle_probability(5), 600.0 / 7776.0));
        assert!(close(farkle_probability(6), 1440.0 / 46656.0));
        assert_eq!(farkle_probability(0), 0.0);
        assert_eq!(farkle_probability(7), 0.0);
    }

    #[test]
    fn expected_best_score_one_die() {
        // (100 + 50) / 6
        assert!(close(expected_best_score(1), 25.0));
        assert!(expected_best_score(6) > expected_best_score(5));
        assert!(expected_best_score(2) > expected_best_score(1));
    }

    #[test]
    fn expected_reroll_formula() {
        assert!(close(expected_reroll(0, 1), 25.0));
        let er = expected_reroll(500, 3);
        let manual = (1.0 - farkle_probability(3)) * 500.0 + expected_best_score(3);
        assert!(close(er, manual));
    }

    #[test]
    fn threshold_matches_break_even() {
        for n in 1..=6 {
            let t = reroll_threshold(n);
            assert!(close(
                expected_reroll(0, n) - farkle_probability(n) * t,
                0.0
            ));
        }
        assert!(
            reroll_threshold(1) < 50.0,
            "un seul dé : ne jamais relancer"
        );
        assert!(
            reroll_threshold(6) > 5000.0,
            "six dés : presque toujours relancer"
        );
    }

    #[test]
    fn decide_typical_cases() {
        let n = Risk::NORMAL;
        assert_eq!(decide(1000, 6, 0, 4000, n), Decision::Reroll);
        assert_eq!(decide(300, 1, 0, 4000, n), Decision::Bank);
        assert_eq!(decide(1000, 2, 0, 4000, n), Decision::Bank);
        assert_eq!(decide(50, 5, 0, 4000, n), Decision::Reroll);
    }

    #[test]
    fn decide_banks_when_it_wins() {
        assert_eq!(decide(600, 6, 3500, 4000, Risk::NORMAL), Decision::Bank);
        assert_eq!(decide(500, 6, 3500, 4000, Risk::BOLD), Decision::Bank);
    }

    #[test]
    fn risk_moves_the_threshold() {
        let t = reroll_threshold(3);
        let above = (t * 1.2) as u32;
        let below = (t * 0.8) as u32;
        assert_eq!(decide(above, 3, 0, 100_000, Risk::NORMAL), Decision::Bank);
        assert_eq!(decide(above, 3, 0, 100_000, Risk::BOLD), Decision::Reroll);
        assert_eq!(decide(below, 3, 0, 100_000, Risk::NORMAL), Decision::Reroll);
        assert_eq!(decide(below, 3, 0, 100_000, Risk::CAUTIOUS), Decision::Bank);
    }

    #[test]
    fn suggest_on_farkle() {
        let a = suggest(
            &dice_from(&[2, 3, 4, 6, 6, 3]).unwrap(),
            300,
            0,
            4000,
            Risk::NORMAL,
        );
        assert!(a.combos.is_empty());
        assert!(a.best.is_none());
    }

    #[test]
    fn suggest_lists_all_combos() {
        let roll = dice_from(&[1, 1, 1, 5, 2, 3]).unwrap();
        let a = suggest(&roll, 0, 0, 4000, Risk::NORMAL);
        assert_eq!(a.combos, find_combos(&roll));
    }

    #[test]
    fn suggest_prefers_keeping_more_dice_to_reroll() {
        // Garder seulement le 1 (100, 5 dés à relancer) vaut mieux que 1 5 5 (200, 3 dés).
        let a = suggest(
            &dice_from(&[1, 5, 5, 2, 3, 6]).unwrap(),
            0,
            0,
            4000,
            Risk::NORMAL,
        );
        let best = a.best.unwrap();
        assert_eq!(values(&best.combo), vec![1]);
        assert_eq!(best.turn_points_after, 100);
        assert_eq!(best.dice_left_after, 5);
        assert_eq!(best.decision, Decision::Reroll);
        assert!(close(best.farkle_probability, farkle_probability(5)));
        assert!(close(best.expected_reroll, expected_reroll(100, 5)));
    }

    #[test]
    fn suggest_full_straight_is_hot_dice() {
        let a = suggest(
            &dice_from(&[1, 2, 3, 4, 5, 6]).unwrap(),
            0,
            0,
            4000,
            Risk::NORMAL,
        );
        let best = a.best.unwrap();
        assert_eq!(best.combo.score, 1500);
        assert_eq!(best.dice_left_after, 6);
        assert_eq!(best.decision, Decision::Reroll);
    }

    #[test]
    fn suggest_banks_after_big_triple_with_few_dice() {
        let a = suggest(
            &dice_from(&[1, 1, 1, 2, 3, 4]).unwrap(),
            0,
            0,
            4000,
            Risk::NORMAL,
        );
        let best = a.best.unwrap();
        assert_eq!(values(&best.combo), vec![1, 1, 1]);
        assert_eq!(best.decision, Decision::Bank);
    }

    #[test]
    fn suggest_takes_the_win() {
        let a = suggest(
            &dice_from(&[5, 2, 3, 3, 6, 6]).unwrap(),
            0,
            3950,
            4000,
            Risk::NORMAL,
        );
        let best = a.best.unwrap();
        assert_eq!(values(&best.combo), vec![5]);
        assert_eq!(best.decision, Decision::Bank);
    }
}
