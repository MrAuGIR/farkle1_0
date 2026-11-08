//! Adversaire IA (le PNJ) : choisit ses actions grâce au conseiller.

use crate::advisor::{Decision, Risk, decide, suggest};
use crate::combos::indices_for;
use crate::dice::{DiceRoller, Die};
use crate::game::{Game, GameError, Phase, RollOutcome};

/// Action de l'IA pour l'étape en cours.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiAction {
    Roll,
    /// Positions (base 0) des dés à garder dans le dernier lancer.
    Keep(Vec<usize>),
    Bank,
}

/// Ce qui s'est passé pendant un tour de l'IA (pour l'affichage).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiEvent {
    Rolled(Vec<Die>),
    Kept { dice: Vec<Die>, points: u32 },
    Farkle { dice: Vec<Die>, lost_points: u32 },
    Banked { points: u32, won: bool },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiPlayer {
    pub risk: Risk,
}

impl Default for AiPlayer {
    fn default() -> Self {
        AiPlayer { risk: Risk::NORMAL }
    }
}

impl AiPlayer {
    pub fn new(risk: Risk) -> Self {
        AiPlayer { risk }
    }

    /// Prochaine action pour le joueur courant, `None` si la partie est finie.
    pub fn next_action(&self, game: &Game) -> Option<AiAction> {
        let score = game.players()[game.current()].score;
        match game.phase() {
            Phase::Start => Some(AiAction::Roll),
            Phase::Choosing => {
                let advice = suggest(
                    game.last_roll(),
                    game.turn_points(),
                    score,
                    game.target(),
                    self.risk,
                );
                let best = advice.best?;
                indices_for(game.last_roll(), &best.combo.dice).map(AiAction::Keep)
            }
            Phase::Kept => Some(
                match decide(
                    game.turn_points(),
                    game.dice_left(),
                    score,
                    game.target(),
                    self.risk,
                ) {
                    Decision::Reroll => AiAction::Roll,
                    Decision::Bank => AiAction::Bank,
                },
            ),
            Phase::GameOver { .. } => None,
        }
    }

    /// Joue une seule étape (pour un affichage pas à pas).
    pub fn step(
        &self,
        game: &mut Game,
        roller: &mut dyn DiceRoller,
    ) -> Result<Option<AiEvent>, GameError> {
        let Some(action) = self.next_action(game) else {
            return Ok(None);
        };
        let event = match action {
            AiAction::Roll => match game.roll(roller)? {
                RollOutcome::Scoring(dice) => AiEvent::Rolled(dice),
                RollOutcome::Farkle { dice, lost_points } => AiEvent::Farkle { dice, lost_points },
            },
            AiAction::Keep(indices) => {
                let dice = indices.iter().map(|&i| game.last_roll()[i]).collect();
                let points = game.keep(&indices)?;
                AiEvent::Kept { dice, points }
            }
            AiAction::Bank => {
                let out = game.bank()?;
                AiEvent::Banked {
                    points: out.banked,
                    won: out.won,
                }
            }
        };
        Ok(Some(event))
    }

    /// Joue tout le tour du joueur courant et renvoie les événements.
    pub fn play_turn(
        &self,
        game: &mut Game,
        roller: &mut dyn DiceRoller,
    ) -> Result<Vec<AiEvent>, GameError> {
        let mut events = Vec::new();
        while let Some(event) = self.step(game, roller)? {
            let turn_over = matches!(event, AiEvent::Farkle { .. } | AiEvent::Banked { .. });
            events.push(event);
            if turn_over {
                break;
            }
        }
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::{ScriptedRoller, dice_from};
    use crate::game::Phase;

    fn roller(rolls: &[&[u8]]) -> ScriptedRoller {
        ScriptedRoller::new(rolls.iter().map(|r| r.to_vec()).collect())
    }

    fn d(v: &[u8]) -> Vec<Die> {
        dice_from(v).unwrap()
    }

    #[test]
    fn rolls_at_start_of_turn() {
        let g = Game::new(&["Hans", "Henry"], 4000).unwrap();
        assert_eq!(AiPlayer::default().next_action(&g), Some(AiAction::Roll));
    }

    #[test]
    fn keeps_best_combo_positions() {
        let mut g = Game::new(&["Hans"], 4000).unwrap();
        g.roll(&mut roller(&[&[2, 1, 3, 1, 4, 1]])).unwrap();
        assert_eq!(
            AiPlayer::default().next_action(&g),
            Some(AiAction::Keep(vec![1, 3, 5]))
        );
    }

    #[test]
    fn banks_or_rerolls_after_keeping() {
        let ai = AiPlayer::default();
        let mut g = Game::new(&["Hans"], 4000).unwrap();
        g.roll(&mut roller(&[&[1, 1, 1, 2, 3, 4]])).unwrap();
        g.keep(&[0, 1, 2]).unwrap();
        assert_eq!(ai.next_action(&g), Some(AiAction::Bank));

        let mut g = Game::new(&["Hans"], 4000).unwrap();
        g.roll(&mut roller(&[&[1, 2, 3, 4, 6, 6]])).unwrap();
        g.keep(&[0]).unwrap();
        assert_eq!(ai.next_action(&g), Some(AiAction::Roll));
    }

    #[test]
    fn no_action_when_game_over() {
        let mut g = Game::new(&["Hans"], 100).unwrap();
        g.roll(&mut roller(&[&[1, 2, 3, 4, 6, 6]])).unwrap();
        g.keep(&[0]).unwrap();
        g.bank().unwrap();
        assert_eq!(g.phase(), Phase::GameOver { winner: 0 });
        assert_eq!(AiPlayer::default().next_action(&g), None);
        let mut r = roller(&[]);
        assert_eq!(AiPlayer::default().step(&mut g, &mut r), Ok(None));
    }

    #[test]
    fn step_plays_one_action_at_a_time() {
        let ai = AiPlayer::default();
        let mut g = Game::new(&["Hans", "Henry"], 4000).unwrap();
        let mut r = roller(&[&[1, 1, 1, 2, 3, 4]]);
        assert_eq!(
            ai.step(&mut g, &mut r),
            Ok(Some(AiEvent::Rolled(d(&[1, 1, 1, 2, 3, 4]))))
        );
        assert_eq!(
            ai.step(&mut g, &mut r),
            Ok(Some(AiEvent::Kept {
                dice: d(&[1, 1, 1]),
                points: 1000
            }))
        );
        assert_eq!(
            ai.step(&mut g, &mut r),
            Ok(Some(AiEvent::Banked {
                points: 1000,
                won: false
            }))
        );
        assert_eq!(g.current(), 1);
    }

    #[test]
    fn play_turn_until_bank() {
        let ai = AiPlayer::default();
        let mut g = Game::new(&["Hans", "Henry"], 4000).unwrap();
        let mut r = roller(&[&[1, 2, 3, 4, 6, 6], &[1, 1, 1, 2, 3]]);
        let events = ai.play_turn(&mut g, &mut r).unwrap();
        assert_eq!(
            events,
            vec![
                AiEvent::Rolled(d(&[1, 2, 3, 4, 6, 6])),
                AiEvent::Kept {
                    dice: d(&[1]),
                    points: 100
                },
                AiEvent::Rolled(d(&[1, 1, 1, 2, 3])),
                AiEvent::Kept {
                    dice: d(&[1, 1, 1]),
                    points: 1000
                },
                AiEvent::Banked {
                    points: 1100,
                    won: false
                },
            ]
        );
        assert_eq!(g.players()[0].score, 1100);
        assert_eq!(g.current(), 1);
    }

    #[test]
    fn play_turn_ends_on_farkle() {
        let ai = AiPlayer::default();
        let mut g = Game::new(&["Hans", "Henry"], 4000).unwrap();
        let mut r = roller(&[&[1, 2, 3, 4, 6, 6], &[2, 3, 4, 6, 6]]);
        let events = ai.play_turn(&mut g, &mut r).unwrap();
        assert_eq!(
            events.last(),
            Some(&AiEvent::Farkle {
                dice: d(&[2, 3, 4, 6, 6]),
                lost_points: 100
            })
        );
        assert_eq!(g.current(), 1);
        assert_eq!(g.players()[0].score, 0);
    }
}
