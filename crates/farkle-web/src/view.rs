//! Vues sérialisables (JSON) de l'état de la partie, pour JavaScript.

use farkle_core::advisor::{self, Decision, Risk};
use farkle_core::combos::{Combo, find_combos, indices_for};
use farkle_core::dice::Die;
use farkle_core::game::{Game, Phase};
use serde::Serialize;

/// Indice du joueur humain dans la partie.
pub const HUMAN: usize = 0;

/// État complet de la partie tel que vu par l'interface.
#[derive(Debug, Serialize)]
pub struct GameView {
    pub target: u32,
    pub players: Vec<PlayerView>,
    /// Indice du joueur dont c'est le tour (0 = humain, 1 = IA).
    pub current: usize,
    /// `true` si c'est à l'IA de jouer (appeler `ai_turn_step`).
    pub ai_turn: bool,
    /// `start`, `choosing`, `kept` ou `game_over`.
    pub phase: &'static str,
    pub winner: Option<usize>,
    pub turn_points: u32,
    /// Dés à lancer au prochain lancer.
    pub dice_left: usize,
    /// Dés du dernier lancer, quand il faut choisir quoi garder (vide sinon).
    pub dice: Vec<u8>,
    pub hot_dice: bool,
    /// Mises de côté possibles du lancer en cours (vide hors phase `choosing`).
    pub combos: Vec<ComboView>,
    /// Meilleure mise de côté (phase `choosing`).
    pub advice: Option<AdviceView>,
    /// Relancer ou encaisser (phase `kept`).
    pub recommendation: Option<RecommendationView>,
    /// Dernière chose qui s'est passée (pour l'affichage).
    pub last_event: Option<EventView>,
}

#[derive(Debug, Serialize)]
pub struct PlayerView {
    pub name: String,
    pub score: u32,
}

#[derive(Debug, Serialize)]
pub struct ComboView {
    pub dice: Vec<u8>,
    /// Positions (base 0) de ces dés dans le lancer.
    pub indices: Vec<usize>,
    pub score: u32,
    /// Motifs en français (« trois 1 », « un 5 », « suite 1-5 »…).
    pub patterns: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AdviceView {
    pub indices: Vec<usize>,
    pub score: u32,
    pub turn_points_after: u32,
    pub dice_left_after: usize,
    pub farkle_probability: f64,
    pub expected_reroll: f64,
    pub decision: DecisionView,
}

#[derive(Debug, Serialize)]
pub struct RecommendationView {
    pub decision: DecisionView,
    pub farkle_probability: f64,
    pub expected_reroll: f64,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionView {
    Reroll,
    Bank,
}

impl From<Decision> for DecisionView {
    fn from(d: Decision) -> Self {
        match d {
            Decision::Reroll => DecisionView::Reroll,
            Decision::Bank => DecisionView::Bank,
        }
    }
}

/// Événement de jeu, sérialisé avec un champ `kind`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EventView {
    Rolled {
        player: usize,
        dice: Vec<u8>,
    },
    Kept {
        player: usize,
        dice: Vec<u8>,
        points: u32,
    },
    Farkle {
        player: usize,
        dice: Vec<u8>,
        lost_points: u32,
    },
    Banked {
        player: usize,
        points: u32,
        new_score: u32,
        won: bool,
    },
}

pub fn values(dice: &[Die]) -> Vec<u8> {
    dice.iter().map(|d| d.value()).collect()
}

pub fn combo_view(roll: &[Die], combo: &Combo) -> ComboView {
    ComboView {
        dice: values(&combo.dice),
        indices: indices_for(roll, &combo.dice).unwrap_or_default(),
        score: combo.score,
        patterns: combo.patterns.iter().map(ToString::to_string).collect(),
    }
}

pub fn combo_views(roll: &[Die]) -> Vec<ComboView> {
    find_combos(roll)
        .iter()
        .map(|c| combo_view(roll, c))
        .collect()
}

impl GameView {
    pub fn new(game: &Game, last_event: Option<EventView>) -> GameView {
        let phase = game.phase();
        let current = game.current();
        let score = game.players()[current].score;
        let choosing = phase == Phase::Choosing;
        let roll = if choosing { game.last_roll() } else { &[] };

        let advice = choosing
            .then(|| advisor::suggest(roll, game.turn_points(), score, game.target(), Risk::NORMAL))
            .and_then(|a| a.best)
            .map(|best| AdviceView {
                indices: indices_for(roll, &best.combo.dice).unwrap_or_default(),
                score: best.combo.score,
                turn_points_after: best.turn_points_after,
                dice_left_after: best.dice_left_after,
                farkle_probability: best.farkle_probability,
                expected_reroll: best.expected_reroll,
                decision: best.decision.into(),
            });

        let recommendation = (phase == Phase::Kept).then(|| {
            let n = game.dice_left();
            RecommendationView {
                decision: advisor::decide(
                    game.turn_points(),
                    n,
                    score,
                    game.target(),
                    Risk::NORMAL,
                )
                .into(),
                farkle_probability: advisor::farkle_probability(n),
                expected_reroll: advisor::expected_reroll(game.turn_points(), n),
            }
        });

        let winner = match phase {
            Phase::GameOver { winner } => Some(winner),
            _ => None,
        };

        GameView {
            target: game.target(),
            players: game
                .players()
                .iter()
                .map(|p| PlayerView {
                    name: p.name.clone(),
                    score: p.score,
                })
                .collect(),
            current,
            ai_turn: winner.is_none() && current != HUMAN,
            phase: match phase {
                Phase::Start => "start",
                Phase::Choosing => "choosing",
                Phase::Kept => "kept",
                Phase::GameOver { .. } => "game_over",
            },
            winner,
            turn_points: game.turn_points(),
            dice_left: game.dice_left(),
            dice: values(roll),
            hot_dice: game.is_hot_dice(),
            combos: combo_views(roll),
            advice,
            recommendation,
            last_event,
        }
    }
}
