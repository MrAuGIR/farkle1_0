//! Façade WebAssembly du moteur Farkle.
//!
//! `WebGame` oppose le joueur humain (indice 0) à l'IA (indice 1). Chaque action
//! renvoie l'état de la partie sérialisé en JSON (voir [`view::GameView`]) ou un
//! message d'erreur en français. Le code est du Rust ordinaire : il se teste en
//! natif, la couche `wasm_bindgen` ne fait que l'exposer à JavaScript.

pub mod view;

use farkle_core::dice::DiceRoller;
use wasm_bindgen::prelude::*;

/// Nom affiché du joueur humain.
pub const HUMAN_NAME: &str = "Vous";
/// Nom affiché de l'IA.
pub const AI_NAME: &str = "Hans";

/// Partie humain contre IA pilotée depuis JavaScript.
#[wasm_bindgen]
pub struct WebGame {}

#[wasm_bindgen]
impl WebGame {
    /// Nouvelle partie avec un lanceur aléatoire initialisé par `seed`.
    #[wasm_bindgen(constructor)]
    pub fn new(target: u32, seed: u32) -> Result<WebGame, String> {
        let _ = (target, seed);
        todo!()
    }

    /// État courant en JSON.
    pub fn state(&self) -> String {
        todo!()
    }

    /// Lance les dés (début de tour). Renvoie le nouvel état.
    pub fn roll(&mut self) -> Result<String, String> {
        todo!()
    }

    /// Met de côté les dés aux positions `indices` (base 0) du dernier lancer.
    pub fn select(&mut self, indices: Vec<u32>) -> Result<String, String> {
        let _ = indices;
        todo!()
    }

    /// Relance les dés restants après une mise de côté.
    pub fn reroll(&mut self) -> Result<String, String> {
        todo!()
    }

    /// Encaisse les points du tour.
    pub fn bank(&mut self) -> Result<String, String> {
        todo!()
    }

    /// Joue une étape du tour de l'IA. Erreur si ce n'est pas son tour.
    pub fn ai_turn_step(&mut self) -> Result<String, String> {
        todo!()
    }
}

impl WebGame {
    /// Nouvelle partie avec un lanceur fourni (tests natifs).
    pub fn with_roller(target: u32, roller: Box<dyn DiceRoller>) -> Result<WebGame, String> {
        let _ = (target, roller);
        todo!()
    }
}

/// Toutes les mises de côté possibles d'un lancer, en JSON.
#[wasm_bindgen]
pub fn combos(dice: Vec<u8>) -> Result<String, String> {
    let _ = dice;
    todo!()
}

/// Règles du jeu au format Markdown.
#[wasm_bindgen]
pub fn rules_markdown() -> String {
    farkle_core::RULES_MD.to_string()
}
