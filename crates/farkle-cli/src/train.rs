//! Mode entraîner : le joueur saisit ses propres lancers (dés réels ou partie en
//! cours dans un autre jeu) et le conseiller indique quoi garder et s'il vaut
//! mieux relancer ou encaisser.

use crate::display;
use crate::input::{Command, parse_command, parse_roll};
use farkle_core::advisor::{Decision, Risk, decide, expected_reroll, farkle_probability, suggest};
use farkle_core::combos::indices_for;
use farkle_core::game::{Game, GameError, Phase, RollOutcome};
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
    let game = Game::new(&["Vous"], config.target)
        .and_then(|g| g.with_scores(&[config.score]))
        .map_err(io::Error::other)?;
    let mut session = Session {
        game,
        turns: 0,
        rerolling: false,
        input,
        out,
    };
    session.run()
}

struct Session<'a> {
    game: Game,
    turns: u32,
    /// Le joueur a choisi de relancer : on attend la saisie du lancer.
    rerolling: bool,
    input: &'a mut dyn BufRead,
    out: &'a mut dyn Write,
}

impl Session<'_> {
    fn run(&mut self) -> io::Result<TrainSummary> {
        writeln!(
            self.out,
            "=== Mode entraîner — objectif {} pts, score actuel {} pts ===",
            self.game.target(),
            self.score()
        )?;
        writeln!(
            self.out,
            "Saisissez chaque lancer (ex. 1 5 5 3 2 6 ou 155326), puis les numéros des dés gardés.\n\
             s = suivre le conseil, r = je relance, b = j'encaisse, ? = règles, q = quitter."
        )?;
        let reached_target = loop {
            if self.game.winner().is_some() {
                writeln!(
                    self.out,
                    "\n*** Objectif atteint en {} tour(s) ! ***",
                    self.turns
                )?;
                break true;
            }
            let keep_going = match self.game.phase() {
                Phase::Start => self.ask_roll()?,
                Phase::Kept if self.rerolling => self.ask_roll()?,
                Phase::Choosing => self.ask_keep()?,
                Phase::Kept => self.ask_decision()?,
                Phase::GameOver { .. } => true,
            };
            if !keep_going {
                break false;
            }
        };
        let summary = TrainSummary {
            turns: self.turns,
            score: self.score(),
            reached_target,
        };
        writeln!(
            self.out,
            "\nBilan : {} tour(s), score {} pts.",
            summary.turns, summary.score
        )?;
        Ok(summary)
    }

    fn score(&self) -> u32 {
        self.game.players()[0].score
    }

    /// Saisie d'un lancer. Renvoie `false` si le joueur quitte.
    fn ask_roll(&mut self) -> io::Result<bool> {
        if self.game.phase() == Phase::Start {
            writeln!(
                self.out,
                "\n--- Nouveau tour --- score {} / {}",
                self.score(),
                self.game.target()
            )?;
        }
        let n = self.game.dice_left();
        let Some(line) = self.read(&format!("Lancer ({}) : ", display::dice_count(n)))? else {
            return Ok(false);
        };
        match line.trim().to_lowercase().as_str() {
            "q" | "quitter" => return Ok(false),
            "?" | "regles" | "règles" => return self.rules().map(|_| true),
            _ => {}
        }
        let dice = match parse_roll(&line, n) {
            Ok(dice) => dice,
            Err(e) => return self.error(e).map(|_| true),
        };
        self.rerolling = false;
        match self.game.submit_roll(dice).map_err(io::Error::other)? {
            RollOutcome::Farkle { dice, lost_points } => {
                writeln!(self.out, "{}", display::roll(&dice))?;
                writeln!(
                    self.out,
                    "Farkle ! Aucun dé ne marque, {lost_points} pts perdus."
                )?;
                self.turns += 1;
            }
            RollOutcome::Scoring(dice) => {
                writeln!(self.out, "{}", display::roll(&dice))?;
                let advice = suggest(
                    &dice,
                    self.game.turn_points(),
                    self.score(),
                    self.game.target(),
                    Risk::NORMAL,
                );
                writeln!(
                    self.out,
                    "Combinaisons possibles :\n{}",
                    display::combos(&advice.combos)
                )?;
                if let Some(best) = &advice.best {
                    writeln!(self.out, "{}", display::advice(best, &dice))?;
                }
            }
        }
        Ok(true)
    }

    /// Saisie des dés gardés. Renvoie `false` si le joueur quitte.
    fn ask_keep(&mut self) -> io::Result<bool> {
        let Some(command) = self.read_command("Dés gardés (numéros, s = conseil) : ")? else {
            return Ok(false);
        };
        let indices = match command {
            Command::Keep(indices) => indices,
            Command::Suggest => {
                let roll = self.game.last_roll().to_vec();
                let advice = suggest(
                    &roll,
                    self.game.turn_points(),
                    self.score(),
                    self.game.target(),
                    Risk::NORMAL,
                );
                match advice.best.and_then(|b| indices_for(&roll, &b.combo.dice)) {
                    Some(indices) => indices,
                    None => return Ok(true),
                }
            }
            Command::Roll | Command::Bank => {
                self.error(GameError::WrongPhase {
                    phase: Phase::Choosing,
                })?;
                return Ok(true);
            }
            Command::ToggleHelp => {
                writeln!(self.out, "L'aide est toujours active en mode entraîner.")?;
                return Ok(true);
            }
            Command::Rules => return self.rules().map(|_| true),
            Command::Quit => return Ok(false),
        };
        match self.game.keep(&indices) {
            Ok(points) => {
                writeln!(
                    self.out,
                    "+{points} pts — {} pts au tour, {} à relancer.",
                    self.game.turn_points(),
                    display::dice_count(self.game.dice_left())
                )?;
                if self.game.is_hot_dice() {
                    writeln!(self.out, "Main chaude ! Vous pouvez relancer les 6 dés.")?;
                }
                let (t, n) = (self.game.turn_points(), self.game.dice_left());
                writeln!(
                    self.out,
                    "{}",
                    display::decision(self.advised_decision(), n, farkle_probability(n))
                )?;
                writeln!(
                    self.out,
                    "Espérance si vous relancez : {:.0} pts (contre {t} pts en encaissant).",
                    expected_reroll(t, n)
                )?;
            }
            Err(e) => self.error(e)?,
        }
        Ok(true)
    }

    /// Relancer ou encaisser. Renvoie `false` si le joueur quitte.
    fn ask_decision(&mut self) -> io::Result<bool> {
        let prompt = format!(
            "[r] je relance {}, [b] j'encaisse {} pts : ",
            display::dice_count(self.game.dice_left()),
            self.game.turn_points()
        );
        let Some(command) = self.read_command(&prompt)? else {
            return Ok(false);
        };
        let command = match command {
            Command::Suggest => match self.advised_decision() {
                Decision::Reroll => Command::Roll,
                Decision::Bank => Command::Bank,
            },
            c => c,
        };
        match command {
            Command::Roll => self.rerolling = true,
            Command::Bank => {
                let res = self.game.bank().map_err(io::Error::other)?;
                self.turns += 1;
                writeln!(
                    self.out,
                    "Vous encaissez {} pts → {} pts au total.",
                    res.banked, res.new_score
                )?;
            }
            Command::Keep(_) => self.error(GameError::WrongPhase { phase: Phase::Kept })?,
            Command::ToggleHelp => {
                writeln!(self.out, "L'aide est toujours active en mode entraîner.")?
            }
            Command::Rules => self.rules()?,
            Command::Quit => return Ok(false),
            Command::Suggest => unreachable!(),
        }
        Ok(true)
    }

    fn advised_decision(&self) -> Decision {
        decide(
            self.game.turn_points(),
            self.game.dice_left(),
            self.score(),
            self.game.target(),
            Risk::NORMAL,
        )
    }

    fn rules(&mut self) -> io::Result<()> {
        writeln!(self.out, "\n{}", farkle_core::RULES_MD)
    }

    fn error(&mut self, e: impl std::fmt::Display) -> io::Result<()> {
        writeln!(self.out, "✗ {e}")
    }

    /// Affiche `prompt` et lit une ligne ; `None` en fin d'entrée.
    fn read(&mut self, prompt: &str) -> io::Result<Option<String>> {
        write!(self.out, "{prompt}")?;
        self.out.flush()?;
        let mut line = String::new();
        if self.input.read_line(&mut line)? == 0 {
            writeln!(self.out)?;
            return Ok(None);
        }
        Ok(Some(line))
    }

    /// Lit une commande valide ; `None` en fin d'entrée.
    fn read_command(&mut self, prompt: &str) -> io::Result<Option<Command>> {
        loop {
            let Some(line) = self.read(prompt)? else {
                return Ok(None);
            };
            match parse_command(&line) {
                Ok(command) => return Ok(Some(command)),
                Err(e) => self.error(e)?,
            }
        }
    }
}
