//! Arguments de la ligne de commande.

use crate::play::PlayConfig;
use crate::train::TrainConfig;
use clap::{Args, Parser, Subcommand, ValueEnum};
use farkle_core::advisor::Risk;
use farkle_core::game::DEFAULT_TARGET;

#[derive(Debug, Parser)]
#[command(
    name = "farkle",
    version,
    about = "Jeu de dés Farkle (règles de Kingdom Come: Deliverance)",
    after_help = "Les règles complètes s'affichent avec `farkle --help` ou `farkle rules`.",
    after_long_help = farkle_core::RULES_MD
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand, PartialEq)]
pub enum Commands {
    /// Jouer une partie contre l'IA (commande par défaut).
    Play(PlayArgs),
    /// Mode entraîner : saisissez vos lancers, le conseiller vous guide.
    Train(TrainArgs),
    /// Afficher les règles du jeu.
    Rules,
}

#[derive(Debug, Args, PartialEq)]
pub struct TrainArgs {
    /// Score à atteindre.
    #[arg(short, long, default_value_t = DEFAULT_TARGET,
          value_parser = clap::value_parser!(u32).range(1..))]
    pub target: u32,
    /// Votre score actuel (pour s'entraîner sur une fin de partie).
    #[arg(short, long, default_value_t = 0)]
    pub score: u32,
}

impl TrainArgs {
    pub fn config(&self) -> TrainConfig {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OnOff {
    On,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum AiLevel {
    Prudente,
    Normale,
    Audacieuse,
}

#[derive(Debug, Args, PartialEq)]
pub struct PlayArgs {
    /// Score à atteindre pour gagner.
    #[arg(short, long, default_value_t = DEFAULT_TARGET,
          value_parser = clap::value_parser!(u32).range(1..))]
    pub target: u32,
    /// Aide : affiche les combinaisons possibles et un conseil (touche `a` en jeu).
    #[arg(long, value_enum, default_value_t = OnOff::Off)]
    pub aide: OnOff,
    /// Style de jeu de l'IA.
    #[arg(long, value_enum, default_value_t = AiLevel::Normale)]
    pub ia: AiLevel,
    /// Votre nom.
    #[arg(long, default_value = "Henry")]
    pub nom: String,
    /// Graine des dés (tests).
    #[arg(long, hide = true)]
    pub seed: Option<u64>,
}

impl Default for PlayArgs {
    fn default() -> Self {
        PlayArgs {
            target: DEFAULT_TARGET,
            aide: OnOff::Off,
            ia: AiLevel::Normale,
            nom: "Henry".into(),
            seed: None,
        }
    }
}

impl PlayArgs {
    pub fn config(&self) -> PlayConfig {
        PlayConfig {
            target: self.target,
            help: self.aide == OnOff::On,
            risk: match self.ia {
                AiLevel::Prudente => Risk::CAUTIOUS,
                AiLevel::Normale => Risk::NORMAL,
                AiLevel::Audacieuse => Risk::BOLD,
            },
            player_name: self.nom.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("farkle").chain(args.iter().copied()))
    }

    fn play(args: &[&str]) -> PlayArgs {
        match parse(args).unwrap().command {
            Some(Commands::Play(p)) => p,
            other => panic!("attendu play, obtenu {other:?}"),
        }
    }

    #[test]
    fn no_subcommand_is_allowed() {
        assert_eq!(parse(&[]).unwrap().command, None);
    }

    #[test]
    fn play_defaults() {
        let p = play(&["play"]);
        assert_eq!(p, PlayArgs::default());
        assert_eq!(p.target, 4000);
        assert_eq!(p.aide, OnOff::Off);
        assert_eq!(p.ia, AiLevel::Normale);
        assert_eq!(p.seed, None);
    }

    #[test]
    fn play_options() {
        let p = play(&[
            "play",
            "--target",
            "2000",
            "--aide",
            "on",
            "--ia",
            "audacieuse",
            "--nom",
            "Theresa",
            "--seed",
            "7",
        ]);
        assert_eq!(p.target, 2000);
        assert_eq!(p.aide, OnOff::On);
        assert_eq!(p.ia, AiLevel::Audacieuse);
        assert_eq!(p.nom, "Theresa");
        assert_eq!(p.seed, Some(7));
        assert_eq!(play(&["play", "-t", "500"]).target, 500);
    }

    #[test]
    fn invalid_options_are_rejected() {
        assert!(parse(&["play", "--target", "0"]).is_err());
        assert!(parse(&["play", "--target", "abc"]).is_err());
        assert!(parse(&["play", "--aide", "peut-etre"]).is_err());
    }

    #[test]
    fn train_subcommand() {
        let Some(Commands::Train(t)) = parse(&["train"]).unwrap().command else {
            panic!("attendu train");
        };
        assert_eq!(
            t,
            TrainArgs {
                target: 4000,
                score: 0
            }
        );
        assert_eq!(
            t.config(),
            TrainConfig {
                target: 4000,
                score: 0
            }
        );
        let Some(Commands::Train(t)) = parse(&["train", "--target", "2000", "--score", "1500"])
            .unwrap()
            .command
        else {
            panic!("attendu train");
        };
        assert_eq!(
            t.config(),
            TrainConfig {
                target: 2000,
                score: 1500
            }
        );
        assert!(parse(&["train", "--target", "0"]).is_err());
    }

    #[test]
    fn rules_subcommand() {
        assert_eq!(parse(&["rules"]).unwrap().command, Some(Commands::Rules));
    }

    #[test]
    fn config_maps_arguments() {
        let c = play(&[
            "play", "--target", "2000", "--aide", "on", "--ia", "prudente",
        ])
        .config();
        assert_eq!(c.target, 2000);
        assert!(c.help);
        assert_eq!(c.risk, Risk::CAUTIOUS);
        assert_eq!(c.player_name, "Henry");
        assert_eq!(
            play(&["play", "--ia", "audacieuse"]).config().risk,
            Risk::BOLD
        );
        assert!(!play(&["play"]).config().help);
    }
}
