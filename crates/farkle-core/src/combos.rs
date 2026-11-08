//! Recherche de toutes les mises de côté possibles dans un lancer (base de l'aide).

use crate::dice::Die;
use crate::scoring::{Pattern, best_partition};
use std::collections::HashSet;

/// Une mise de côté valide : les dés gardés, leurs points et leur découpage en motifs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Combo {
    /// Dés gardés, triés par valeur croissante.
    pub dice: Vec<Die>,
    pub score: u32,
    pub patterns: Vec<Pattern>,
}

/// Toutes les mises de côté valides (sans doublon), triées par score décroissant
/// puis par nombre de dés croissant. Vide si le lancer est un Farkle.
pub fn find_combos(roll: &[Die]) -> Vec<Combo> {
    let mut seen = HashSet::new();
    let mut combos = Vec::new();
    for mask in 1u32..(1 << roll.len()) {
        let mut dice: Vec<Die> = (0..roll.len())
            .filter(|i| mask & (1 << i) != 0)
            .map(|i| roll[i])
            .collect();
        dice.sort();
        if !seen.insert(dice.clone()) {
            continue;
        }
        if let Some((score, patterns)) = best_partition(&dice) {
            combos.push(Combo {
                dice,
                score,
                patterns,
            });
        }
    }
    combos.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(a.dice.len().cmp(&b.dice.len()))
            .then(a.dice.cmp(&b.dice))
    });
    combos
}

/// Positions (base 0, triées) dans `roll` des dés de `dice`, en prenant pour chaque
/// valeur les premières occurrences. `None` si `dice` n'est pas inclus dans `roll`.
pub fn indices_for(roll: &[Die], dice: &[Die]) -> Option<Vec<usize>> {
    let mut used = vec![false; roll.len()];
    let mut indices = Vec::with_capacity(dice.len());
    for die in dice {
        let i = (0..roll.len()).find(|&i| !used[i] && roll[i] == *die)?;
        used[i] = true;
        indices.push(i);
    }
    indices.sort_unstable();
    Some(indices)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::dice_from;

    fn combos(values: &[u8]) -> Vec<(Vec<u8>, u32)> {
        find_combos(&dice_from(values).unwrap())
            .into_iter()
            .map(|c| (c.dice.iter().map(|d| d.value()).collect(), c.score))
            .collect()
    }

    #[test]
    fn farkle_has_no_combo() {
        assert!(combos(&[2, 3, 4, 6, 6, 3]).is_empty());
    }

    #[test]
    fn full_straight_roll() {
        let c = combos(&[1, 5, 2, 3, 4, 6]);
        assert_eq!(
            c,
            vec![
                (vec![1, 2, 3, 4, 5, 6], 1500),
                (vec![2, 3, 4, 5, 6], 750),
                (vec![1, 2, 3, 4, 5], 500),
                (vec![1, 5], 150),
                (vec![1], 100),
                (vec![5], 50),
            ]
        );
    }

    #[test]
    fn no_duplicate_for_identical_dice() {
        assert_eq!(
            combos(&[1, 1, 2, 3, 4, 6]),
            vec![(vec![1, 1], 200), (vec![1], 100)]
        );
    }

    #[test]
    fn triple_with_single() {
        let c = combos(&[1, 1, 1, 5, 2, 3]);
        assert_eq!(c[0], (vec![1, 1, 1, 5], 1050));
        assert!(c.contains(&(vec![1, 1, 1], 1000)));
        assert!(c.contains(&(vec![1, 1, 5], 250)));
        assert!(c.contains(&(vec![5], 50)));
    }

    #[test]
    fn same_score_prefers_fewer_dice() {
        // [1] et [5, 5] valent tous deux 100 : le plus court d'abord.
        let c = combos(&[1, 5, 5, 2, 3, 6]);
        let pos_1 = c.iter().position(|x| x.0 == vec![1]).unwrap();
        let pos_55 = c.iter().position(|x| x.0 == vec![5, 5]).unwrap();
        assert!(pos_1 < pos_55, "{c:?}");
    }

    #[test]
    fn indices_for_maps_values_to_positions() {
        let roll = dice_from(&[5, 1, 5, 2]).unwrap();
        assert_eq!(
            indices_for(&roll, &dice_from(&[1, 5]).unwrap()),
            Some(vec![0, 1])
        );
        assert_eq!(
            indices_for(&roll, &dice_from(&[5, 5]).unwrap()),
            Some(vec![0, 2])
        );
        assert_eq!(indices_for(&roll, &dice_from(&[5, 5, 5]).unwrap()), None);
        assert_eq!(indices_for(&roll, &dice_from(&[6]).unwrap()), None);
        assert_eq!(indices_for(&roll, &[]), Some(vec![]));
    }

    #[test]
    fn combo_carries_patterns() {
        let best = &find_combos(&dice_from(&[4, 4, 4, 4, 1, 2]).unwrap())[0];
        assert_eq!(best.score, 900);
        assert_eq!(best.patterns.len(), 2);
    }
}
