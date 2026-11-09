//! Tests dans un moteur WebAssembly : `wasm-pack test --node crates/farkle-web`.
#![cfg(target_arch = "wasm32")]

use farkle_web::{WebGame, combos, rules_markdown};
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn plays_a_turn_in_wasm() {
    let mut g = WebGame::new(4000, 1).unwrap();
    let s: serde_json::Value = serde_json::from_str(&g.roll().unwrap()).unwrap();
    assert_eq!(s["players"][0]["name"], "Vous");
    assert!(s["phase"] == "choosing" || s["ai_turn"] == true);
}

#[wasm_bindgen_test]
fn errors_cross_the_boundary() {
    let mut g = WebGame::new(4000, 1).unwrap();
    assert!(g.bank().is_err());
    assert!(WebGame::new(0, 1).is_err());
}

#[wasm_bindgen_test]
fn combos_and_rules() {
    assert!(combos(vec![1, 5]).unwrap().contains("150"));
    assert!(rules_markdown().contains("Farkle"));
}
