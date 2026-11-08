//! Moteur de partie : enchaînement des tours, Farkle, main chaude, victoire.
//!
//! Déroulement d'un tour :
//! `Start --roll--> Choosing --keep--> Kept --roll--> Choosing ...`
//! puis `Kept --bank-->` joueur suivant (ou fin de partie),
//! ou `roll` sans dé qui marque : Farkle, joueur suivant.

use crate::dice::{DiceRoller, Die};
use crate::scoring::{is_farkle, score_selection};
use std::fmt;

/// Nombre de dés en jeu.
pub const DICE_COUNT: usize = 6;
/// Score cible par défaut.
pub const DEFAULT_TARGET: u32 = 4000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub name: String,
    pub score: u32,
}

/// Étape du tour en cours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Début de tour : le joueur doit lancer les 6 dés.
    Start,
    /// Dés lancés : le joueur doit mettre de côté des dés qui marquent.
    Choosing,
    /// Dés mis de côté : le joueur relance ou encaisse.
    Kept,
    /// Partie terminée.
    GameOver { winner: usize },
}

/// Résultat d'un lancer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollOutcome {
    /// Au moins un dé marque : le joueur doit choisir quoi garder.
    Scoring(Vec<Die>),
    /// Aucun dé ne marque : points du tour perdus, la main passe.
    Farkle { dice: Vec<Die>, lost_points: u32 },
}

/// Résultat d'un encaissement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BankOutcome {
    pub banked: u32,
    pub new_score: u32,
    /// `true` si ce joueur atteint le score cible et gagne.
    pub won: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    NoPlayers,
    InvalidTarget,
    /// Action impossible à cette étape du tour.
    WrongPhase {
        phase: Phase,
    },
    /// La partie est finie.
    GameOver,
    EmptySelection,
    IndexOutOfRange {
        index: usize,
    },
    DuplicateIndex {
        index: usize,
    },
    /// Sélection contenant au moins un dé qui ne marque pas.
    NonScoringSelection,
    /// Lancer saisi avec un mauvais nombre de dés.
    WrongDiceCount {
        expected: usize,
        got: usize,
    },
    /// Scores de départ invalides (un par joueur, inférieurs au score cible).
    InvalidScores,
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::NoPlayers => write!(f, "il faut au moins un joueur"),
            GameError::InvalidTarget => write!(f, "le score cible doit être supérieur à 0"),
            GameError::WrongPhase { phase } => {
                let hint = match phase {
                    Phase::Start => "il faut d'abord lancer les dés",
                    Phase::Choosing => "il faut d'abord mettre de côté des dés qui marquent",
                    Phase::Kept => "il faut relancer ou encaisser",
                    Phase::GameOver { .. } => "la partie est terminée",
                };
                write!(f, "action impossible : {hint}")
            }
            GameError::GameOver => write!(f, "la partie est terminée"),
            GameError::EmptySelection => write!(f, "aucun dé sélectionné"),
            GameError::IndexOutOfRange { index } => write!(f, "le dé n°{} n'existe pas", index + 1),
            GameError::DuplicateIndex { index } => {
                write!(f, "le dé n°{} est sélectionné deux fois", index + 1)
            }
            GameError::NonScoringSelection => {
                write!(f, "la sélection contient un dé qui ne marque pas")
            }
            GameError::WrongDiceCount { expected, got } => {
                write!(f, "{expected} dés attendus, {got} saisis")
            }
            GameError::InvalidScores => write!(
                f,
                "scores de départ invalides (un par joueur, inférieurs au score cible)"
            ),
        }
    }
}

impl std::error::Error for GameError {}

#[derive(Debug, Clone)]
pub struct Game {
    players: Vec<Player>,
    target: u32,
    current: usize,
    phase: Phase,
    turn_points: u32,
    dice_left: usize,
    last_roll: Vec<Die>,
}

impl Game {
    /// Nouvelle partie ; le premier joueur commence.
    pub fn new(names: &[&str], target: u32) -> Result<Game, GameError> {
        if names.is_empty() {
            return Err(GameError::NoPlayers);
        }
        if target == 0 {
            return Err(GameError::InvalidTarget);
        }
        Ok(Game {
            players: names
                .iter()
                .map(|n| Player {
                    name: n.to_string(),
                    score: 0,
                })
                .collect(),
            target,
            current: 0,
            phase: Phase::Start,
            turn_points: 0,
            dice_left: DICE_COUNT,
            last_roll: Vec::new(),
        })
    }

    /// Fixe les scores de départ (pour reprendre une situation en cours de partie).
    pub fn with_scores(self, scores: &[u32]) -> Result<Game, GameError> {
        let _ = scores;
        todo!()
    }

    pub fn players(&self) -> &[Player] {
        &self.players
    }
    pub fn target(&self) -> u32 {
        self.target
    }
    pub fn current(&self) -> usize {
        self.current
    }
    pub fn phase(&self) -> Phase {
        self.phase
    }
    /// Points accumulés pendant le tour en cours (pas encore encaissés).
    pub fn turn_points(&self) -> u32 {
        self.turn_points
    }
    /// Nombre de dés à lancer au prochain lancer.
    pub fn dice_left(&self) -> usize {
        self.dice_left
    }
    /// Dernier lancer (vide avant le premier lancer du tour).
    pub fn last_roll(&self) -> &[Die] {
        &self.last_roll
    }
    pub fn winner(&self) -> Option<usize> {
        match self.phase {
            Phase::GameOver { winner } => Some(winner),
            _ => None,
        }
    }
    /// Main chaude : les 6 dés ont marqué, on peut relancer les 6.
    pub fn is_hot_dice(&self) -> bool {
        self.phase == Phase::Kept && self.dice_left == DICE_COUNT
    }

    /// Lance les dés restants avec `roller`.
    pub fn roll(&mut self, roller: &mut dyn DiceRoller) -> Result<RollOutcome, GameError> {
        self.ensure_can_roll()?;
        let dice = roller.roll(self.dice_left);
        Ok(self.apply_roll(dice))
    }

    /// Joue un lancer fourni (saisie manuelle du mode entraîner).
    pub fn submit_roll(&mut self, dice: Vec<Die>) -> Result<RollOutcome, GameError> {
        self.ensure_can_roll()?;
        if dice.len() != self.dice_left {
            return Err(GameError::WrongDiceCount {
                expected: self.dice_left,
                got: dice.len(),
            });
        }
        Ok(self.apply_roll(dice))
    }

    /// Met de côté les dés du dernier lancer aux positions `indices` (base 0).
    /// Retourne les points gagnés. L'état est inchangé en cas d'erreur.
    pub fn keep(&mut self, indices: &[usize]) -> Result<u32, GameError> {
        self.ensure_phase(Phase::Choosing)?;
        if indices.is_empty() {
            return Err(GameError::EmptySelection);
        }
        let mut used = [false; DICE_COUNT];
        let mut selection = Vec::with_capacity(indices.len());
        for &index in indices {
            if index >= self.last_roll.len() {
                return Err(GameError::IndexOutOfRange { index });
            }
            if std::mem::replace(&mut used[index], true) {
                return Err(GameError::DuplicateIndex { index });
            }
            selection.push(self.last_roll[index]);
        }
        let points = score_selection(&selection).ok_or(GameError::NonScoringSelection)?;
        self.turn_points += points;
        self.dice_left -= selection.len();
        if self.dice_left == 0 {
            self.dice_left = DICE_COUNT; // main chaude
        }
        self.phase = Phase::Kept;
        Ok(points)
    }

    /// Encaisse les points du tour et passe la main (ou termine la partie).
    pub fn bank(&mut self) -> Result<BankOutcome, GameError> {
        self.ensure_phase(Phase::Kept)?;
        let banked = self.turn_points;
        let player = &mut self.players[self.current];
        player.score += banked;
        let new_score = player.score;
        let won = new_score >= self.target;
        if won {
            self.phase = Phase::GameOver {
                winner: self.current,
            };
        } else {
            self.next_player();
        }
        Ok(BankOutcome {
            banked,
            new_score,
            won,
        })
    }

    fn ensure_phase(&self, expected: Phase) -> Result<(), GameError> {
        match self.phase {
            Phase::GameOver { .. } => Err(GameError::GameOver),
            p if p == expected => Ok(()),
            phase => Err(GameError::WrongPhase { phase }),
        }
    }

    fn ensure_can_roll(&self) -> Result<(), GameError> {
        match self.phase {
            Phase::Start | Phase::Kept => Ok(()),
            _ => self.ensure_phase(Phase::Start),
        }
    }

    fn apply_roll(&mut self, dice: Vec<Die>) -> RollOutcome {
        if is_farkle(&dice) {
            let lost_points = self.turn_points;
            self.next_player();
            RollOutcome::Farkle { dice, lost_points }
        } else {
            self.last_roll = dice.clone();
            self.phase = Phase::Choosing;
            RollOutcome::Scoring(dice)
        }
    }

    fn next_player(&mut self) {
        self.current = (self.current + 1) % self.players.len();
        self.phase = Phase::Start;
        self.turn_points = 0;
        self.dice_left = DICE_COUNT;
        self.last_roll.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::{ScriptedRoller, dice_from};

    fn game() -> Game {
        Game::new(&["Henry", "Hans"], 4000).unwrap()
    }

    fn roller(rolls: &[&[u8]]) -> ScriptedRoller {
        ScriptedRoller::new(rolls.iter().map(|r| r.to_vec()).collect())
    }

    #[test]
    fn new_game_initial_state() {
        let g = game();
        assert_eq!(g.players().len(), 2);
        assert_eq!(g.players()[0].name, "Henry");
        assert!(g.players().iter().all(|p| p.score == 0));
        assert_eq!(g.target(), 4000);
        assert_eq!(g.current(), 0);
        assert_eq!(g.phase(), Phase::Start);
        assert_eq!(g.turn_points(), 0);
        assert_eq!(g.dice_left(), 6);
        assert!(g.last_roll().is_empty());
        assert_eq!(g.winner(), None);
        assert!(!g.is_hot_dice());
    }

    #[test]
    fn new_game_rejects_bad_config() {
        assert_eq!(Game::new(&[], 4000).unwrap_err(), GameError::NoPlayers);
        assert_eq!(Game::new(&["A"], 0).unwrap_err(), GameError::InvalidTarget);
    }

    #[test]
    fn cannot_keep_or_bank_before_rolling() {
        let mut g = game();
        assert_eq!(
            g.keep(&[0]),
            Err(GameError::WrongPhase {
                phase: Phase::Start
            })
        );
        assert_eq!(
            g.bank(),
            Err(GameError::WrongPhase {
                phase: Phase::Start
            })
        );
    }

    #[test]
    fn roll_moves_to_choosing() {
        let mut g = game();
        let out = g.roll(&mut roller(&[&[1, 2, 3, 4, 6, 6]])).unwrap();
        assert_eq!(
            out,
            RollOutcome::Scoring(dice_from(&[1, 2, 3, 4, 6, 6]).unwrap())
        );
        assert_eq!(g.phase(), Phase::Choosing);
        assert_eq!(
            g.last_roll(),
            dice_from(&[1, 2, 3, 4, 6, 6]).unwrap().as_slice()
        );
    }

    #[test]
    fn cannot_roll_or_bank_while_choosing() {
        let mut g = game();
        g.roll(&mut roller(&[&[1, 2, 3, 4, 6, 6]])).unwrap();
        let wrong = GameError::WrongPhase {
            phase: Phase::Choosing,
        };
        assert_eq!(
            g.roll(&mut roller(&[&[1, 1, 1, 1, 1, 1]])).unwrap_err(),
            wrong
        );
        assert_eq!(g.bank().unwrap_err(), wrong);
    }

    #[test]
    fn keep_adds_points_and_removes_dice() {
        let mut g = game();
        g.roll(&mut roller(&[&[1, 5, 2, 3, 6, 6]])).unwrap();
        assert_eq!(g.keep(&[0, 1]), Ok(150));
        assert_eq!(g.turn_points(), 150);
        assert_eq!(g.dice_left(), 4);
        assert_eq!(g.phase(), Phase::Kept);
    }

    #[test]
    fn keep_rejects_invalid_selections_without_changing_state() {
        let mut g = game();
        g.roll(&mut roller(&[&[1, 5, 2, 3, 6, 6]])).unwrap();
        assert_eq!(g.keep(&[]), Err(GameError::EmptySelection));
        assert_eq!(g.keep(&[6]), Err(GameError::IndexOutOfRange { index: 6 }));
        assert_eq!(g.keep(&[0, 0]), Err(GameError::DuplicateIndex { index: 0 }));
        assert_eq!(g.keep(&[0, 2]), Err(GameError::NonScoringSelection));
        assert_eq!(g.phase(), Phase::Choosing);
        assert_eq!(g.turn_points(), 0);
        assert_eq!(g.dice_left(), 6);
    }

    #[test]
    fn cannot_keep_twice_for_one_roll() {
        let mut g = game();
        g.roll(&mut roller(&[&[1, 5, 2, 3, 6, 6]])).unwrap();
        g.keep(&[0]).unwrap();
        assert_eq!(
            g.keep(&[1]),
            Err(GameError::WrongPhase { phase: Phase::Kept })
        );
    }

    #[test]
    fn reroll_uses_remaining_dice_and_accumulates() {
        let mut g = game();
        let mut r = roller(&[&[1, 5, 2, 3, 6, 6], &[5, 2, 3, 4]]);
        g.roll(&mut r).unwrap();
        g.keep(&[0, 1]).unwrap();
        g.roll(&mut r).unwrap(); // ScriptedRoller vérifie qu'on lance bien 4 dés
        assert_eq!(g.keep(&[0]), Ok(50));
        assert_eq!(g.turn_points(), 200);
        assert_eq!(g.dice_left(), 3);
    }

    #[test]
    fn farkle_loses_turn_points_and_passes() {
        let mut g = game();
        let mut r = roller(&[&[1, 5, 2, 3, 6, 6], &[2, 3, 4, 6]]);
        g.roll(&mut r).unwrap();
        g.keep(&[0, 1]).unwrap();
        let out = g.roll(&mut r).unwrap();
        assert_eq!(
            out,
            RollOutcome::Farkle {
                dice: dice_from(&[2, 3, 4, 6]).unwrap(),
                lost_points: 150
            }
        );
        assert_eq!(g.current(), 1);
        assert_eq!(g.phase(), Phase::Start);
        assert_eq!(g.turn_points(), 0);
        assert_eq!(g.dice_left(), 6);
        assert_eq!(g.players()[0].score, 0);
    }

    #[test]
    fn farkle_on_first_roll() {
        let mut g = game();
        let out = g.roll(&mut roller(&[&[2, 2, 3, 3, 4, 6]])).unwrap();
        assert!(matches!(out, RollOutcome::Farkle { lost_points: 0, .. }));
        assert_eq!(g.current(), 1);
    }

    #[test]
    fn hot_dice_in_one_keep() {
        let mut g = game();
        let mut r = roller(&[&[1, 2, 3, 4, 5, 6], &[1, 2, 3, 4, 6, 6]]);
        g.roll(&mut r).unwrap();
        assert_eq!(g.keep(&[0, 1, 2, 3, 4, 5]), Ok(1500));
        assert!(g.is_hot_dice());
        assert_eq!(g.dice_left(), 6);
        g.roll(&mut r).unwrap(); // 6 dés relancés
        assert!(!g.is_hot_dice());
        g.keep(&[0]).unwrap();
        assert_eq!(g.turn_points(), 1600);
    }

    #[test]
    fn hot_dice_over_several_keeps() {
        let mut g = game();
        let mut r = roller(&[&[2, 2, 2, 3, 4, 6], &[1, 5, 5]]);
        g.roll(&mut r).unwrap();
        g.keep(&[0, 1, 2]).unwrap();
        assert!(!g.is_hot_dice());
        g.roll(&mut r).unwrap();
        g.keep(&[0, 1, 2]).unwrap();
        assert!(g.is_hot_dice());
        assert_eq!(g.dice_left(), 6);
        assert_eq!(g.turn_points(), 400);
    }

    #[test]
    fn bank_adds_score_and_passes() {
        let mut g = game();
        g.roll(&mut roller(&[&[1, 1, 1, 2, 3, 4]])).unwrap();
        g.keep(&[0, 1, 2]).unwrap();
        assert_eq!(
            g.bank(),
            Ok(BankOutcome {
                banked: 1000,
                new_score: 1000,
                won: false
            })
        );
        assert_eq!(g.players()[0].score, 1000);
        assert_eq!(g.current(), 1);
        assert_eq!(g.phase(), Phase::Start);
        assert_eq!(g.turn_points(), 0);
        assert_eq!(g.dice_left(), 6);
        assert!(g.last_roll().is_empty());
    }

    #[test]
    fn turn_order_wraps_around() {
        let mut g = game();
        let mut r = roller(&[&[2, 2, 3, 3, 4, 6], &[2, 2, 3, 3, 4, 6]]);
        g.roll(&mut r).unwrap();
        g.roll(&mut r).unwrap();
        assert_eq!(g.current(), 0);
    }

    #[test]
    fn reaching_target_exactly_wins() {
        let mut g = Game::new(&["Henry", "Hans"], 1000).unwrap();
        g.roll(&mut roller(&[&[1, 1, 1, 2, 3, 4]])).unwrap();
        g.keep(&[0, 1, 2]).unwrap();
        assert_eq!(
            g.bank(),
            Ok(BankOutcome {
                banked: 1000,
                new_score: 1000,
                won: true
            })
        );
        assert_eq!(g.phase(), Phase::GameOver { winner: 0 });
        assert_eq!(g.winner(), Some(0));
    }

    #[test]
    fn no_action_after_game_over() {
        let mut g = Game::new(&["Henry"], 500).unwrap();
        g.roll(&mut roller(&[&[5, 5, 5, 2, 3, 4]])).unwrap();
        g.keep(&[0, 1, 2]).unwrap();
        assert!(g.bank().unwrap().won);
        assert_eq!(
            g.roll(&mut roller(&[&[1, 1, 1, 1, 1, 1]])).unwrap_err(),
            GameError::GameOver
        );
        assert_eq!(g.keep(&[0]), Err(GameError::GameOver));
        assert_eq!(g.bank(), Err(GameError::GameOver));
    }

    #[test]
    fn submit_roll_checks_dice_count() {
        let mut g = game();
        let err = g.submit_roll(dice_from(&[1, 2, 3]).unwrap()).unwrap_err();
        assert_eq!(
            err,
            GameError::WrongDiceCount {
                expected: 6,
                got: 3
            }
        );
        assert_eq!(g.phase(), Phase::Start);
        let out = g
            .submit_roll(dice_from(&[1, 2, 3, 4, 6, 6]).unwrap())
            .unwrap();
        assert!(matches!(out, RollOutcome::Scoring(_)));
    }

    #[test]
    fn starting_scores() {
        let g = game().with_scores(&[3500, 1200]).unwrap();
        assert_eq!(g.players()[0].score, 3500);
        assert_eq!(g.players()[1].score, 1200);
        assert_eq!(
            game().with_scores(&[100]).unwrap_err(),
            GameError::InvalidScores
        );
        assert_eq!(
            game().with_scores(&[4000, 0]).unwrap_err(),
            GameError::InvalidScores
        );
    }

    #[test]
    fn errors_have_french_messages() {
        assert!(
            GameError::NonScoringSelection
                .to_string()
                .contains("marque")
        );
        assert!(
            GameError::WrongDiceCount {
                expected: 6,
                got: 3
            }
            .to_string()
            .contains('6')
        );
        assert!(!GameError::GameOver.to_string().is_empty());
    }
}
