//! Tests d'intégration de la boucle de jeu, avec des dés scriptés.

use farkle_cli::play::{PlayConfig, PlayResult, run_play};
use farkle_core::advisor::Risk;
use farkle_core::dice::ScriptedRoller;
use std::io::Cursor;

fn config(target: u32, help: bool) -> PlayConfig {
    PlayConfig {
        target,
        help,
        risk: Risk::NORMAL,
        player_name: "Henry".into(),
    }
}

fn play(cfg: PlayConfig, rolls: &[&[u8]], input: &str) -> (PlayResult, String) {
    let mut roller = ScriptedRoller::new(rolls.iter().map(|r| r.to_vec()).collect());
    let mut out = Vec::new();
    let result = run_play(
        &cfg,
        &mut roller,
        &mut Cursor::new(input.to_string()),
        &mut out,
    )
    .unwrap();
    (result, String::from_utf8(out).unwrap())
}

#[test]
fn keep_then_bank_wins() {
    let (res, out) = play(config(1000, false), &[&[1, 1, 1, 2, 3, 4]], "1 2 3\nb\n");
    assert_eq!(res, PlayResult::Finished { winner: 0 });
    assert!(out.contains("[1] [1] [1] [2] [3] [4]"), "{out}");
    assert!(out.contains("Henry gagne"), "{out}");
}

#[test]
fn invalid_selection_is_reported_and_asked_again() {
    let (res, out) = play(
        config(1000, false),
        &[&[1, 1, 1, 2, 3, 4]],
        "4\nzz\n1 2 3\nb\n",
    );
    assert_eq!(res, PlayResult::Finished { winner: 0 });
    assert!(out.contains("ne marque pas"), "{out}");
    assert!(out.contains("« zz »"), "{out}");
}

#[test]
fn farkle_passes_the_hand_to_the_ai() {
    let (res, out) = play(
        config(1000, false),
        &[&[2, 2, 3, 3, 4, 6], &[1, 1, 1, 2, 3, 4]],
        "",
    );
    assert_eq!(res, PlayResult::Finished { winner: 1 });
    assert!(out.contains("Farkle"), "{out}");
    assert!(out.contains("Hans gagne"), "{out}");
}

#[test]
fn reroll_accumulates_and_ai_plays_its_turn() {
    let (res, out) = play(
        config(4000, false),
        &[
            &[1, 2, 3, 4, 6, 6], // Henry garde le 1
            &[5, 2, 3, 4, 6],    // relance 5 dés, garde la suite 2-6 (750)
            &[2, 2, 3, 3, 4, 6], // Hans : Farkle
        ],
        "1\nr\n1 2 3 4 5\nb\nq\n",
    );
    assert_eq!(res, PlayResult::Quit);
    assert!(out.contains("850"), "{out}");
    assert!(out.contains("Henry 850 | Hans 0"), "{out}");
}

#[test]
fn hot_dice_is_announced() {
    let (_, out) = play(
        config(4000, false),
        &[&[1, 2, 3, 4, 5, 6], &[2, 2, 3, 3, 4, 6]],
        "123456\nr\nq\n",
    );
    assert!(out.contains("Main chaude"), "{out}");
}

#[test]
fn help_on_shows_combos_and_advice() {
    let (_, out) = play(config(1000, true), &[&[1, 1, 1, 2, 3, 4]], "1 2 3\nb\n");
    assert!(out.contains("Combinaisons possibles"), "{out}");
    assert!(out.contains("1 1 1 → 1000 pts (trois 1)"), "{out}");
    assert!(out.contains("Conseil"), "{out}");
}

#[test]
fn help_off_hides_combos() {
    let (_, out) = play(config(1000, false), &[&[1, 1, 1, 2, 3, 4]], "1 2 3\nb\n");
    assert!(!out.contains("Combinaisons possibles"), "{out}");
}

#[test]
fn help_can_be_toggled_during_the_game() {
    let (_, out) = play(config(1000, false), &[&[1, 1, 1, 2, 3, 4]], "a\n1 2 3\nb\n");
    assert!(out.contains("Aide activée"), "{out}");
    assert!(out.contains("Combinaisons possibles"), "{out}");
}

#[test]
fn suggest_command_follows_the_advice() {
    let (res, out) = play(config(1000, false), &[&[1, 1, 1, 2, 3, 4]], "s\ns\n");
    assert_eq!(res, PlayResult::Finished { winner: 0 });
    assert!(out.contains("Henry gagne"), "{out}");
}

#[test]
fn rules_can_be_displayed_in_game() {
    let (_, out) = play(config(1000, false), &[&[1, 1, 1, 2, 3, 4]], "?\nq\n");
    assert!(out.contains("Tableau des scores"), "{out}");
}

#[test]
fn quit_and_end_of_input() {
    let (res, _) = play(config(1000, false), &[&[1, 1, 1, 2, 3, 4]], "q\n");
    assert_eq!(res, PlayResult::Quit);
    let (res, _) = play(config(1000, false), &[&[1, 1, 1, 2, 3, 4]], "");
    assert_eq!(res, PlayResult::Quit);
}

#[test]
fn wrong_phase_commands_are_explained() {
    // `b` avant d'avoir gardé des dés, `r` aussi : refusés avec un message.
    let (_, out) = play(config(1000, false), &[&[1, 1, 1, 2, 3, 4]], "b\nr\nq\n");
    assert!(out.contains("mettre de côté"), "{out}");
}
