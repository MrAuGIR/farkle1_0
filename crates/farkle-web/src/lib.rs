//! Façade WebAssembly du moteur Farkle (bindings ajoutés au lot 6).

pub fn rules_markdown() -> &'static str {
    farkle_core::RULES_MD
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_rules() {
        assert!(super::rules_markdown().contains("Farkle"));
    }
}
