//! Mise en forme du texte affiché en console (fonctions pures).

use farkle_core::advisor::{Decision, KeepAdvice};
use farkle_core::combos::{Combo, indices_for};
use farkle_core::dice::Die;
use farkle_core::game::Game;
use farkle_core::scoring::Pattern;

/// Valeurs séparées par des espaces : `1 1 5`.
pub fn dice_values(dice: &[Die]) -> String {
    join(dice.iter().map(|d| d.value().to_string()), " ")
}

/// Lancer sur deux lignes : les dés, puis leur numéro.
/// ```text
/// [5] [2] [6]
///  1   2   3
/// ```
pub fn roll(dice: &[Die]) -> String {
    let faces = join(dice.iter().map(|d| format!("[{}]", d.value())), " ");
    let numbers = join((1..=dice.len()).map(|i| format!(" {i} ")), " ");
    format!("{faces}\n{numbers}")
}

/// `1 1 1 5 → 1050 pts (trois 1 + un 5)`
pub fn combo(c: &Combo) -> String {
    format!(
        "{} → {} pts ({})",
        dice_values(&c.dice),
        c.score,
        patterns(&c.patterns)
    )
}

/// Motifs séparés par « + », les dés isolés identiques regroupés : `un 1 + deux 5`.
pub fn patterns(list: &[Pattern]) -> String {
    let mut parts = Vec::new();
    let mut singles = [0usize; 7];
    for p in list {
        match p {
            Pattern::Single(face) => singles[*face as usize] += 1,
            other => parts.push(other.to_string()),
        }
    }
    for face in [1usize, 5] {
        match singles[face] {
            0 => {}
            1 => parts.push(format!("un {face}")),
            n => parts.push(format!(
                "{} {face}",
                ["deux", "trois", "quatre"][(n - 2).min(2)]
            )),
        }
    }
    parts.join(" + ")
}

/// Liste numérotée des combinaisons possibles.
pub fn combos(list: &[Combo]) -> String {
    if list.is_empty() {
        return "  (aucune)".into();
    }
    join(list.iter().map(|c| format!("  - {}", combo(c))), "\n")
}

/// Conseil pour un lancer.
pub fn advice(a: &KeepAdvice, roll: &[Die]) -> String {
    let positions = indices_for(roll, &a.combo.dice).unwrap_or_default();
    let label = if positions.len() > 1 {
        "dés n°"
    } else {
        "dé n°"
    };
    let positions = join(positions.iter().map(|i| (i + 1).to_string()), " ");
    let next = match a.decision {
        Decision::Reroll => format!(
            "relancer {} (risque de Farkle {})",
            dice_count(a.dice_left_after),
            percent(a.farkle_probability)
        ),
        Decision::Bank => "encaisser".into(),
    };
    format!(
        "Conseil : garder {} ({label}{positions}) → {} pts au tour, puis {next}",
        dice_values(&a.combo.dice),
        a.turn_points_after
    )
}

/// Conseil après une mise de côté.
pub fn decision(d: Decision, dice_left: usize, farkle_probability: f64) -> String {
    let p = percent(farkle_probability);
    match d {
        Decision::Reroll if dice_left == 1 => {
            format!("Conseil : relancer le dé (risque de Farkle {p})")
        }
        Decision::Reroll => {
            format!("Conseil : relancer les {dice_left} dés (risque de Farkle {p})")
        }
        Decision::Bank => format!(
            "Conseil : encaisser (risque de Farkle {p} si vous relancez {})",
            dice_count(dice_left)
        ),
    }
}

/// Tableau des scores : `Henry 1050 | Hans 300 (cible 4000)`.
pub fn scores(game: &Game) -> String {
    let players = join(
        game.players()
            .iter()
            .map(|p| format!("{} {}", p.name, p.score)),
        " | ",
    );
    format!("{players} (cible {})", game.target())
}

/// `1 dé`, `5 dés`.
pub fn dice_count(n: usize) -> String {
    if n == 1 {
        "1 dé".into()
    } else {
        format!("{n} dés")
    }
}

fn join(items: impl Iterator<Item = String>, sep: &str) -> String {
    items.collect::<Vec<_>>().join(sep)
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
    fn identical_singles_are_grouped() {
        let c = &find_combos(&d(&[1, 5, 5, 3, 2, 6]))[0];
        assert_eq!(combo(c), "1 5 5 → 200 pts (un 1 + deux 5)");
        let c = &find_combos(&d(&[1, 1, 2, 3, 4, 6]))[0];
        assert_eq!(combo(c), "1 1 → 200 pts (deux 1)");
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
