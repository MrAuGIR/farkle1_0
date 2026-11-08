//! Tests d'intégration : l'IA joue des parties entières avec des dés aléatoires.

use farkle_core::advisor::Risk;
use farkle_core::ai::AiPlayer;
use farkle_core::dice::RandomRoller;
use farkle_core::game::Game;

#[test]
fn ai_vs_ai_games_always_finish_with_a_valid_winner() {
    for seed in 0..50 {
        let mut g = Game::new(&["Hans", "Henry"], 4000).unwrap();
        let mut r = RandomRoller::from_seed(seed);
        let players = [AiPlayer::new(Risk::CAUTIOUS), AiPlayer::new(Risk::BOLD)];
        let mut turns = 0;
        while g.winner().is_none() {
            players[g.current()].play_turn(&mut g, &mut r).unwrap();
            turns += 1;
            assert!(turns < 1000, "partie sans fin (graine {seed})");
        }
        let w = g.winner().unwrap();
        assert!(g.players()[w].score >= 4000);
        assert!(g.players()[1 - w].score < 4000);
    }
}

#[test]
fn normal_ai_beats_a_reckless_strategy_on_average() {
    // L'IA normale doit battre une IA téméraire (qui relance presque toujours).
    let normal = AiPlayer::new(Risk::NORMAL);
    let reckless = AiPlayer::new(Risk(20.0));
    let mut wins = 0;
    for seed in 0..200 {
        // On alterne qui commence.
        let normal_first = seed % 2 == 0;
        let (names, players) = if normal_first {
            (["Normale", "Téméraire"], [normal, reckless])
        } else {
            (["Téméraire", "Normale"], [reckless, normal])
        };
        let mut g = Game::new(&names, 4000).unwrap();
        let mut r = RandomRoller::from_seed(seed);
        while g.winner().is_none() {
            players[g.current()].play_turn(&mut g, &mut r).unwrap();
        }
        if g.players()[g.winner().unwrap()].name == "Normale" {
            wins += 1;
        }
    }
    assert!(wins > 110, "victoires de l'IA normale : {wins}/200");
}
