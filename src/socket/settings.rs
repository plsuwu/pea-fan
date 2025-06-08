// pub const BASE_WEBSOCKET_URL: &'static str = "wss://eventsub.wss.twitch.tv/ws";
pub const IRC_WEBSOCKET_URL: &'static str = "wss://irc-ws.chat.twitch.tv";

#[derive(Debug, Clone)]
pub struct ConnectionSettings {
    pub url: &'static str,
    pub ws_authentication: [String; 5],
}

/**
 * CAP REQ :twitch.tv/tags twitch.tv/commands
 * PASS oauth:************..
 * NICK plss
 * USER plss 8 * :plss
 * JOIN #[channel]
 */

impl ConnectionSettings {
    pub fn new(auth: &str, login: &str, channel: &str) -> ConnectionSettings {
        let capabilities = "CAP REQ :twitch.tv/tags twitch.tv/commands".to_string();
        let channel = format!("JOIN #{}", channel);

        let user_oauth = format!("PASS oauth:{}", auth);
        let user_nick = format!("NICK {}", login);
        let user_login = format!("USER {} 8 * :{}", login, login);

        let ws_authentication = [capabilities, user_oauth, user_nick, user_login, channel];

        ConnectionSettings {
            url: IRC_WEBSOCKET_URL,
            ws_authentication,
        }
    }
}
