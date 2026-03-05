//! Pure functions for parsing and transforming IRC `Message` data into domain types

use irc::proto::Command;

use crate::irc::commands::{IncomingMessage, IrcTags};

pub fn is_pong(msg: &irc::proto::Message) -> bool {
    match &msg.command {
        Command::PONG(_, _) => true,
        _ => false,
    }
}

pub fn parse_incoming(msg: &irc::proto::Message) -> Option<IncomingMessage> {
    match &msg.command {
        Command::PRIVMSG(channel, content) => {
            let tags = parse_tags(msg, channel);
            let message = content.to_string();

            Some(IncomingMessage::Privmsg {
                tags,
                text: message,
            })
        }

        _ => {
            tracing::info!(
                command = ?msg.command,
                tags = ?msg.tags,
                prefix = ?msg.prefix,
                "unknown_message_type"
            );
            None
        }
    }
}

pub fn parse_tags(msg: &irc::proto::Message, channel: &str) -> IrcTags {
    let mut result = IrcTags {
        channel_name: channel.rsplit('#').next().unwrap_or("UNKNOWN").to_string(),
        ..Default::default()
    };

    for tag in msg.tags.clone().unwrap_or_default() {
        match (tag.0.as_str(), tag.1) {
            ("room-id", Some(room_id)) => result.channel_id = room_id,
            ("display-name", Some(name)) => result.user_login = name.to_lowercase(),
            ("user-id", Some(user_id)) => result.user_id = user_id,
            ("color", Some(color)) => result.color = color,
            ("id", Some(msg_id)) => result.msg_id = msg_id,
            _ => (),
        }
    }

    result
}

pub fn format_username(msg_parts: Vec<&str>) -> String {
    if msg_parts.len() != 1 {
        return format!("{}'s", msg_parts[1]);
    }

    "your".to_string()
}

pub fn is_counter_user(msg: &irc::proto::Message, counter_user: &str) -> bool {
    matches!(
        &msg.prefix,
        Some(irc::proto::Prefix::Nickname(nick, _, _))
            if nick.eq_ignore_ascii_case(counter_user)
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use irc::proto::message::Tag;
    use irc::proto::{Command, Message, Prefix};

    fn make_privmsg(channel: &str, text: &str, tags: Vec<Tag>) -> Message {
        Message {
            tags: Some(tags),
            prefix: Some(Prefix::Nickname(
                "someuser".into(),
                "someuser".into(),
                "someuser.tmi.twitch.tv".into(),
            )),
            command: Command::PRIVMSG(channel.into(), text.into()),
        }
    }

    /// Tags from the [Twitch IRC Reference] page
    ///
    /// [Twitch IRC Reference]: https://dev.twitch.tv/docs/chat/irc/
    fn standard_tags() -> Vec<Tag> {
        vec![
            Tag("badges".into(), Some("broadcaster/1".into())),
            Tag("client-nonce".into(), Some("examplenonce123".into())),
            Tag("room-id".into(), Some("123456789".into())),
            Tag("display-name".into(), Some("Example".into())),
            Tag("color".into(), Some("#0000FF".into())),
            Tag("emotes".into(), Some("62835:0-10".into())),
            Tag("first-msg".into(), Some("0".into())),
            Tag("id".into(), Some("example-message-uuid".into())),
            Tag("mod".into(), Some("0".into())),
            Tag("subscriber".into(), Some("0".into())),
            Tag("tmi-sent-ts".into(), Some("1642696567751".into())),
            Tag("turbo".into(), Some("0".into())),
            Tag("user-id".into(), Some("123456789".into())),
            Tag("user-type".into(), None),
        ]
    }

    #[test]
    fn parse_tags_extracts_all_fields() {
        let msg = make_privmsg("#testchannel", "test", standard_tags());
        let tags = parse_tags(&msg, "#testchannel");

        assert_eq!(tags.channel_name, "testchannel");
        assert_eq!(tags.channel_id, "123456789");
        assert_eq!(tags.user_id, "123456789");
        assert_eq!(tags.color, "#0000FF");
        assert_eq!(tags.msg_id, "example-message-uuid");
    }

    #[test]
    fn parse_tags_handles_missing_tags() {
        let msg = make_privmsg("#testchannel", "test", vec![]);
        let tags = parse_tags(&msg, "#testchannel");

        assert_eq!(tags.channel_name, "testchannel");
        assert_eq!(tags.user_id, "");
        assert_eq!(tags.channel_id, "");
    }

    #[test]
    fn parse_tags_strips_channel_prefix() {
        let msg = make_privmsg("#testchannel", "test", vec![]);
        let tags = parse_tags(&msg, "#testchannel");

        assert_eq!(tags.channel_name, "testchannel");

        // edge case: PROBABLY won't occur but who could say
        let tags = parse_tags(&msg, "unprefixed");
        assert_eq!(tags.channel_name, "unprefixed");
    }

    #[test]
    fn parse_incoming_returns_privmsg() {
        let msg = make_privmsg("#testchannel", "test", standard_tags());
        let parsed = parse_incoming(&msg);

        assert!(matches!(
            parsed,
            Some(IncomingMessage::Privmsg { ref text, .. }) if text == "test"
        ));
    }

    #[test]
    fn parse_incoming_ignores_ping() {
        let msg = Message {
            tags: None,
            prefix: None,
            command: Command::PING("tmi.twitch.tv".into(), None),
        };

        assert!(parse_incoming(&msg).is_none());
    }

    #[test]
    fn is_pong_detects_pong_frame() {
        let pong = Message {
            tags: None,
            prefix: None,
            command: Command::PONG("tmi.twitch.tv".into(), None),
        };

        let not_pong = make_privmsg("#x", "PONG", vec![]);

        assert!(is_pong(&pong));
        assert!(!is_pong(&not_pong));
    }
}
