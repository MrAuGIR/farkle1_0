//! Tests natifs de la façade web : actions, état JSON, tours de l'IA.

use farkle_core::dice::ScriptedRoller;
use farkle_web::{AI_NAME, HUMAN_NAME, WebGame, combos, rules_markdown};
use serde_json::{Value, json};

fn game(target: u32, rolls: &[&[u8]]) -> WebGame {
    let roller = ScriptedRoller::new(rolls.iter().map(|r| r.to_vec()).collect());
    WebGame::with_roller(target, Box::new(roller)).unwrap()
}

fn parse(s: String) -> Value {
    serde_json::from_str(&s).unwrap_or_else(|e| panic!("JSON invalide ({e}) : {s}"))
}

fn state(g: &WebGame) -> Value {
    parse(g.state())
}

#[test]
fn rejects_zero_target() {
    assert!(WebGame::new(0, 42).is_err());
    assert!(WebGame::new(4000, 42).is_ok());
}

#[test]
fn initial_state() {
    let s = state(&game(3000, &[]));
    assert_eq!(s["target"], 3000);
    assert_eq!(s["phase"], "start");
    assert_eq!(s["current"], 0);
    assert_eq!(s["ai_turn"], false);
    assert_eq!(s["winner"], Value::Null);
    assert_eq!(
        s["players"],
        json!([{"name": HUMAN_NAME, "score": 0}, {"name": AI_NAME, "score": 0}])
    );
    assert_eq!(s["turn_points"], 0);
    assert_eq!(s["dice_left"], 6);
    assert_eq!(s["dice"], json!([]));
    assert_eq!(s["hot_dice"], false);
    assert_eq!(s["combos"], json!([]));
    assert_eq!(s["advice"], Value::Null);
    assert_eq!(s["recommendation"], Value::Null);
    assert_eq!(s["last_event"], Value::Null);
}

#[test]
fn roll_shows_dice_combos_and_advice() {
    let mut g = game(4000, &[&[1, 1, 1, 2, 3, 4]]);
    let s = parse(g.roll().unwrap());
    assert_eq!(s, state(&g), "l'action renvoie le nouvel état");
    assert_eq!(s["phase"], "choosing");
    assert_eq!(s["dice"], json!([1, 1, 1, 2, 3, 4]));
    assert_eq!(
        s["last_event"],
        json!({"kind": "rolled", "player": 0, "dice": [1, 1, 1, 2, 3, 4]})
    );

    let best = &s["combos"][0];
    assert_eq!(best["score"], 1000);
    assert_eq!(best["dice"], json!([1, 1, 1]));
    assert_eq!(best["indices"], json!([0, 1, 2]));
    assert_eq!(best["patterns"], json!(["trois 1"]));
    assert!(s["combos"].as_array().unwrap().len() > 1);

    let advice = &s["advice"];
    assert_eq!(advice["indices"], json!([0, 1, 2]));
    assert_eq!(advice["score"], 1000);
    assert_eq!(advice["turn_points_after"], 1000);
    assert_eq!(advice["dice_left_after"], 3);
    assert!(advice["farkle_probability"].as_f64().unwrap() > 0.0);
    assert!(advice["expected_reroll"].is_number());
    assert!(["reroll", "bank"].contains(&advice["decision"].as_str().unwrap()));
}

#[test]
fn invalid_selection_is_rejected_without_changing_state() {
    let mut g = game(4000, &[&[1, 1, 1, 2, 3, 4]]);
    g.roll().unwrap();
    let before = g.state();
    let err = g.select(vec![3]).unwrap_err();
    assert!(err.contains("ne marque pas"), "{err}");
    assert!(g.select(vec![]).is_err());
    assert!(g.select(vec![9]).is_err());
    assert!(g.select(vec![0, 0]).is_err());
    assert_eq!(g.state(), before);
}

#[test]
fn select_then_reroll() {
    let mut g = game(4000, &[&[1, 1, 1, 2, 3, 4], &[5, 2, 3]]);
    assert!(
        g.reroll().is_err(),
        "pas de relance avant le premier lancer"
    );
    g.roll().unwrap();
    assert!(g.reroll().is_err(), "il faut d'abord garder des dés");

    let s = parse(g.select(vec![0, 1, 2]).unwrap());
    assert_eq!(s["phase"], "kept");
    assert_eq!(s["turn_points"], 1000);
    assert_eq!(s["dice_left"], 3);
    assert_eq!(s["dice"], json!([]));
    assert_eq!(s["advice"], Value::Null);
    assert_eq!(
        s["last_event"],
        json!({"kind": "kept", "player": 0, "dice": [1, 1, 1], "points": 1000})
    );
    let rec = &s["recommendation"];
    assert!(["reroll", "bank"].contains(&rec["decision"].as_str().unwrap()));
    assert!(rec["farkle_probability"].as_f64().unwrap() > 0.0);
    assert!(rec["expected_reroll"].is_number());

    assert!(g.roll().is_ok(), "roll relance aussi les dés restants");
    let s = state(&g);
    assert_eq!(s["dice"], json!([5, 2, 3]));
    assert_eq!(s["turn_points"], 1000);
}

#[test]
fn reroll_after_keep() {
    let mut g = game(4000, &[&[1, 1, 1, 2, 3, 4], &[5, 2, 3]]);
    g.roll().unwrap();
    g.select(vec![0, 1, 2]).unwrap();
    let s = parse(g.reroll().unwrap());
    assert_eq!(s["phase"], "choosing");
    assert_eq!(s["dice"], json!([5, 2, 3]));
}

#[test]
fn bank_passes_hand_to_ai() {
    let mut g = game(4000, &[&[1, 1, 1, 2, 3, 4]]);
    assert!(g.bank().is_err());
    g.roll().unwrap();
    g.select(vec![0, 1, 2]).unwrap();
    let s = parse(g.bank().unwrap());
    assert_eq!(s["players"][0]["score"], 1000);
    assert_eq!(s["current"], 1);
    assert_eq!(s["ai_turn"], true);
    assert_eq!(s["phase"], "start");
    assert_eq!(
        s["last_event"],
        json!({"kind": "banked", "player": 0, "points": 1000, "new_score": 1000, "won": false})
    );
}

#[test]
fn human_cannot_act_during_ai_turn_and_ai_cannot_act_during_human_turn() {
    let mut g = game(4000, &[&[1, 1, 1, 2, 3, 4]]);
    assert!(g.ai_turn_step().is_err());
    g.roll().unwrap();
    g.select(vec![0, 1, 2]).unwrap();
    g.bank().unwrap();
    let before = g.state();
    for err in [g.roll(), g.select(vec![0]), g.reroll(), g.bank()] {
        assert!(err.unwrap_err().contains(AI_NAME));
    }
    assert_eq!(g.state(), before);
}

#[test]
fn ai_plays_step_by_step() {
    let mut g = game(4000, &[&[1, 1, 1, 2, 3, 4], &[2, 3, 4, 6, 2, 3]]);
    g.roll().unwrap();
    g.select(vec![0, 1, 2]).unwrap();
    g.bank().unwrap();

    let s = parse(g.ai_turn_step().unwrap());
    assert_eq!(
        s["last_event"],
        json!({"kind": "farkle", "player": 1, "dice": [2, 3, 4, 6, 2, 3], "lost_points": 0})
    );
    assert_eq!(s["ai_turn"], false);
    assert_eq!(s["current"], 0);
    assert_eq!(s["phase"], "start");
}

#[test]
fn human_farkle_passes_hand() {
    let mut g = game(4000, &[&[2, 2, 3, 3, 4, 6]]);
    let s = parse(g.roll().unwrap());
    assert_eq!(
        s["last_event"],
        json!({"kind": "farkle", "player": 0, "dice": [2, 2, 3, 3, 4, 6], "lost_points": 0})
    );
    assert_eq!(s["ai_turn"], true);
    assert_eq!(s["dice"], json!([]));
}

#[test]
fn hot_dice() {
    let mut g = game(4000, &[&[1, 1, 1, 5, 5, 5]]);
    g.roll().unwrap();
    let s = parse(g.select(vec![0, 1, 2, 3, 4, 5]).unwrap());
    assert_eq!(s["hot_dice"], true);
    assert_eq!(s["dice_left"], 6);
    assert_eq!(s["turn_points"], 1500);
}

#[test]
fn victory_ends_the_game() {
    let mut g = game(1000, &[&[1, 1, 1, 2, 3, 4]]);
    g.roll().unwrap();
    g.select(vec![0, 1, 2]).unwrap();
    let s = parse(g.bank().unwrap());
    assert_eq!(s["phase"], "game_over");
    assert_eq!(s["winner"], 0);
    assert_eq!(s["ai_turn"], false);
    assert_eq!(s["last_event"]["won"], true);
    assert!(g.roll().is_err());
    assert!(g.ai_turn_step().is_err());
}

#[test]
fn combos_of_a_roll() {
    let list = parse(combos(vec![1, 1, 1, 5, 2, 3]).unwrap());
    let list = list.as_array().unwrap();
    assert_eq!(list[0]["score"], 1050);
    assert_eq!(list[0]["dice"], json!([1, 1, 1, 5]));
    assert_eq!(list[0]["indices"], json!([0, 1, 2, 3]));
    assert!(
        list.iter()
            .any(|c| c["score"] == 1000 && c["patterns"] == json!(["trois 1"]))
    );

    assert_eq!(parse(combos(vec![2, 2, 3, 3, 4, 6]).unwrap()), json!([]));
    assert!(combos(vec![7]).is_err());
    assert!(combos(vec![0, 1]).is_err());
    assert!(combos(vec![]).is_err());
    assert!(combos(vec![1; 7]).is_err());
}

#[test]
fn exposes_rules() {
    assert!(rules_markdown().contains("Farkle"));
}

/// Joue une partie complète en suivant les conseils ; renvoie l'état final.
fn autoplay(seed: u32) -> String {
    let mut g = WebGame::new(2000, seed).unwrap();
    for _ in 0..10_000 {
        let s = state(&g);
        if s["phase"] == "game_over" {
            return g.state();
        }
        let next = if s["ai_turn"] == true {
            g.ai_turn_step()
        } else {
            match s["phase"].as_str().unwrap() {
                "start" => g.roll(),
                "choosing" => {
                    let indices = s["advice"]["indices"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|i| i.as_u64().unwrap() as u32)
                        .collect();
                    g.select(indices)
                }
                "kept" if s["recommendation"]["decision"] == "reroll" => g.reroll(),
                "kept" => g.bank(),
                phase => panic!("phase inattendue : {phase}"),
            }
        };
        next.unwrap();
    }
    panic!("la partie ne s'est pas terminée");
}

#[test]
fn full_random_game_is_deterministic_for_a_seed() {
    let end = parse(autoplay(7));
    assert_eq!(end["phase"], "game_over");
    let winner = end["winner"].as_u64().unwrap() as usize;
    assert!(end["players"][winner]["score"].as_u64().unwrap() >= 2000);
    assert_eq!(autoplay(7), autoplay(7));
}
