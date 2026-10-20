use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Reload {
        library: PathBuf,

        config_dir: PathBuf,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("empty command line")]
    Empty,

    #[error("unknown command verb: {0:?}")]
    UnknownVerb(String),

    #[error("command {verb:?} is missing its argument")]
    MissingArgument { verb: &'static str },
}

impl Command {
    /// Serializes the command into its single-line wire form (no newline).
    pub fn encode(&self) -> String {
        match self {
            Command::Reload {
                library,
                config_dir,
            } => format!("reload {}\t{}", library.display(), config_dir.display()),
        }
    }

    /// Parses one wire line back into a [`Command`].
    ///
    /// The argument runs to the end of the line, so paths containing spaces
    /// survive the round trip.
    pub fn decode(line: &str) -> Result<Self, ProtocolError> {
        let line = line.trim();
        if line.is_empty() {
            return Err(ProtocolError::Empty);
        }
        let (verb, arg) = match line.split_once(' ') {
            Some((verb, arg)) => (verb, arg.trim()),
            None => (line, ""),
        };
        match verb {
            "reload" => {
                if arg.is_empty() {
                    return Err(ProtocolError::MissingArgument { verb: "reload" });
                }
                // The library path and the config directory are tab-separated;
                // an old-style line without a tab still decodes (no config dir).
                let (library, config_dir) = match arg.split_once('\t') {
                    Some((library, config_dir)) => {
                        (PathBuf::from(library), PathBuf::from(config_dir))
                    }
                    None => (PathBuf::from(arg), PathBuf::new()),
                };
                Ok(Command::Reload {
                    library,
                    config_dir,
                })
            }
            other => Err(ProtocolError::UnknownVerb(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn reload_round_trips() {
        let command = Command::Reload {
            library: PathBuf::from("/tmp/libclient.so"),
            config_dir: PathBuf::from("/home/work"),
        };
        let wire = command.encode();
        assert_eq!(wire, "reload /tmp/libclient.so\t/home/work");
        assert_eq!(Command::decode(&wire), Ok(command));
    }

    #[test]
    fn reload_paths_with_spaces_survive_the_round_trip() {
        let command = Command::Reload {
            library: PathBuf::from("/home/My Games/libclient.so"),
            config_dir: PathBuf::from("/home/My Games/cfg dir"),
        };
        assert_eq!(Command::decode(&command.encode()), Ok(command));
    }

    #[test]
    fn decode_ignores_surrounding_whitespace() {
        assert_eq!(
            Command::decode("  reload /a/b.so\t/c\n"),
            Ok(Command::Reload {
                library: PathBuf::from("/a/b.so"),
                config_dir: PathBuf::from("/c"),
            }),
        );
    }

    #[test]
    fn reload_without_a_config_dir_decodes_with_an_empty_one() {
        assert_eq!(
            Command::decode("reload /a/b.so"),
            Ok(Command::Reload {
                library: PathBuf::from("/a/b.so"),
                config_dir: PathBuf::new(),
            }),
        );
    }

    #[test]
    fn an_empty_line_is_rejected() {
        assert_eq!(Command::decode("   "), Err(ProtocolError::Empty));
    }

    #[test]
    fn reload_without_an_argument_is_rejected() {
        assert_eq!(
            Command::decode("reload"),
            Err(ProtocolError::MissingArgument { verb: "reload" }),
        );
    }

    #[test]
    fn an_unknown_verb_is_rejected() {
        assert_eq!(
            Command::decode("frobnicate x"),
            Err(ProtocolError::UnknownVerb("frobnicate".to_string())),
        );
    }
}
