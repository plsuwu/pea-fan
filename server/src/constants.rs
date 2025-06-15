pub const IRC_WEBSOCKET_URL: &'static str = "wss://irc-ws.chat.twitch.tv";
pub const TWITCH_OAUTH_LENGTH: usize = 30;

#[cfg(feature = "production")]
pub const ORIGIN_URL_ENDSWITH: &'static [u8; 9] = b".piss.fan";

// #[cfg(feature = "production")]
pub const CALLBACK_ROUTE: &str = "https://api.piss.fan/webhook-global";

// #[cfg(not(feature = "production"))]
// pub const CALLBACK_ROUTE: &str = "https://pls.ngrok.io/webhook-global";

pub const API_GQL_URL: &str = "https://gql.twitch.tv/gql";
pub const API_HELIX_URL: &str = "https://api.twitch.tv/helix";
pub const BROWSER_CLIENT_ID: &str = "kimne78kx3ncx6brgo4mv6wki5h1ko";
pub const TESTING_CLIENT_ID: &str = "7jz14ixoeglm6aq8eott8196p4g5ox";

pub const SERVER_PORT: u16 = 3000;
pub const STREAM_ONLINE: &str = "stream.online";
pub const STREAM_OFFLINE: &str = "stream.offline";
pub const CHANNEL_CHAT_MESSAGE: &'static str = "channel.chat.message";

pub const VERSION: &str = "1";
pub const NEEDLE: &str = "piss";


pub const HMAC_PREFIX: &'static str = "sha256=";
pub const TWITCH_MESSAGE_ID: &'static str = "Twitch-Eventsub-Message-Id";
pub const TWITCH_MESSAGE_TIMESTAMP: &'static str = "Twitch-Eventsub-Message-Timestamp";
pub const TWITCH_MESSAGE_SIGNATURE: &'static str = "Twitch-Eventsub-Message-Signature";
pub const TWITCH_MESSAGE_TYPE_HEADER: &str = "Twitch-Eventsub-Message-Type";

// IRC COMMUNICATION COMMANDS
//
// Maybe these should be expanded upon :)
pub const IRC_COMMAND_PING: &str = "PING";
pub const IRC_COMMAND_CHAT: &str = "PRIVMSG";
pub const IRC_COMMAND_JOIN: &str = "JOIN";
pub const KEEPALIVE_RESPONSE: &str = "PONG :tmi.twitch.tv";

#[cfg(feature = "production")]
pub const TRACKED_CHANNELS_COUNT: usize = 26;

#[cfg(not(feature = "production"))]
pub const TRACKED_CHANNELS_COUNT: usize = 1;

pub type TrackedChannels = [&'static str; TRACKED_CHANNELS_COUNT];

#[cfg(feature = "production")]
pub const CHANNELS: TrackedChannels = [
    "sleepiebug",
    "parasi",
    "unipiu",
    "cchiko_",
    "liljuju",
    "vacu0usly",
    "bexvalentine",
    "rena_chuu",
    "snoozy",
    "womfyy",
    "kyoharuvt",
    "batatvideogames",
    "myrmidonvt",
    "kokopimento",
    "myramors",
    "sheriff_baiken",
    "lcolonq",
    "chocojax",
    "miffygeist",
    "haelpc",
    "gloomybyte",
    "niupao",
    "souly_ch",
    "kyundere",
    "miaelou",
    "saltae",
];

#[cfg(not(feature = "production"))]
pub const CHANNELS: TrackedChannels = ["plss"];
