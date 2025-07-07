use std::fmt;

#[allow(dead_code)]
pub trait Connection: fmt::Debug {
    fn new(
        socket_url: &str,
        needle: &str,
        user_token: &str,
        user_login: &str,
        channel: &str,
    ) -> Self;
    fn url(&self) -> &str;
    fn auth_commands(&self) -> &[String; 5];
    fn needle(&self) -> &str;
    fn channel(&self) -> &str;
}

pub const CAPABILITIES: &str = "CAP REQ :twitch.tv/tags twitch.tv/commands";

#[derive(Debug, Clone, Default)]
pub struct WsConnection {
    pub socket_url: String,
    pub channel: String,
    pub auth_commands: [String; 5],
    pub needle: String,
}

/// Macro to simplify (?) creating `Connection` implementations for new structs.
///
///
/// There are three implementation details for implementers to note:
///   - The struct must also implement `Debug`,
///   - The struct must have a field named `auth_commands` (typed as `[String; 5]`),
///   - The macro call must use a specific ordering:
///     1. Struct name,
///     2. URL to for which to connect,
///     3. The needle (i.e, the search string),
///     4. User OAuth token,
///     5. User login (i.e, the user's username),
///     6. Channel to join.
///
/// # Example
/// 
/// ```
/// // The struct must implement `Debug`, as required by `Connection` trait bounds
/// #[derive(Debug)]
/// struct ExampleConnection {
///     pub url: String,
///     pub channel: String,
///     pub needle: String,
///     // Requires an `auth_commands` field
///     pub auth_commands: [String; 5],
/// }
///
/// // Pass the name of each respective parameter to the macro - as if calling
/// // `ExampleConnection::new`. the actual parameters don't have to exist as variables,
/// // but should reflect the respective fields in the implementing struct.
/// // 
/// // The required order is:
/// //  1. Struct name,
/// //  2. URL to connect to,
/// //  3. Needle (search string)
/// //  4. User's OAuth token,
/// //  5. User's login/username
/// //  6. Target channel's name
/// impl_connection!(ExampleConnection, url, needle, token, login, channel);
///
/// // ...
/// // `ExampleConnection` implements methods from the `Connection` trait:
/// let example_connection = ExampleConnection::new(
///     "wss://irc-ws.twitch.tv", 
///     "trigger_word",
///     "oauth_token_example", 
///     "john_chatter", 
///     "john_twitch",
/// );
///
/// assert_eq!(example_connection.url(), "wss://irc-ws.twitch.tv");
/// assert_eq!(example_connection.channel(), "john_twitch");
/// assert_eq!(example_connection.needle(), "trigger_word");
/// ```
#[macro_export]
macro_rules! impl_connection {
    (
        $struct:ty,
        $url:ident,
        $needle:ident,
        $user_token:ident, 
        $user_login:ident, 
        $channel:ident
    ) => {
        impl Connection for $struct {
            fn new(
                $url: &str,
                $needle: &str,
                $user_token: &str,
                $user_login: &str,
                $channel: &str,
            ) -> Self {

                let channel_join = format!("JOIN #{}", $channel);
                let user_oauth = format!("PASS oauth:{}", $user_token);
                let user_nick = format!("NICK {}", $user_login);
                let user_login = format!("USER {} 8 * :{}", $user_login, $user_login);

                // keep ordering!
                // Twitch IRC requirements mean these must be sent in order.
                let auth_commands = [
                    crate::ws::connection::CAPABILITIES.to_string(),
                    user_oauth,
                    user_nick,
                    user_login,
                    channel_join,
                ];

                Self {
                    $channel: $channel.to_string(),
                    $needle: $needle.to_string(),
                    $url: $url.to_string(),
                    auth_commands,
                }
            }
            fn url(&self) -> &str {
                &self.$url
            }
            fn channel(&self) -> &str {
                &self.$channel
            }
            fn auth_commands(&self) -> &[String; 5] {
                &self.auth_commands
            }
            fn needle(&self) -> &str {
                &self.$needle
            }
        }
    };
}

impl_connection!(WsConnection, socket_url,  needle, user_token, user_login, channel);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ws_connection() {
        let socket_url = "wss://irc-ws.chat.twitch.tv";
        let user_token = "fake_token_for_testing";
        let user_login = "testusername";
        let channel = "testchannel";

        let result = WsConnection::new(socket_url, "test", user_token, user_login, channel);
        assert_eq!(result.url(), socket_url);
        assert_eq!(result.channel(), channel);
        assert_eq!(result.auth_commands().len(), 5);
    }

    #[test]
    fn test_auth_commands() {
        let socket_url = "wss://irc-ws.chat.twitch.tv";
        let user_token = "fake_token_for_testing";
        let user_login = "testusername";
        let channel = "testchannel";

        let result = WsConnection::new(socket_url, "test", user_token, user_login, channel);

        assert_eq!(
            result.auth_commands[0],
            "CAP REQ :twitch.tv/tags twitch.tv/commands"
        );
        assert_eq!(result.auth_commands[1], "PASS oauth:fake_token_for_testing");
        assert_eq!(result.auth_commands[2], "NICK testusername");
        assert_eq!(
            result.auth_commands[3],
            "USER testusername 8 * :testusername"
        );
        assert_eq!(result.auth_commands[4], "JOIN #testchannel");
    }
}
