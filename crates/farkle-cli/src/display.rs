//! Mise en forme du texte affiché en console (fonctions pures).

use farkle_core::advisor::{Decision, KeepAdvice};
use farkle_core::combos::Combo;
use farkle_core::dice::Die;
use farkle_core::game::Game;

/// Valeurs séparées par des espaces : `1 1 5`.
pub fn dice_values(dice: &[Die]) -> String {
    let _ = dice;
    todo!()
}

/// Lancer sur deux lignes : les dés, puis leur numéro.
/// ```text
/// [5] [2] [6]
///  1   2   3
/// ```
pub fn roll(dice: &[Die]) -> String {
    let _ = dice;
    todo!()
}

/// `1 1 1 5 → 1050 pts (trois 1 + un 5)`
pub fn combo(c: &Combo) -> String {
    let _ = c;
    todo!()
}

/// Liste numérotée des combinaisons possibles.
pub fn combos(list: &[Combo]) -> String {
    let _ = list;
    todo!()
}

/// Conseil pour un lancer.
pub fn advice(a: &KeepAdvice, roll: &[Die]) -> String {
    let _ = (a, roll);
    todo!()
}

/// Conseil après une mise de côté.
pub fn decision(d: Decision, dice_left: usize, farkle_probability: f64) -> String {
    let _ = (d, dice_left, farkle_probability);
    todo!()
}

/// Tableau des scores : `Henry 1050 | Hans 300 (cible 4000)`.
pub fn scores(game: &Game) -> String {
    let _ = game;
    todo!()
}

/// Pourcentage à une décimale : `7.7 %`.
pub fn percent(p: f64) -> String {
    format!("{:.1} %", p * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use farkle_core::advisor::{Risk, suggest};
    use farkle_core::combos::find_combos;
    use farkle_core::dice::dice_from;

    fn d(v: &[u8]) -> Vec<Die> {
        dice_from(v).unwrap()
    }

    #[test]
    fn values() {
        assert_eq!(dice_values(&d(&[1, 1, 5])), "1 1 5");
        assert_eq!(dice_values(&[]), "");
    }

    #[test]
    fn roll_two_lines() {
        assert_eq!(roll(&d(&[5, 2, 6])), "[5] [2] [6]\n 1   2   3 ");
    }

    #[test]
    fn combo_line() {
        let best = &find_combos(&d(&[1, 1, 1, 5, 2, 3]))[0];
        assert_eq!(combo(best), "1 1 1 5 → 1050 pts (trois 1 + un 5)");
    }

    #[test]
    fn combos_list() {
        let text = combos(&find_combos(&d(&[1, 2, 3, 4, 6, 6])));
        assert_eq!(text, "  - 1 → 100 pts (un 1)");
        assert_eq!(combos(&[]), "  (aucune)");
    }

    #[test]
    fn advice_line() {
        let roll_ = d(&[2, 1, 3, 4, 6, 6]);
        let a = suggest(&roll_, 0, 0, 4000, Risk::NORMAL).best.unwrap();
        let text = advice(&a, &roll_);
        assert!(text.contains("garder 1 (dé n°2)"), "{text}");
        assert!(text.contains("100 pts"), "{text}");
        assert!(text.contains("relancer 5 dés"), "{text}");
        assert!(text.contains(&percent(a.farkle_probability)), "{text}");
    }

    #[test]
    fn decision_line() {
        assert_eq!(
            decision(Decision::Reroll, 4, 0.157),
            "Conseil : relancer les 4 dés (risque de Farkle 15.7 %)"
        );
        assert_eq!(
            decision(Decision::Bank, 1, 0.667),
            "Conseil : encaisser (risque de Farkle 66.7 % si vous relancez 1 dé)"
        );
    }

    #[test]
    fn scores_line() {
        let g = Game::new(&["Henry", "Hans"], 4000).unwrap();
        assert_eq!(scores(&g), "Henry 0 | Hans 0 (cible 4000)");
    }

    #[test]
    fn percent_format() {
        assert_eq!(percent(0.0309), "3.1 %");
    }
}
