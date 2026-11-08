use clap::Parser;
use farkle_cli::cli::{Cli, Commands, PlayArgs};
use farkle_cli::play::{PlayResult, run_play};
use farkle_core::dice::RandomRoller;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Commands::Play(PlayArgs::default())) {
        Commands::Rules => {
            println!("{}", farkle_core::RULES_MD);
            ExitCode::SUCCESS
        }
        Commands::Play(args) => {
            let seed = args.seed.unwrap_or_else(|| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(0)
            });
            let mut roller = RandomRoller::from_seed(seed);
            let stdin = io::stdin();
            let mut input = stdin.lock();
            let mut out = io::stdout();
            match run_play(&args.config(), &mut roller, &mut input, &mut out) {
                Ok(PlayResult::Finished { .. } | PlayResult::Quit) => {
                    let _ = out.flush();
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("erreur : {e}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}
