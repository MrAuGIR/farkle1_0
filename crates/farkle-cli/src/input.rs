//! Lecture des commandes saisies par le joueur.

use farkle_core::dice::Die;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Positions (base 0) des dés à garder.
    Keep(Vec<usize>),
    Roll,
    Bank,
    /// Suivre le conseil du conseiller.
    Suggest,
    ToggleHelp,
    Rules,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputError {
    Empty,
    Unknown(String),
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputError::Empty => write!(f, "vous n'avez rien saisi"),
            InputError::Unknown(s) => write!(f, "saisie incomprise : « {s} »"),
        }
    }
}

/// Analyse une ligne saisie. Les numéros de dés commencent à 1 ; ils peuvent être
/// séparés par des espaces ou des virgules, ou collés (`135`).
pub fn parse_command(line: &str) -> Result<Command, InputError> {
    let text = line.trim().to_lowercase();
    let command = match text.as_str() {
        "" => return Err(InputError::Empty),
        "r" | "relancer" => Command::Roll,
        "b" | "e" | "encaisser" => Command::Bank,
        "s" | "conseil" => Command::Suggest,
        "a" | "aide" => Command::ToggleHelp,
        "?" | "regles" | "règles" => Command::Rules,
        "q" | "quitter" => Command::Quit,
        _ => {
            let mut indices = Vec::new();
            for token in text.split(|c: char| c.is_whitespace() || c == ',') {
                if token.is_empty() {
                    continue;
                }
                if !token.chars().all(|c| ('1'..='6').contains(&c)) {
                    return Err(InputError::Unknown(token.to_string()));
                }
                indices.extend(token.bytes().map(|b| (b - b'1') as usize));
            }
            Command::Keep(indices)
        }
    };
    Ok(command)
}

/// Erreur de saisie d'un lancer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollInputError {
    Empty,
    InvalidValue(String),
    WrongCount { expected: usize, got: usize },
}

impl fmt::Display for RollInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = f;
        todo!()
    }
}

/// Analyse un lancer saisi à la main : valeurs de 1 à 6, séparées par des espaces
/// ou des virgules, ou collées (`155326`). Il faut exactement `expected` dés.
pub fn parse_roll(line: &str, expected: usize) -> Result<Vec<Die>, RollInputError> {
    let _ = (line, expected);
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords() {
        assert_eq!(parse_command("r"), Ok(Command::Roll));
        assert_eq!(parse_command(" R \n"), Ok(Command::Roll));
        assert_eq!(parse_command("relancer"), Ok(Command::Roll));
        assert_eq!(parse_command("b"), Ok(Command::Bank));
        assert_eq!(parse_command("e"), Ok(Command::Bank));
        assert_eq!(parse_command("encaisser"), Ok(Command::Bank));
        assert_eq!(parse_command("s"), Ok(Command::Suggest));
        assert_eq!(parse_command("a"), Ok(Command::ToggleHelp));
        assert_eq!(parse_command("aide"), Ok(Command::ToggleHelp));
        assert_eq!(parse_command("?"), Ok(Command::Rules));
        assert_eq!(parse_command("règles"), Ok(Command::Rules));
        assert_eq!(parse_command("q"), Ok(Command::Quit));
        assert_eq!(parse_command("quitter"), Ok(Command::Quit));
    }

    #[test]
    fn dice_positions_are_one_based() {
        assert_eq!(parse_command("1"), Ok(Command::Keep(vec![0])));
        assert_eq!(parse_command("1 3 5"), Ok(Command::Keep(vec![0, 2, 4])));
        assert_eq!(parse_command("1,3, 5"), Ok(Command::Keep(vec![0, 2, 4])));
        assert_eq!(parse_command("135"), Ok(Command::Keep(vec![0, 2, 4])));
    }

    #[test]
    fn invalid_input() {
        assert_eq!(parse_command(""), Err(InputError::Empty));
        assert_eq!(parse_command("   "), Err(InputError::Empty));
        assert_eq!(parse_command("x"), Err(InputError::Unknown("x".into())));
        assert_eq!(parse_command("0"), Err(InputError::Unknown("0".into())));
        assert_eq!(parse_command("1 z"), Err(InputError::Unknown("z".into())));
        assert_eq!(parse_command("17"), Err(InputError::Unknown("17".into())));
    }

    fn values(dice: Vec<Die>) -> Vec<u8> {
        dice.into_iter().map(|d| d.value()).collect()
    }

    #[test]
    fn roll_values() {
        assert_eq!(
            parse_roll("1 5 5 3 2 6", 6).map(values),
            Ok(vec![1, 5, 5, 3, 2, 6])
        );
        assert_eq!(
            parse_roll("155326\n", 6).map(values),
            Ok(vec![1, 5, 5, 3, 2, 6])
        );
        assert_eq!(parse_roll("1,5, 5", 3).map(values), Ok(vec![1, 5, 5]));
    }

    #[test]
    fn roll_errors() {
        assert_eq!(parse_roll("  ", 6), Err(RollInputError::Empty));
        assert_eq!(
            parse_roll("1 2 7 4 5 6", 6),
            Err(RollInputError::InvalidValue("7".into()))
        );
        assert_eq!(
            parse_roll("1 2 x", 3),
            Err(RollInputError::InvalidValue("x".into()))
        );
        assert_eq!(
            parse_roll("1 2 3", 6),
            Err(RollInputError::WrongCount {
                expected: 6,
                got: 3
            })
        );
        assert!(
            RollInputError::WrongCount {
                expected: 6,
                got: 3
            }
            .to_string()
            .contains("6 dés attendus")
        );
        assert!(
            RollInputError::InvalidValue("7".into())
                .to_string()
                .contains("« 7 »")
        );
    }

    #[test]
    fn error_messages() {
        assert!(InputError::Empty.to_string().contains("rien"));
        assert!(
            InputError::Unknown("x".into())
                .to_string()
                .contains("« x »")
        );
    }
}
