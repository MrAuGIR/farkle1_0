//! Tests d'intégration du binaire `farkle`.

use assert_cmd::Command;
use predicates::prelude::*;

fn farkle() -> Command {
    Command::cargo_bin("farkle").unwrap()
}

#[test]
fn long_help_shows_usage_and_rules() {
    farkle()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("play"))
        .stdout(predicate::str::contains("Tableau des scores"));
}

#[test]
fn play_help_lists_options() {
    farkle()
        .args(["play", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--target"))
        .stdout(predicate::str::contains("--aide"))
        .stdout(predicate::str::contains("--seed").not());
}

#[test]
fn rules_subcommand_prints_rules() {
    farkle()
        .arg("rules")
        .assert()
        .success()
        .stdout(predicate::str::contains("Main chaude"));
}

#[test]
fn invalid_target_fails() {
    farkle().args(["play", "--target", "0"]).assert().failure();
}

#[test]
fn quit_immediately() {
    farkle()
        .args(["play", "--seed", "1"])
        .write_stdin("q\nq\nq\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Au revoir"));
}

#[test]
fn seeded_game_following_advice_reaches_the_end() {
    // `s` suit le conseil à chaque étape : la partie finit forcément.
    farkle()
        .args(["play", "--seed", "42", "--target", "1500", "--aide", "on"])
        .write_stdin("s\n".repeat(500))
        .assert()
        .success()
        .stdout(predicate::str::contains("Partie terminée"))
        .stdout(predicate::str::contains("Combinaisons possibles"));
}

#[test]
fn train_session_through_the_binary() {
    farkle()
        .args(["train", "--target", "1000"])
        .write_stdin("1 1 1 5 2 3\ns\nb\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 1 1 5 → 1050 pts"))
        .stdout(predicate::str::contains("Objectif atteint"));
}

#[test]
fn train_rejects_a_score_above_target() {
    farkle()
        .args(["train", "--target", "1000", "--score", "1000"])
        .assert()
        .failure();
}
