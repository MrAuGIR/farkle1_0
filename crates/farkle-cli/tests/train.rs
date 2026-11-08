//! Tests d'intégration du mode entraîner.

use farkle_cli::train::{TrainConfig, TrainSummary, run_train};
use std::io::Cursor;

fn train(target: u32, score: u32, input: &str) -> (TrainSummary, String) {
    let mut out = Vec::new();
    let summary = run_train(
        &TrainConfig { target, score },
        &mut Cursor::new(input.to_string()),
        &mut out,
    )
    .unwrap();
    (summary, String::from_utf8(out).unwrap())
}

#[test]
fn full_turn_with_advice() {
    let (summary, out) = train(4000, 0, "1 5 5 3 2 6\n1\nr\n5 5 5 3 2\n1 2 3\nb\nq\n");
    assert!(out.contains("Lancer (6 dés)"), "{out}");
    assert!(out.contains("[1] [5] [5] [3] [2] [6]"), "{out}");
    assert!(out.contains("Combinaisons possibles"), "{out}");
    assert!(out.contains("1 5 5 → 200 pts"), "{out}");
    assert!(out.contains("Conseil : garder 1 (dé n°1)"), "{out}");
    assert!(out.contains("Lancer (5 dés)"), "{out}");
    assert!(out.contains("Espérance si vous relancez"), "{out}");
    assert!(out.contains("600 pts"), "{out}");
    assert_eq!(
        summary,
        TrainSummary {
            turns: 1,
            score: 600,
            reached_target: false
        }
    );
}

#[test]
fn wrong_dice_count_is_asked_again() {
    let (_, out) = train(4000, 0, "1 2 3\n1 2 3 4 6 6\nq\n");
    assert!(out.contains("6 dés attendus, 3 saisis"), "{out}");
    assert!(out.contains("[1] [2] [3] [4] [6] [6]"), "{out}");
}

#[test]
fn invalid_die_value_is_reported() {
    let (_, out) = train(4000, 0, "1 2 7 4 5 6\nq\n");
    assert!(out.contains("« 7 »"), "{out}");
}

#[test]
fn farkle_resets_the_turn() {
    let (summary, out) = train(4000, 0, "1 2 3 4 6 6\n1\nr\n2 3 4 6 6\nq\n");
    assert!(out.contains("Farkle"), "{out}");
    assert!(out.contains("100 pts perdus"), "{out}");
    // Nouveau tour : on redemande 6 dés.
    assert_eq!(out.matches("Lancer (6 dés)").count(), 2, "{out}");
    assert_eq!(
        summary,
        TrainSummary {
            turns: 1,
            score: 0,
            reached_target: false
        }
    );
}

#[test]
fn hot_dice_asks_for_six_dice_again() {
    let (_, out) = train(4000, 0, "1 2 3 4 5 6\n123456\nr\nq\n");
    assert!(out.contains("Main chaude"), "{out}");
    assert_eq!(out.matches("Lancer (6 dés)").count(), 2, "{out}");
}

#[test]
fn suggest_keeps_the_advised_dice() {
    let (summary, _) = train(4000, 0, "1 1 1 2 3 4\ns\nb\nq\n");
    assert_eq!(summary.score, 1000);
}

#[test]
fn invalid_keep_is_refused() {
    let (_, out) = train(4000, 0, "1 1 1 2 3 4\n4\nr\n1 2 3\nb\nq\n");
    assert!(out.contains("ne marque pas"), "{out}");
    assert!(out.contains("mettre de côté"), "{out}");
}

#[test]
fn reaching_the_target_ends_the_session() {
    let (summary, out) = train(1000, 0, "1 1 1 2 3 4\n1 2 3\nb\n");
    assert!(out.contains("Objectif atteint"), "{out}");
    assert_eq!(
        summary,
        TrainSummary {
            turns: 1,
            score: 1000,
            reached_target: true
        }
    );
}

#[test]
fn starting_score_changes_the_advice() {
    // À 3950 / 4000, garder le 5 fait gagner : le conseil est d'encaisser.
    let (_, out) = train(4000, 3950, "5 2 3 3 6 6\nq\n");
    assert!(out.contains("puis encaisser"), "{out}");
}

#[test]
fn invalid_starting_score_is_an_error() {
    let mut out = Vec::new();
    let res = run_train(
        &TrainConfig {
            target: 1000,
            score: 1000,
        },
        &mut Cursor::new(String::new()),
        &mut out,
    );
    assert!(res.is_err());
}

#[test]
fn rules_quit_and_end_of_input() {
    let (_, out) = train(4000, 0, "?\nq\n");
    assert!(out.contains("Tableau des scores"), "{out}");
    assert!(out.contains("Bilan"), "{out}");
    let (summary, _) = train(4000, 0, "");
    assert_eq!(
        summary,
        TrainSummary {
            turns: 0,
            score: 0,
            reached_target: false
        }
    );
}
