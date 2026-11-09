//! Façade WebAssembly du moteur Farkle.
//!
//! `WebGame` oppose le joueur humain (indice 0) à l'IA (indice 1). Chaque action
//! renvoie l'état de la partie sérialisé en JSON (voir [`view::GameView`]) ou un
//! message d'erreur en français. Le code est du Rust ordinaire : il se teste en
//! natif, la couche `wasm_bindgen` ne fait que l'exposer à JavaScript.

pub mod view;

use farkle_core::ai::{AiEvent, AiPlayer};
use farkle_core::dice::{DiceRoller, RandomRoller, dice_from};
use farkle_core::game::{DICE_COUNT, Game, GameError, Phase, RollOutcome};
use view::{EventView, GameView, HUMAN, combo_views, values};
use wasm_bindgen::prelude::*;

/// Nom affiché du joueur humain.
pub const HUMAN_NAME: &str = "Vous";
/// Nom affiché de l'IA.
pub const AI_NAME: &str = "Hans";

/// Partie humain contre IA pilotée depuis JavaScript.
#[wasm_bindgen]
pub struct WebGame {
    game: Game,
    roller: Box<dyn DiceRoller>,
    ai: AiPlayer,
    last_event: Option<EventView>,
}

#[wasm_bindgen]
impl WebGame {
    /// Nouvelle partie avec un lanceur aléatoire initialisé par `seed`.
    #[wasm_bindgen(constructor)]
    pub fn new(target: u32, seed: u32) -> Result<WebGame, String> {
        // Étale la graine 32 bits sur 64 bits pour le générateur.
        let seed = u64::from(seed).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        WebGame::with_roller(target, Box::new(RandomRoller::from_seed(seed)))
    }

    /// État courant en JSON (voir [`view::GameView`]).
    pub fn state(&self) -> String {
        let view = GameView::new(&self.game, self.last_event.clone());
        serde_json::to_string(&view).expect("état sérialisable")
    }

    /// Lance les dés restants (début de tour ou relance). Renvoie le nouvel état.
    pub fn roll(&mut self) -> Result<String, String> {
        self.ensure_human_turn()?;
        let player = self.game.current();
        let outcome = self.game.roll(self.roller.as_mut()).map_err(err)?;
        self.last_event = Some(roll_event(player, outcome));
        Ok(self.state())
    }

    /// Met de côté les dés aux positions `indices` (base 0) du dernier lancer.
    pub fn select(&mut self, indices: Vec<u32>) -> Result<String, String> {
        self.ensure_human_turn()?;
        let indices: Vec<usize> = indices.into_iter().map(|i| i as usize).collect();
        let dice = indices
            .iter()
            .filter_map(|&i| self.game.last_roll().get(i).map(|d| d.value()))
            .collect();
        let points = self.game.keep(&indices).map_err(err)?;
        self.last_event = Some(EventView::Kept {
            player: HUMAN,
            dice,
            points,
        });
        Ok(self.state())
    }

    /// Relance les dés restants après une mise de côté.
    pub fn reroll(&mut self) -> Result<String, String> {
        self.ensure_human_turn()?;
        if self.game.phase() != Phase::Kept {
            return Err(err(GameError::WrongPhase {
                phase: self.game.phase(),
            }));
        }
        self.roll()
    }

    /// Encaisse les points du tour.
    pub fn bank(&mut self) -> Result<String, String> {
        self.ensure_human_turn()?;
        let out = self.game.bank().map_err(err)?;
        self.last_event = Some(EventView::Banked {
            player: HUMAN,
            points: out.banked,
            new_score: out.new_score,
            won: out.won,
        });
        Ok(self.state())
    }

    /// Joue une étape du tour de l'IA. Erreur si ce n'est pas son tour.
    pub fn ai_turn_step(&mut self) -> Result<String, String> {
        if let Phase::GameOver { .. } = self.game.phase() {
            return Err(err(GameError::GameOver));
        }
        let player = self.game.current();
        if player == HUMAN {
            return Err(format!("ce n'est pas le tour de {AI_NAME}"));
        }
        let event = self
            .ai
            .step(&mut self.game, self.roller.as_mut())
            .map_err(err)?
            .ok_or_else(|| err(GameError::GameOver))?;
        self.last_event = Some(match event {
            AiEvent::Rolled(dice) => EventView::Rolled {
                player,
                dice: values(&dice),
            },
            AiEvent::Kept { dice, points } => EventView::Kept {
                player,
                dice: values(&dice),
                points,
            },
            AiEvent::Farkle { dice, lost_points } => EventView::Farkle {
                player,
                dice: values(&dice),
                lost_points,
            },
            AiEvent::Banked { points, won } => EventView::Banked {
                player,
                points,
                new_score: self.game.players()[player].score,
                won,
            },
        });
        Ok(self.state())
    }
}

impl WebGame {
    /// Nouvelle partie avec un lanceur fourni (tests natifs).
    pub fn with_roller(target: u32, roller: Box<dyn DiceRoller>) -> Result<WebGame, String> {
        let game = Game::new(&[HUMAN_NAME, AI_NAME], target).map_err(err)?;
        Ok(WebGame {
            game,
            roller,
            ai: AiPlayer::default(),
            last_event: None,
        })
    }

    fn ensure_human_turn(&self) -> Result<(), String> {
        match self.game.phase() {
            Phase::GameOver { .. } => Err(err(GameError::GameOver)),
            _ if self.game.current() != HUMAN => Err(format!("c'est au tour de {AI_NAME}")),
            _ => Ok(()),
        }
    }
}

fn err(e: GameError) -> String {
    e.to_string()
}

fn roll_event(player: usize, outcome: RollOutcome) -> EventView {
    match outcome {
        RollOutcome::Scoring(dice) => EventView::Rolled {
            player,
            dice: values(&dice),
        },
        RollOutcome::Farkle { dice, lost_points } => EventView::Farkle {
            player,
            dice: values(&dice),
            lost_points,
        },
    }
}

/// Toutes les mises de côté possibles d'un lancer (1 à 6 dés), en JSON.
#[wasm_bindgen]
pub fn combos(dice: Vec<u8>) -> Result<String, String> {
    if dice.is_empty() || dice.len() > DICE_COUNT {
        return Err(format!("il faut entre 1 et {DICE_COUNT} dés"));
    }
    let roll = dice_from(&dice).ok_or("chaque dé doit valoir entre 1 et 6")?;
    Ok(serde_json::to_string(&combo_views(&roll)).expect("combinaisons sérialisables"))
}

/// Règles du jeu au format Markdown.
#[wasm_bindgen]
pub fn rules_markdown() -> String {
    farkle_core::RULES_MD.to_string()
}
