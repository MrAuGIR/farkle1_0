//! Mode entraîner : le joueur saisit ses propres lancers (dés réels ou partie en
//! cours dans un autre jeu) et le conseiller indique quoi garder et s'il vaut
//! mieux relancer ou encaisser.

use std::io::{self, BufRead, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrainConfig {
    pub target: u32,
    /// Score de départ.
    pub score: u32,
}

/// Bilan de la séance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrainSummary {
    /// Tours terminés (encaissés ou perdus sur un Farkle).
    pub turns: u32,
    pub score: u32,
    /// `true` si le score cible a été atteint.
    pub reached_target: bool,
}

/// Lance une séance d'entraînement interactive.
pub fn run_train(
    config: &TrainConfig,
    input: &mut dyn BufRead,
    out: &mut dyn Write,
) -> io::Result<TrainSummary> {
    let _ = (config, input, out);
    todo!()
}
