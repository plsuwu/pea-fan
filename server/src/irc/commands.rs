// #![allow(dead_code)]

use tokio::sync::oneshot;

#[derive(Debug, Default)]
pub struct IrcTags {
    pub user_id: String,
    pub user_login: String,

    pub display_name: String,

    pub color: String,
    pub channel_name: String,
    pub channel_id: String,
    pub msg_id: String,
}

#[derive(Debug)]
pub enum UserNoticeType {
    // Sub,
    Resub,
    // SubGift,
    // SubMysteryGift,
    // GiftPaidUpgrade,
    // RewardGift,
    // AnonGiftPaidUpgrade,
    // Raid,
    // Unraid,
    // BitsBadgeTier,
    // SharedChatNotice,
    Other(String),
}

#[derive(Debug)]
pub enum IncomingMessage {
    Clearchat {
        channel: String,
        chatter: String,
    },
    Clearmsg {
        channel: String,
        message: String,
    },
    Privmsg {
        tags: IrcTags,
        text: String,
    },
    Notice {
        channel: String,
        message: String,
    },
    GlobalUserstate,
    Userstate {
        channel: String,
        chatter: String,
        id: Option<String>,
    },
    Usernotice {
        channel: String,
        chatter: String,
        notice_type: UserNoticeType,
    },
    Join {
        channel: String,
        chatter: String,
    },
    Part {
        channel: String,
        chatter: String,
    },
}

pub enum OutgoingCommand {
    Reply { message: irc::proto::Message },
}

pub enum IrcQuery {
    GetJoinedChannels { reply: oneshot::Sender<Vec<String>> },
}

#[derive(Debug)]
pub enum TwitchCapability {
    Tags,
    Commands,
    Membership,
}

impl From<TwitchCapability> for irc::proto::Capability {
    fn from(value: TwitchCapability) -> Self {
        match value {
            TwitchCapability::Tags => irc::proto::Capability::Custom("twitch.tv/tags"),
            TwitchCapability::Commands => irc::proto::Capability::Custom("twitch.tv/commands"),
            TwitchCapability::Membership => irc::proto::Capability::Custom("twitch.tv/membership"),
        }
    }
}
