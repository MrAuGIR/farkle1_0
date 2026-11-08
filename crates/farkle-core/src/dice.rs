//! Dés et lanceurs de dés (aléatoire déterministe ou scripté).

use std::collections::VecDeque;

/// Un dé à six faces (valeur garantie entre 1 et 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Die(u8);

impl Die {
    /// Crée un dé, `None` si la valeur n'est pas entre 1 et 6.
    pub fn new(value: u8) -> Option<Die> {
        (1..=6).contains(&value).then_some(Die(value))
    }

    pub fn value(self) -> u8 {
        self.0
    }
}

/// Convertit une liste de valeurs en dés, `None` si une valeur est invalide.
pub fn dice_from(values: &[u8]) -> Option<Vec<Die>> {
    values.iter().map(|&v| Die::new(v)).collect()
}

/// Source de lancers de dés.
pub trait DiceRoller {
    /// Lance `n` dés.
    fn roll(&mut self, n: usize) -> Vec<Die>;
}

/// Lanceur pseudo-aléatoire (SplitMix64), reproductible à partir d'une graine.
/// Sans dépendance externe : fonctionne aussi en WebAssembly.
#[derive(Debug, Clone)]
pub struct RandomRoller {
    state: u64,
}

impl RandomRoller {
    pub fn from_seed(seed: u64) -> Self {
        RandomRoller { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Tirage uniforme entre 1 et 6 (rejet pour éviter le biais du modulo).
    fn next_face(&mut self) -> u8 {
        const LIMIT: u64 = u64::MAX - (u64::MAX % 6);
        loop {
            let x = self.next_u64();
            if x < LIMIT {
                return (x % 6) as u8 + 1;
            }
        }
    }
}

impl DiceRoller for RandomRoller {
    fn roll(&mut self, n: usize) -> Vec<Die> {
        (0..n).map(|_| Die(self.next_face())).collect()
    }
}

/// Lanceur qui rejoue des lancers prédéfinis (tests, mode entraîner).
#[derive(Debug, Clone, Default)]
pub struct ScriptedRoller {
    rolls: VecDeque<Vec<Die>>,
}

impl ScriptedRoller {
    /// Panique si une valeur n'est pas entre 1 et 6.
    pub fn new(rolls: Vec<Vec<u8>>) -> Self {
        let rolls = rolls
            .into_iter()
            .map(|r| dice_from(&r).unwrap_or_else(|| panic!("valeur de dé invalide : {r:?}")))
            .collect();
        ScriptedRoller { rolls }
    }
}

impl DiceRoller for ScriptedRoller {
    /// Panique si le script est épuisé ou si le nombre de dés ne correspond pas.
    fn roll(&mut self, n: usize) -> Vec<Die> {
        let roll = self
            .rolls
            .pop_front()
            .expect("ScriptedRoller : plus de lancer prévu");
        assert_eq!(
            roll.len(),
            n,
            "ScriptedRoller : {n} dés attendus, lancer {roll:?}"
        );
        roll
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn die_accepts_only_1_to_6() {
        assert_eq!(Die::new(0), None);
        assert_eq!(Die::new(7), None);
        for v in 1..=6 {
            assert_eq!(Die::new(v).unwrap().value(), v);
        }
    }

    #[test]
    fn dice_from_validates_every_value() {
        let d = dice_from(&[1, 5, 6]).unwrap();
        assert_eq!(
            d.iter().map(|d| d.value()).collect::<Vec<_>>(),
            vec![1, 5, 6]
        );
        assert_eq!(dice_from(&[1, 9, 2]), None);
        assert_eq!(dice_from(&[]), Some(vec![]));
    }

    #[test]
    fn scripted_roller_replays_in_order() {
        let mut r = ScriptedRoller::new(vec![vec![1, 2, 3], vec![5, 5]]);
        assert_eq!(r.roll(3), dice_from(&[1, 2, 3]).unwrap());
        assert_eq!(r.roll(2), dice_from(&[5, 5]).unwrap());
    }

    #[test]
    #[should_panic]
    fn scripted_roller_panics_on_wrong_count() {
        let mut r = ScriptedRoller::new(vec![vec![1, 2, 3]]);
        r.roll(2);
    }

    #[test]
    #[should_panic]
    fn scripted_roller_panics_when_exhausted() {
        let mut r = ScriptedRoller::new(vec![]);
        r.roll(1);
    }

    #[test]
    fn random_roller_rolls_n_valid_dice() {
        let mut r = RandomRoller::from_seed(42);
        for n in 0..=6 {
            let roll = r.roll(n);
            assert_eq!(roll.len(), n);
            assert!(roll.iter().all(|d| (1..=6).contains(&d.value())));
        }
    }

    #[test]
    fn random_roller_is_deterministic_for_a_seed() {
        let mut a = RandomRoller::from_seed(7);
        let mut b = RandomRoller::from_seed(7);
        for _ in 0..20 {
            assert_eq!(a.roll(6), b.roll(6));
        }
    }

    #[test]
    fn random_roller_is_roughly_uniform() {
        let mut r = RandomRoller::from_seed(123);
        let mut counts = [0u32; 7];
        for d in r.roll(60_000) {
            counts[d.value() as usize] += 1;
        }
        for (face, &c) in counts.iter().enumerate().skip(1) {
            assert!((9_000..11_000).contains(&c), "face {face}: {c}");
        }
    }
}
