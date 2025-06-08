#![allow(dead_code)]

use crate::args;

use std::sync::{LazyLock, RwLock};

const IRC_URL: &'static str = "wss://irc-ws.chat.twitch.tv";
const IRC_CMD_CAP_REQ: &'static str = "CAP REQ :twitch.tv/tags twitch.tv/commands";

// this is the same as just writing the strings inline so this
// might be fine i think
const IRC_CMD_PASS: &'static str = "PASS oauth:";
const IRC_CMD_NICK: &'static str = "NICK";
const IRC_CMD_USER: &'static str = "USER"; // -> concat("[login] 8 * [login]")
const IRC_CMD_JOIN: &'static str = "JOIN #"; // -> concat("[broadcaster_login]")

const BROADCASTER: &'static str = "plss";

// currently facilitates a single connection to a broadcaster - needs to be reworked slightly
// if we want to track multiple broadcasters.
pub static CONNECTION_SETTINGS: LazyLock<RwLock<Vec<ConnectionSettings>>> =
    LazyLock::new(|| RwLock::new(vec![ConnectionSettings::new(BROADCASTER)]));

/// Websocket OAuth and related connection settings.
#[derive(Debug, Clone)]
pub struct ConnectionSettings {
    pub url: String,
    pub ws_auth_commands: [String; 5],

    // These may not be required here if we've got them in the `ws_auth_commands` array but also
    // this could be simply more convenient I DONT KNOW i dont know please i don't know stop stop
    pub login: String,
    pub token: String,
}

impl ConnectionSettings {
    pub(crate) fn new(broadcaster: &str) -> Self {
        let args = args::parse_cli_args();

        let args_clone = args.clone();

        let pass = format!("{}{}", IRC_CMD_PASS, args_clone.token);
        let nick = format!("{} {}", IRC_CMD_NICK, args_clone.login);
        let user = format!("{} {} 8 * {}", IRC_CMD_USER, args.login, args.login);
        let join = format!("{}{}", IRC_CMD_JOIN, broadcaster);

        let ws_auth_commands: [String; 5] = [IRC_CMD_CAP_REQ.to_string(), pass, nick, user, join];

        Self {
            url: IRC_URL.to_string(),
            login: args.login.clone(),
            token: args.token.clone(),
            ws_auth_commands,
        }
    }
}
