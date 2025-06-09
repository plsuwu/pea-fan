use crate::parser::lexer::Lexer;
use std::collections::HashMap;

/// Represents the result of parsing an IRC message
#[derive(Debug, Clone, PartialEq)]
pub struct IrcMessage<'a> {
    pub tags: HashMap<&'a str, &'a str>,
    pub source: Option<IrcSource<'a>>,
    pub command: &'a str,
    pub params: Vec<&'a str>,
}

/// Represents the source part of an IRC message (`nick!user@host`)
#[derive(Debug, Clone, PartialEq)]
pub struct IrcSource<'a> {
    pub nick: &'a str,
    pub user: Option<&'a str>,
    pub host: Option<&'a str>,
}

/// Specific data to extract from messages sent with the `PRIVMSG` command
#[derive(Debug, Clone)]
pub struct ChatData<'a> {
    pub channel: &'a str,
    pub display_name: &'a str,
    pub user_id: &'a str,
    pub color: Option<&'a str>,
    pub message: &'a str,
}

/// Parser errors
#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidFormat,
    MissingCommand,
    InvalidSource,
    MissingRequiredTag(&'static str),
    NotPrivmsg,
}

/// Main parser struct.
///
/// > Stateless
pub struct IrcParser;

impl IrcParser {
    pub fn new() -> Self {
        Self
    }

    /// Parses a raw incoming data from a twitch IRC message into its constituents (an AST, if you
    /// will).
    pub fn parse_socket_data<'a>(&'a self, input: &'a str) -> Result<IrcMessage<'a>, ParseError> {
        let input = input.trim_end_matches('\n').trim_end_matches('\r');
        let mut lexer = Lexer::new(input);
        self.read_notification(&mut lexer)
    }

    pub fn get_chat<'a>(&'a self, message: &IrcMessage<'a>) -> Result<ChatData<'a>, ParseError> {
        if message.command != "PRIVMSG" {
            return Err(ParseError::NotPrivmsg);
        }

        let channel = message.params.get(0).ok_or(ParseError::InvalidFormat)?;
        let message_content = message.params.get(1).ok_or(ParseError::InvalidFormat)?;
        let display_name = message
            .tags
            .get("display-name")
            .ok_or(ParseError::MissingRequiredTag("display-name"))?;
        let user_id = message
            .tags
            .get("user-id")
            .ok_or(ParseError::MissingRequiredTag("user-id"))?;
        let color = message.tags.get("color").copied();

        Ok(ChatData {
            channel,
            display_name,
            user_id,
            color,
            message: message_content,
        })
    }

    pub fn get_channel<'a>(&'a self, message: &IrcMessage<'a>) -> Result<&'a str, ParseError> {
        Ok(message.params.get(0).ok_or(ParseError::InvalidFormat)?)
    }

    pub fn read_notification<'a>(
        &'a self,
        lexer: &mut Lexer<'a>,
    ) -> Result<IrcMessage<'a>, ParseError> {
        let mut tags = HashMap::new();
        let mut source = None;

        if lexer.peek_char() == Some('@') {
            lexer.next();
            tags = self.read_tags(lexer)?;
            lexer.skip_whitespace();
        }

        if lexer.peek_char() == Some(':') {
            lexer.next();
            source = Some(self.read_source(lexer)?);
            lexer.skip_whitespace();
        }

        let command = lexer.next_word().ok_or(ParseError::MissingCommand)?;
        lexer.skip_whitespace();

        let params = self.read_params(lexer);

        Ok(IrcMessage {
            tags,
            source,
            command,
            params,
        })
    }

    pub fn read_tags<'a>(
        &'a self,
        lexer: &mut Lexer<'a>,
    ) -> Result<HashMap<&'a str, &'a str>, ParseError> {
        let mut tags = HashMap::new();

        while let Some(key) = lexer.next_until(&['=', ';', ' ']) {
            if lexer.peek_char() == Some('=') {
                lexer.next();
                let value = lexer.next_until(&[';', ' ']);
                tags.insert(key, value.unwrap_or(""));
            } else {
                tags.insert(key, "");
            }

            if lexer.peek_char() == Some(';') {
                lexer.next();
            } else {
                break;
            }
        }

        Ok(tags)
    }

    pub fn read_source<'a>(&'a self, lexer: &mut Lexer<'a>) -> Result<IrcSource<'a>, ParseError> {
        let source_str = lexer.next_word().ok_or(ParseError::InvalidSource)?;
        let nick_end = source_str.find('!').unwrap_or(source_str.len());
        let nick = &source_str[..nick_end];

        if nick_end == source_str.len() {
            // ONLY `nick` present; no `user@host`.
            //
            // I don't think this function is called on websocket messages that this would apply to
            // but its just in case.
            return Ok(IrcSource {
                nick,
                user: None,
                host: None,
            });
        }

        let user_host = &source_str[nick_end + 1..];
        let at_pos = user_host.find('@');

        let (user, host) = match at_pos {
            Some(pos) => (Some(&user_host[..pos]), Some(&user_host[pos + 1..])),
            None => (Some(user_host), None),
        };

        Ok(IrcSource { nick, user, host })
    }

    pub fn read_params<'a>(&self, lexer: &mut Lexer<'a>) -> Vec<&'a str> {
        let mut params = Vec::new();
        while !lexer.is_eof() {
            if lexer.peek_char() == Some(':') {
                lexer.next();
                if let Some(trailing) = lexer.rest() {
                    params.push(trailing);
                }
                break;
            } else {
                if let Some(param) = lexer.next_word() {
                    params.push(param);
                    lexer.skip_whitespace();
                } else {
                    break;
                }
            }
        }

        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_privmsg() {
        let input = r#"@badge-info=;badges=broadcaster/1,twitch-recap-2023/1;client-nonce=00000000000000000000000000000000;color=#FFBEDF;display-name=plss;emotes=;first-msg=0;flags=;id=77ac96fb-34c4-4494-b4a2-83873aecb333;mod=0;returning-chatter=0;room-id=103033809;subscriber=0;tmi-sent-ts=1749208156695;turbo=0;user-id=103033809;user-type= :plss!plss@plss.tmi.twitch.tv PRIVMSG #plss :eeeeeeeee"#;

        let parser = IrcParser::new();
        let message = parser.parse_socket_data(input).unwrap();

        assert_eq!(message.command, "PRIVMSG");
        assert_eq!(message.tags.get("display-name"), Some(&"plss"));
        assert_eq!(message.tags.get("user-id"), Some(&"103033809"));
        assert_eq!(message.tags.get("color"), Some(&"#FFBEDF"));

        let privmsg_data = parser.get_chat(&message).unwrap();
        assert_eq!(privmsg_data.channel, "#plss");
        assert_eq!(privmsg_data.display_name, "plss");
        assert_eq!(privmsg_data.user_id, "103033809");
        assert_eq!(privmsg_data.color, Some("#FFBEDF"));
        assert_eq!(privmsg_data.message, "eeeeeeeee");
    }

    #[test]
    fn test_parse_simple_message() {
        let input = "PRIVMSG #test :Hello world";
        let parser = IrcParser::new();
        let message = parser.parse_socket_data(input).unwrap();

        assert_eq!(message.command, "PRIVMSG");
        assert_eq!(message.params, vec!["#test", "Hello world"]);
    }

    #[test]
    fn test_parse_non_privmsg() {
        let input = r#"@badge-info=subscriber/8;badges=vip/1,subscriber/6,twitch-recap-2023/1;color=#FFBEDF;display-name=plss;emote-sets=0,793,8231,19194,876326,935873,1232221,300374282,300380967,301464307,302029931,302512232,302792003,303148195,323827706,326691955,334292379,344011590,345474279,366226437,387726113,390658648,392630734,409842248,415514593,416564655,418871744,427477847,435300334,440880357,441442142,454806117,459526139,460760505,468360508,470888728,477339272,484906151,496680382,537206155,1306162089,1911289880,15a031d7-8783-468d-99f2-f5832a08d7c0,35b067de-37af-4430-99b0-6591201aa8c7,398cca87-aea0-4fd7-b29d-0613ab67320a,3c5be0d3-3eb7-4e96-93e2-44ac38b40819,5263b216-dab4-47e5-bc72-94fa093f6906,560c6a32-134b-4340-8185-a3e99e87237b,7c63ed2d-8e7e-4525-85a4-51e0b78ad0e3,7d68dda4-5170-442a-8dd8-9e5eb1ed8d27,acccd20c-25a2-497f-8265-59b890b61d62,bc112c6f-a202-43c2-b144-2c93e20cc5a2,bd70e005-1bb7-4879-b910-67779c22ccf9,bd70e005-1bb7-4879-b910-67779c22ccf9,c64918b8-0ebd-41c9-b153-300ca3491aa8,c9a93654-bae4-439e-ac62-0d69ecad1786,d31f1a6c-72ee-4aab-9bd3-7bf7f1d037bc,d92eb0a5-4f2b-43f6-892d-bc398567a0e1,e3ac0383-f23b-4dcf-ad65-d5a7ee1b26cb,ebe796ee-3c56-472c-922a-af70aeeff96d,ed963b8b-9b40-4d60-ba5b-f68985586441;mod=0;subscriber=1;user-type= :tmi.twitch.tv USERSTATE #sleepiebug @emote-only=0;followers-only=-1;r9k=0;room-id=610533290;slow=0;subs-only=0 :tmi.twitch.tv ROOMSTATE #sleepiebug"#;
        let parser = IrcParser::new();

        let message = parser.parse_socket_data(input).unwrap();

        println!("{:?}", message);
    }
}
