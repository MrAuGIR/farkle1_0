//! Boucle de jeu console : le joueur humain contre l'IA.

use farkle_core::advisor::Risk;
use farkle_core::dice::DiceRoller;
use std::io::{self, BufRead, Write};

/// Nom de l'adversaire IA.
pub const AI_NAME: &str = "Hans";

#[derive(Debug, Clone, PartialEq)]
pub struct PlayConfig {
    pub target: u32,
    /// Aide activée au démarrage.
    pub help: bool,
    /// Style de jeu de l'IA.
    pub risk: Risk,
    pub player_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayResult {
    /// Partie terminée ; `winner` vaut 0 pour le joueur humain, 1 pour l'IA.
    Finished { winner: usize },
    /// Le joueur a quitté (commande `q` ou fin de l'entrée).
    Quit,
}

/// Joue une partie complète en lisant les commandes sur `input`.
pub fn run_play(
    config: &PlayConfig,
    roller: &mut dyn DiceRoller,
    input: &mut dyn BufRead,
    out: &mut dyn Write,
) -> io::Result<PlayResult> {
    let _ = (config, roller, input, out);
    todo!()
}
