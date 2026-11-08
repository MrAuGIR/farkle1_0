//! Boucle de jeu console : le joueur humain contre l'IA.

use crate::display;
use crate::input::{Command, parse_command};
use farkle_core::advisor::{Decision, Risk, decide, farkle_probability, suggest};
use farkle_core::ai::{AiEvent, AiPlayer};
use farkle_core::combos::indices_for;
use farkle_core::dice::DiceRoller;
use farkle_core::game::{Game, GameError, Phase, RollOutcome};
use std::io::{self, BufRead, Write};

/// Nom de l'adversaire IA.
pub const AI_NAME: &str = "Hans";

#[derive(Debug, Clone, PartialEq)]
pub struct PlayConfig {
    pub target: u32,
    /// Aide activée au démarrage.
    pub help: bool,
    /// Style de jeu de l'IA.
    pub risk: Risk,
    pub player_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayResult {
    /// Partie terminée ; `winner` vaut 0 pour le joueur humain, 1 pour l'IA.
    Finished { winner: usize },
    /// Le joueur a quitté (commande `q` ou fin de l'entrée).
    Quit,
}

const HUMAN: usize = 0;

/// Joue une partie complète en lisant les commandes sur `input`.
pub fn run_play(
    config: &PlayConfig,
    roller: &mut dyn DiceRoller,
    input: &mut dyn BufRead,
    out: &mut dyn Write,
) -> io::Result<PlayResult> {
    let game = Game::new(&[&config.player_name, AI_NAME], config.target).map_err(io_error)?;
    let mut session = Session {
        game,
        ai: AiPlayer::new(config.risk),
        help: config.help,
        roller,
        input,
        out,
    };
    session.run()
}

struct Session<'a> {
    game: Game,
    ai: AiPlayer,
    help: bool,
    roller: &'a mut dyn DiceRoller,
    input: &'a mut dyn BufRead,
    out: &'a mut dyn Write,
}

enum TurnEnd {
    Continue,
    Quit,
}

impl Session<'_> {
    fn run(&mut self) -> io::Result<PlayResult> {
        writeln!(
            self.out,
            "=== Farkle : {} contre {} — premier à {} points ===",
            self.game.players()[HUMAN].name,
            AI_NAME,
            self.game.target()
        )?;
        writeln!(
            self.out,
            "Commandes : numéros des dés à garder (ex. 1 3 5), r = relancer, b = encaisser,\n\
             s = suivre le conseil, a = aide on/off, ? = règles, q = quitter."
        )?;
        loop {
            if let Some(winner) = self.game.winner() {
                let p = &self.game.players()[winner];
                writeln!(
                    self.out,
                    "\n*** Partie terminée : {} gagne avec {} points ! ***",
                    p.name, p.score
                )?;
                writeln!(self.out, "{}", display::scores(&self.game))?;
                return Ok(PlayResult::Finished { winner });
            }
            let end = if self.game.current() == HUMAN {
                self.human_turn()?
            } else {
                self.ai_turn()?
            };
            if let TurnEnd::Quit = end {
                writeln!(self.out, "\nAu revoir !")?;
                return Ok(PlayResult::Quit);
            }
        }
    }

    fn human_turn(&mut self) -> io::Result<TurnEnd> {
        let name = self.game.players()[HUMAN].name.clone();
        writeln!(
            self.out,
            "\n--- Tour de {name} --- {}",
            display::scores(&self.game)
        )?;
        if self.roll()? {
            return Ok(TurnEnd::Continue);
        }
        let mut show_help = true;
        loop {
            match self.game.phase() {
                Phase::Choosing => {
                    if self.help && show_help {
                        self.show_roll_help()?;
                    }
                    show_help = false;
                    let Some(command) = self.prompt("Dés à garder : ")? else {
                        return Ok(TurnEnd::Quit);
                    };
                    match command {
                        Command::Keep(indices) => self.keep(&indices)?,
                        Command::Suggest => {
                            let roll = self.game.last_roll().to_vec();
                            let score = self.game.players()[HUMAN].score;
                            let advice = suggest(
                                &roll,
                                self.game.turn_points(),
                                score,
                                self.game.target(),
                                Risk::NORMAL,
                            );
                            if let Some(best) = advice.best {
                                let indices =
                                    indices_for(&roll, &best.combo.dice).unwrap_or_default();
                                self.keep(&indices)?;
                            }
                        }
                        Command::Roll | Command::Bank => self.error(GameError::WrongPhase {
                            phase: Phase::Choosing,
                        })?,
                        Command::ToggleHelp => {
                            self.toggle_help()?;
                            show_help = true;
                        }
                        Command::Rules => self.rules()?,
                        Command::Quit => return Ok(TurnEnd::Quit),
                    }
                    if self.game.phase() == Phase::Kept {
                        show_help = true;
                    }
                }
                Phase::Kept => {
                    if self.help && show_help {
                        let n = self.game.dice_left();
                        let d = self.advised_decision();
                        writeln!(
                            self.out,
                            "{}",
                            display::decision(d, n, farkle_probability(n))
                        )?;
                    }
                    show_help = false;
                    let prompt = format!(
                        "[r] relancer {}, [b] encaisser {} pts : ",
                        display::dice_count(self.game.dice_left()),
                        self.game.turn_points()
                    );
                    let Some(command) = self.prompt(&prompt)? else {
                        return Ok(TurnEnd::Quit);
                    };
                    let command = match command {
                        Command::Suggest => match self.advised_decision() {
                            Decision::Reroll => Command::Roll,
                            Decision::Bank => Command::Bank,
                        },
                        c => c,
                    };
                    match command {
                        Command::Roll => {
                            if self.roll()? {
                                return Ok(TurnEnd::Continue);
                            }
                            show_help = true;
                        }
                        Command::Bank => {
                            let res = self.game.bank().map_err(io_error)?;
                            writeln!(
                                self.out,
                                "{name} encaisse {} pts → {} pts au total.",
                                res.banked, res.new_score
                            )?;
                            return Ok(TurnEnd::Continue);
                        }
                        Command::Keep(_) => {
                            self.error(GameError::WrongPhase { phase: Phase::Kept })?
                        }
                        Command::ToggleHelp => {
                            self.toggle_help()?;
                            show_help = true;
                        }
                        Command::Rules => self.rules()?,
                        Command::Quit => return Ok(TurnEnd::Quit),
                        Command::Suggest => unreachable!(),
                    }
                }
                Phase::Start | Phase::GameOver { .. } => return Ok(TurnEnd::Continue),
            }
        }
    }

    fn ai_turn(&mut self) -> io::Result<TurnEnd> {
        writeln!(
            self.out,
            "\n--- Tour de {AI_NAME} --- {}",
            display::scores(&self.game)
        )?;
        while let Some(event) = self
            .ai
            .step(&mut self.game, self.roller)
            .map_err(io_error)?
        {
            match &event {
                AiEvent::Rolled(dice) => {
                    writeln!(self.out, "{AI_NAME} lance : {}", display::dice_values(dice))?
                }
                AiEvent::Kept { dice, points } => {
                    writeln!(
                        self.out,
                        "{AI_NAME} garde {} (+{points} pts, {} pts au tour)",
                        display::dice_values(dice),
                        self.game.turn_points()
                    )?;
                    if self.game.is_hot_dice() {
                        writeln!(self.out, "Main chaude pour {AI_NAME} !")?;
                    }
                }
                AiEvent::Farkle { dice, lost_points } => {
                    writeln!(self.out, "{AI_NAME} lance : {}", display::dice_values(dice))?;
                    writeln!(self.out, "Farkle ! {AI_NAME} perd {lost_points} pts.")?;
                }
                AiEvent::Banked { points, .. } => {
                    let score = self.game.players()[1].score;
                    writeln!(
                        self.out,
                        "{AI_NAME} encaisse {points} pts → {score} pts au total."
                    )?;
                }
            }
            if matches!(event, AiEvent::Farkle { .. } | AiEvent::Banked { .. }) {
                break;
            }
        }
        Ok(TurnEnd::Continue)
    }

    /// Lance les dés du joueur humain ; renvoie `true` en cas de Farkle.
    fn roll(&mut self) -> io::Result<bool> {
        match self.game.roll(self.roller).map_err(io_error)? {
            RollOutcome::Scoring(dice) => {
                writeln!(self.out, "\n{}", display::roll(&dice))?;
                Ok(false)
            }
            RollOutcome::Farkle { dice, lost_points } => {
                writeln!(self.out, "\n{}", display::roll(&dice))?;
                writeln!(
                    self.out,
                    "Farkle ! Aucun dé ne marque, {lost_points} pts perdus."
                )?;
                Ok(true)
            }
        }
    }

    fn keep(&mut self, indices: &[usize]) -> io::Result<()> {
        match self.game.keep(indices) {
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
                Ok(())
            }
            Err(e) => self.error(e),
        }
    }

    fn show_roll_help(&mut self) -> io::Result<()> {
        let score = self.game.players()[HUMAN].score;
        let advice = suggest(
            self.game.last_roll(),
            self.game.turn_points(),
            score,
            self.game.target(),
            Risk::NORMAL,
        );
        writeln!(
            self.out,
            "Combinaisons possibles :\n{}",
            display::combos(&advice.combos)
        )?;
        if let Some(best) = &advice.best {
            writeln!(self.out, "{}", display::advice(best, self.game.last_roll()))?;
        }
        Ok(())
    }

    fn advised_decision(&self) -> Decision {
        decide(
            self.game.turn_points(),
            self.game.dice_left(),
            self.game.players()[HUMAN].score,
            self.game.target(),
            Risk::NORMAL,
        )
    }

    fn toggle_help(&mut self) -> io::Result<()> {
        self.help = !self.help;
        let state = if self.help {
            "activée"
        } else {
            "désactivée"
        };
        writeln!(self.out, "Aide {state}.")
    }

    fn rules(&mut self) -> io::Result<()> {
        writeln!(self.out, "\n{}", farkle_core::RULES_MD)
    }

    fn error(&mut self, e: impl std::fmt::Display) -> io::Result<()> {
        writeln!(self.out, "✗ {e}")
    }

    /// Affiche `text` et lit une commande valide ; `None` en fin d'entrée.
    fn prompt(&mut self, text: &str) -> io::Result<Option<Command>> {
        loop {
            write!(self.out, "{text}")?;
            self.out.flush()?;
            let mut line = String::new();
            if self.input.read_line(&mut line)? == 0 {
                writeln!(self.out)?;
                return Ok(None);
            }
            match parse_command(&line) {
                Ok(command) => return Ok(Some(command)),
                Err(e) => self.error(e)?,
            }
        }
    }
}

fn io_error(e: GameError) -> io::Error {
    io::Error::other(e)
}
