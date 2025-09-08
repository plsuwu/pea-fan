use core::fmt;
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, instrument, warn};

use super::commands;
use crate::{parsing::lexer::Lexer};

pub type ParseResult<T> = core::result::Result<T, ParseError>;

pub trait Parser: Send + Sync + fmt::Debug {
    fn parse(&self, raw: &str) -> ParseResult<IrcAst>;
}

const COMMAND_MENTION: &str = "@plss";
const COMMAND_TRIGGER: &str = "pisscount";

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("invalid message format: {0}")]
    InvalidFormat(String),

    #[error("cannot find a command in message - raw: {0}")]
    MissingCommand(String),

    #[error("invalid source format: {0}")]
    InvalidSource(String),

    #[error("cannot find required parameter for command '{command}': {param}")]
    MissingParameter { command: String, param: String },

    #[error("invalid numeric command: {0}")]
    InvalidNumeric(String),

    #[error("lex error: {0}")]
    LexerError(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrcAst {
    pub tags: HashMap<String, String>,
    pub source: Option<IrcSource>,
    pub command: commands::IrcCommand,
    pub raw_params: Vec<String>,
}

/// Represents the source part of an IRC message (`nick!user@host`)
#[derive(Debug, Clone, PartialEq)]
pub struct IrcSource {
    pub nick: String,
    pub user: Option<String>,
    pub host: Option<String>,
}

#[derive(Debug)]
pub struct IrcParser;

impl IrcParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, raw: &str) -> ParseResult<IrcAst> {
        let input = raw.trim_end_matches('\n').trim_end_matches('\r');

        let mut lexer = Lexer::new(input);

        let (tags, source, command_str, raw_params) = self.parse_structure(&mut lexer)?;
        let command = self.parse_command(&command_str, &raw_params, &tags, &source)?;

        Ok(IrcAst {
            tags,
            source,
            command,
            raw_params,
        })
    }

    #[instrument(skip(self, lexer))]
    fn parse_structure(
        &self,
        lexer: &mut Lexer,
    ) -> ParseResult<(
        HashMap<String, String>,
        Option<IrcSource>,
        String,
        Vec<String>,
    )> {
        let mut tags = HashMap::new();
        let mut source = None;

        // parse tags if present
        if lexer.peek_char() == Some('@') {
            lexer.next();
            tags = self.parse_tags(lexer)?;
            lexer.skip_whitespace();
        }

        // parse source if present
        if lexer.peek_char() == Some(':') {
            lexer.next();
            source = Some(self.parse_source(lexer)?);
        }

        lexer.skip_whitespace();

        // parse command
        let command = lexer
            .next_word()
            .ok_or(ParseError::MissingCommand(lexer.input.to_string()))?
            .to_uppercase();
        lexer.skip_whitespace();

        // parse params
        let params = self.parse_params(lexer);

        debug!(
            "parsed IRC structure: command={},tags={},params={}",
            command,
            tags.len(),
            params.len(),
        );

        Ok((tags, source, command, params))
    }

    #[instrument(skip(self, lexer))]
    fn parse_tags(&self, lexer: &mut Lexer) -> ParseResult<HashMap<String, String>> {
        let mut tags = HashMap::new();

        while let Some(key) = lexer.next_until(&['=', ';', ' ']) {
            let value = if lexer.peek_char() == Some('=') {
                lexer.next();
                lexer.next_until(&[';', ' ']).unwrap_or("").to_string()
            } else {
                String::new()
            };

            tags.insert(key.to_string(), value);
            if lexer.peek_char() == Some(';') {
                lexer.next();
            } else {
                break;
            }
        }

        debug!("parsed {} tags", tags.len());
        Ok(tags)
    }

    #[instrument(skip(self, lexer))]
    fn parse_source(&self, lexer: &mut Lexer) -> ParseResult<IrcSource> {
        let source_str = lexer
            .next_word()
            .ok_or_else(|| ParseError::InvalidSource("source is empty".to_string()))?;

        let nick_end = source_str.find('!').unwrap_or(source_str.len());
        let nick = source_str[..nick_end].to_string();

        if nick_end == source_str.len() {
            return Ok(IrcSource {
                nick,
                user: None,
                host: None,
            });
        }

        let user_host = &source_str[nick_end + 1..];
        let (user, host) = match user_host.find('@') {
            Some(pos) => (
                Some(user_host[..pos].to_string()),
                Some(user_host[pos + 1..].to_string()),
            ),
            None => (Some(user_host.to_string()), None),
        };

        debug!(
            "parsed source: nick={},user={:?},host={:?}",
            nick, user, host
        );
        Ok(IrcSource { nick, user, host })
    }

    #[instrument(skip(self, lexer))]
    fn parse_params(&self, lexer: &mut Lexer) -> Vec<String> {
        let mut params = Vec::new();

        while !lexer.is_eof() {
            if lexer.peek_char() == Some(':') {
                lexer.next();
                if let Some(trailing) = lexer.rest() {
                    params.push(trailing.to_string());
                }

                break;
            } else if let Some(param) = lexer.next_word() {
                params.push(param.to_string());
                lexer.skip_whitespace();
            } else {
                break;
            }
        }

        debug!("parsed {} params", params.len());
        params
    }

    #[instrument(skip(self))]
    fn parse_command(
        &self,
        command: &str,
        params: &[String],
        tags: &HashMap<String, String>,
        source: &Option<IrcSource>,
    ) -> ParseResult<commands::IrcCommand> {
        let user_info = self.extract_user_info(tags);

        match command {
            "PRIVMSG" => self.parse_privmsg(params, user_info),
            // "JOIN" => self.parse_join(params, user_info),
            // "PART" => self.parse_part(params, user_info),
            "NOTICE" => self.parse_notice(params),
            "PING" => self.parse_ping(params),
            "PONG" => self.parse_pong(params),
            "USERNOTICE" => self.parse_usernotice(params, tags, user_info),
            "USERSTATE" => self.parse_userstate(params, tags, user_info),
            "CLEARCHAT" => self.parse_clearchat(params, tags),
            "CLEARMSG" => self.parse_clearmsg(params, tags),
            cmd if cmd.chars().all(|c| c.is_ascii_digit()) => {
                if let Ok(code) = cmd.parse::<u16>() {
                    Ok(commands::IrcCommand::Numeric {
                        code,
                        params: params.to_vec(),
                    })
                } else {
                    Err(ParseError::InvalidNumeric(cmd.to_string()))
                }
            }
            _ => Ok(commands::IrcCommand::Unknown {
                command: command.to_string(),
                params: params.to_vec(),
            }),
        }
    }

    fn extract_user_info(&self, tags: &HashMap<String, String>) -> Option<commands::UserInfo> {
        if tags.is_empty() {
            return None;
        }

        let badges: Vec<String> = tags
            .get("badges")
            .map(|b| b.split(',').map(|s| s.to_string()).collect())
            .unwrap_or_default();

        let display_name = tags.get("display-name").cloned();
        let login = match display_name.clone() {
            Some(name) => Some(name.to_lowercase()),
            None => None,
        };

        Some(commands::UserInfo {
            user_id: tags.get("user-id").cloned(),
            login,
            display_name,
            color: tags.get("color").cloned(),
            badges: badges.clone(),
            subscriber: tags.get("subscriber").is_some(),
            moderator: tags.get("mod").is_some_and(|val| val == "1"),
            vip: tags.get("vip").is_some_and(|val| val == "1"),
            broadcaster: badges.iter().any(|b| b.starts_with("broadcaster")),
        })
    }

    // certain that per-command parsing like this could be pulled out but
    // at this point we are just balling
    //
    fn parse_privmsg(
        &self,
        params: &[String],
        user_info: Option<commands::UserInfo>,
    ) -> ParseResult<commands::IrcCommand> {
        let channel = params
            .get(0)
            .ok_or_else(|| ParseError::MissingParameter {
                command: "PRIVMSG".to_string(),
                param: "channel".to_string(),
            })?
            .clone();

        let message = params
            .get(1)
            .ok_or_else(|| ParseError::MissingParameter {
                command: "PRIVMSG".to_string(),
                param: "message".to_string(),
            })?
            .clone();

        Ok(commands::IrcCommand::PrivMsg {
            channel,
            message,
            user_info,
        })
    }

    fn parse_notice(&self, params: &[String]) -> ParseResult<commands::IrcCommand> {
        let target = params
            .get(0)
            .ok_or_else(|| ParseError::MissingParameter {
                command: "NOTICE".to_string(),
                param: "target".to_string(),
            })?
            .clone();

        let message = params
            .get(1)
            .ok_or_else(|| ParseError::MissingParameter {
                command: "NOTICE".to_string(),
                param: "message".to_string(),
            })?
            .clone();

        Ok(commands::IrcCommand::Notice { target, message })
    }

    fn parse_ping(&self, params: &[String]) -> ParseResult<commands::IrcCommand> {
        let server = params
            .get(0)
            .ok_or_else(|| ParseError::MissingParameter {
                command: "PING".to_string(),
                param: "server".to_string(),
            })?
            .clone();

        Ok(commands::IrcCommand::Ping { server })
    }

    fn parse_pong(&self, params: &[String]) -> ParseResult<commands::IrcCommand> {
        let server = params
            .get(0)
            .ok_or_else(|| ParseError::MissingParameter {
                command: "PONG".to_string(),
                param: "server".to_string(),
            })?
            .clone();

        Ok(commands::IrcCommand::Pong { server })
    }
    fn parse_usernotice(
        &self,
        params: &[String],
        tags: &HashMap<String, String>,
        user_info: Option<commands::UserInfo>,
    ) -> ParseResult<commands::IrcCommand> {
        let channel = params
            .get(0)
            .ok_or_else(|| ParseError::MissingParameter {
                command: "USERNOTICE".to_string(),
                param: "channel".to_string(),
            })?
            .clone();

        let message = params.get(1).cloned();
        let msg_id = tags.get("msg-id").cloned();

        Ok(commands::IrcCommand::UserNotice {
            channel,
            message,
            msg_id,
            user_info,
        })
    }

    fn parse_userstate(
        &self,
        params: &[String],
        tags: &HashMap<String, String>,
        user_info: Option<commands::UserInfo>,
    ) -> ParseResult<commands::IrcCommand> {
        let channel = params
            .get(0)
            .ok_or_else(|| ParseError::MissingParameter {
                command: "USERSTATE".to_string(),
                param: "channel".to_string(),
            })?
            .clone();

        let message = params.get(1).cloned();
        let msg_id = tags.get("msg-id").cloned();

        Ok(commands::IrcCommand::UserNotice {
            channel,
            message,
            msg_id,
            user_info,
        })
    }

    fn parse_clearchat(
        &self,
        params: &[String],
        tags: &HashMap<String, String>,
    ) -> ParseResult<commands::IrcCommand> {
        let channel = params
            .get(0)
            .ok_or_else(|| ParseError::MissingParameter {
                command: "USERNOTICE".to_string(),
                param: "channel".to_string(),
            })?
            .clone();

        let target_user = params.get(1).cloned();
        let duration = tags.get("ban-duration").and_then(|d| d.parse().ok());

        Ok(commands::IrcCommand::ClearChat {
            channel,
            target_user,
            duration,
        })
    }

    fn parse_clearmsg(
        &self,
        params: &[String],
        tags: &HashMap<String, String>,
    ) -> ParseResult<commands::IrcCommand> {
        let channel = params
            .get(0)
            .ok_or_else(|| ParseError::MissingParameter {
                command: "CLEARMSG".to_string(),
                param: "channel".to_string(),
            })?
            .clone();

        let target_msg_id = tags
            .get("target-msg-id")
            .ok_or_else(|| ParseError::MissingParameter {
                command: "CLEARMSG".to_string(),
                param: "target-msg-id".to_string(),
            })?
            .clone();

        Ok(commands::IrcCommand::ClearMsg {
            channel,
            target_msg_id,
        })
    }
}

impl Parser for IrcParser {
    fn parse(&self, raw: &str) -> ParseResult<IrcAst> {
        self.parse(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_privmsg() {
        let input = r#"@badge-info=;badges=broadcaster/1;color=#FFBEDF;display-name=plss;user-id=103033809 :plss!plss@plss.tmi.twitch.tv PRIVMSG #plss :Hello world"#;

        let parser = IrcParser::new();
        let ast = parser.parse(input);

        match ast.unwrap().command {
            commands::IrcCommand::PrivMsg {
                channel,
                message,
                user_info,
            } => {
                assert_eq!(channel, "#plss");
                assert_eq!(message, "Hello world");
                assert!(user_info.is_some());
                let user = user_info.unwrap();
                assert_eq!(user.display_name, Some("plss".to_string()));
                assert_eq!(user.user_id, Some("103033809".to_string()));
                assert!(user.broadcaster);
            }

            _ => panic!("expected PRIVMSG command"),
        }
    }

    #[test]
    fn test_parse_numeric() {
        let input = ":server.example.com 001 nick :Welcome to the network";
        let parser = IrcParser::new();
        let ast = parser.parse(input).unwrap();

        match ast.command {
            commands::IrcCommand::Numeric { code, params } => {
                assert_eq!(code, 1);
                assert_eq!(params, vec!["nick", "Welcome to the network"]);
            }
            _ => panic!("Expected Numeric command"),
        }
    }

    #[test]
    fn test_parse_unknown_command() {
        let input = ":server UNKNOWNCMD param1 param2";
        let parser = IrcParser::new();
        let ast = parser.parse(input).unwrap();

        match ast.command {
            commands::IrcCommand::Unknown { command, params } => {
                assert_eq!(command, "UNKNOWNCMD");
                assert_eq!(params, vec!["param1", "param2"]);
            }
            _ => panic!("Expected Unknown command"),
        }
    }
}
