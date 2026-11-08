//! Lecture des commandes saisies par le joueur.

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
        let _ = f;
        todo!()
    }
}

/// Analyse une ligne saisie. Les numéros de dés commencent à 1 ; ils peuvent être
/// séparés par des espaces ou des virgules, ou collés (`135`).
pub fn parse_command(line: &str) -> Result<Command, InputError> {
    let _ = line;
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
