use std::{collections::HashMap, fmt};

pub trait Manager: fmt::Debug {
    fn new(
        socket_url: &str,
        user_token: &str,
        user_login: &str,
        channels: Vec<impl Into<String>>,
    ) -> Self;
    fn is_joined(&self, channel: &str) -> bool;
}

pub const CAPABILITIES: &str = "CAP REQ :twitch.tv/tags twitch.tv/commands";
pub const DEFAULT_IRC: &str = "wss://irc-ws.chat.twitch.tv/";

#[derive(Debug, Clone)]
pub struct IrcAuthInfo {
    caps: String,
    pass: String,
    nick: String,
    user: String,
}

impl IrcAuthInfo {
    pub fn new(user_token: &str, user_login: &str) -> Self {
        let caps = CAPABILITIES.to_string();
        let pass = format!("PASS oauth:{}", user_token);
        let nick = format!("NICK {}", user_login);
        let user = format!("USER {} 8 * :{}", user_login, user_login);

        Self {
            caps,
            pass,
            nick,
            user,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IrcChannel {
    name: String,
    custom_needle: Vec<String>,
    joined: bool,
}

#[derive(Debug, Clone)]
pub struct SocketManager {
    url: String,
    auth_info: IrcAuthInfo,
    channels: HashMap<String, IrcChannel>,
}

impl Manager for SocketManager {
    fn new(
        socket_url: &str,
        user_token: &str,
        user_login: &str,
        tracked_channels: Vec<impl Into<String>>,
    ) -> Self {
        let auth_info = IrcAuthInfo::new(user_token, user_login);

        let mut channels = HashMap::new();
        tracked_channels.into_iter().for_each(|chan| {
            channels.insert(chan, false);
        });

        Self {
            url: socket_url.to_string(),
            auth_info,
            channels,
        }
    }

    fn is_joined(&self, channel: &str) -> bool {
        self.channels.contains_key(channel)
    }
}

// /// Macro to simplify (?) creating `Connection` implementations for new structs.
// ///
// ///
// /// There are three implementation details for implementers to note:
// ///   - The struct must also implement `Debug`,
// ///   - The struct must have a field named `auth_commands` (typed as `[String; 5]`),
// ///   - The macro call must use a specific ordering:
// ///     1. Struct name,
// ///     2. URL to for which to connect,
// ///     3. The needle (i.e, the search string),
// ///     4. User OAuth token,
// ///     5. User login (i.e, the user's username),
// ///     6. Channel to join.
// ///
// /// # Example
// ///
// /// ```
// /// // The struct must implement `Debug`, as required by `Connection` trait bounds
// /// #[derive(Debug)]
// /// struct ExampleConnection {
// ///     pub url: String,
// ///     pub channels: Vec<String>,
// ///     pub needle: String,
// ///     // Requires an `auth_commands` field
// ///     pub auth_commands: [String; 5],
// /// }
// ///
// /// // Pass the name of each respective parameter to the macro - as if calling
// /// // `ExampleConnection::new`. the actual parameters don't have to exist as variables,
// /// // but should reflect the respective fields in the implementing struct.
// /// //
// /// // The required order is:
// /// //  1. Struct name,
// /// //  2. URL to connect to,
// /// //  3. Needle (search string)
// /// //  4. User's OAuth token,
// /// //  5. User's login/username
// /// //  6. Target channel's name
// /// impl_connection!(ExampleConnection, url, needle, token, login, channel);
// ///
// /// // ...
// /// // `ExampleConnection` implements methods from the `Connection` trait:
// /// let example_connection = ExampleConnection::new(
// ///     "wss://irc-ws.twitch.tv",
// ///     "Joel",
// ///     "oauth_token_example",
// ///     "john_chatter",
// ///     "john_twitch",
// /// );
// ///
// /// assert_eq!(example_connection.url(), "wss://irc-ws.twitch.tv");
// /// assert_eq!(example_connection.channel(), "john_twitch");
// /// assert_eq!(example_connection.needle(), "Joel");
// /// ```
// #[macro_export]
// macro_rules! impl_connection {
//     (
//         $struct:ty,
//         $url:ident,
//         $needle:ident,
//         $user_token:ident,
//         $user_login:ident,
//         $channels:ident
//     ) => {
//         impl Connection for $struct {
//             fn new(
//                 $url: &str,
//                 $needle: &str,
//                 $user_token: &str,
//                 $user_login: &str,
//                 $channels: Vec<impl Into<String>>,
//             ) -> Self {
//
//                 let mut join = String::from("JOIN ");
//                 let count = $channels.len();
//
//                 let channels = $channels.into_iter().enumerate().map(|(idx, chan)|  {
//                     let chan = chan.into();
//                     if idx != count {
//                         join.push_str(&format!("#{},", chan));
//                     } else {
//                         join.push_str(&format!("#{}", chan));
//                     }
//
//                     chan
//                 }).collect();
//
//                 let user_oauth = format!("PASS oauth:{}", $user_token);
//                 let user_nick = format!("NICK {}", $user_login);
//                 let user_login = format!("USER {} 8 * :{}", $user_login, $user_login);
//
//                 // keep ordering!
//                 // Twitch IRC requirements mean these must be sent in order.
//                 let auth_commands = [
//                     crate::ws::connection::CAPABILITIES.to_string(),
//                     user_oauth,
//                     user_nick,
//                     user_login,
//                     join
//                 ];
//
//                 Self {
//                     $channels: channels,
//                     $needle: $needle.to_string(),
//                     $url: $url.to_string(),
//                     auth_commands,
//                 }
//             }
//             fn url(&self) -> &str {
//                 &self.$url
//             }
//             fn channels(&self) -> &Vec<String> {
//                 &self.$channels
//             }
//             fn auth_commands(&self) -> &[String; 5] {
//                 &self.auth_commands
//             }
//             fn needle(&self) -> &str {
//                 &self.$needle
//             }
//         }
//     };
// }
//
// impl_connection!(SocketConnection, socket_url,  needle, user_token, user_login, channels);

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_new_ws_connection() {
//
//         let channels = vec!["asaa", "asdfasdf", "asdf"];
//         let mut join_channels = String::from("JOIN ");
//         for (idx, chan) in channels.iter().enumerate() {
//             if idx != channels.len() - 1 {
//                 join_channels.push_str(&format!("#{},", chan));
//             } else {
//                 join_channels.push_str(&format!("#{}", chan));
//             };
//         }
//
//
//         let socket_url = "wss://irc-ws.chat.twitch.tv";
//         let user_token = "fake_token_for_testing";
//         let user_login = "testusername";
//
//         let result = SocketConnection::new(socket_url, "test", user_token, user_login, channels.clone());
//         assert_eq!(result.url(), socket_url);
//         assert_eq!(result.auth_commands().len(), 5);
//
//         result.channels().iter().enumerate().for_each(|(idx, channel)| {
//             assert_eq!(channel, &channels[idx]);
//         });
//     }
//
//     #[test]
//     fn test_auth_commands() {
//         let socket_url = "wss://irc-ws.chat.twitch.tv";
//         let user_token = "fake_token_for_testing";
//         let user_login = "testusername";
//         let channel = "testchannel";
//
//         let result = SocketConnection::new(socket_url, "test", user_token, user_login, channels);
//
//         assert_eq!(
//             result.auth_commands[0],
//             "CAP REQ :twitch.tv/tags twitch.tv/commands"
//         );
//         assert_eq!(result.auth_commands[1], "PASS oauth:fake_token_for_testing");
//         assert_eq!(result.auth_commands[2], "NICK testusername");
//         assert_eq!(
//             result.auth_commands[3],
//             "USER testusername 8 * :testusername"
//         );
//         assert_eq!(result.auth_commands[4], "JOIN #testchannel");
//     }
// }
