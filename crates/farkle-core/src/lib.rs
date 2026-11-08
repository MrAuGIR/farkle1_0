//! Moteur du Farkle : dés, scoring, partie, conseiller et IA (sans I/O).

/// Règles du jeu (source unique : `docs/REGLES.md`).
pub const RULES_MD: &str = include_str!("../../../docs/REGLES.md");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_are_embedded() {
        assert!(RULES_MD.contains("Farkle"));
    }
}
