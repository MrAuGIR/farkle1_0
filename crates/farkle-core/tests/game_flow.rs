//! Tests d'intégration : parties complètes avec des lancers scriptés.

use farkle_core::dice::ScriptedRoller;
use farkle_core::game::{Game, Phase, RollOutcome};

fn roller(rolls: &[&[u8]]) -> ScriptedRoller {
    ScriptedRoller::new(rolls.iter().map(|r| r.to_vec()).collect())
}

#[test]
fn full_game_between_two_players() {
    let mut g = Game::new(&["Henry", "Hans"], 2000).unwrap();
    let mut r = roller(&[
        // Tour 1 — Henry : 1 1 1 (1000), relance 3 dés : 5 (50), encaisse 1050.
        &[1, 1, 1, 2, 3, 4],
        &[5, 2, 3],
        // Tour 2 — Hans : 5 (50), relance 5 dés : Farkle.
        &[5, 2, 3, 4, 6, 6],
        &[2, 3, 4, 6, 6],
        // Tour 3 — Henry : suite 1-6 (1500, main chaude), relance 6 : 1 (100), encaisse.
        &[6, 5, 4, 3, 2, 1],
        &[1, 2, 3, 4, 6, 6],
    ]);

    g.roll(&mut r).unwrap();
    g.keep(&[0, 1, 2]).unwrap();
    g.roll(&mut r).unwrap();
    g.keep(&[0]).unwrap();
    assert_eq!(g.bank().unwrap().new_score, 1050);

    g.roll(&mut r).unwrap();
    g.keep(&[0]).unwrap();
    assert!(matches!(
        g.roll(&mut r).unwrap(),
        RollOutcome::Farkle {
            lost_points: 50,
            ..
        }
    ));
    assert_eq!(g.players()[1].score, 0);

    g.roll(&mut r).unwrap();
    g.keep(&[0, 1, 2, 3, 4, 5]).unwrap();
    assert!(g.is_hot_dice());
    g.roll(&mut r).unwrap();
    g.keep(&[0]).unwrap();
    let out = g.bank().unwrap();
    assert_eq!(out.new_score, 2650);
    assert!(out.won);
    assert_eq!(g.phase(), Phase::GameOver { winner: 0 });
}

#[test]
fn manual_rolls_drive_a_turn() {
    use farkle_core::dice::dice_from;
    let mut g = Game::new(&["Joueur"], 4000).unwrap();
    g.submit_roll(dice_from(&[1, 5, 5, 3, 2, 6]).unwrap())
        .unwrap();
    g.keep(&[0]).unwrap();
    g.submit_roll(dice_from(&[5, 5, 5, 3, 2]).unwrap()).unwrap();
    g.keep(&[0, 1, 2]).unwrap();
    assert_eq!(g.turn_points(), 600);
    assert_eq!(g.dice_left(), 2);
    assert_eq!(g.bank().unwrap().new_score, 600);
}
