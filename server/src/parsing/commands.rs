#[derive(Debug, Clone, PartialEq)]
pub enum IrcCommand {
    PrivMsg {
        channel: String,
        message: String,
        user_info: Option<UserInfo>,
    },
    Notice {
        target: String,
        message: String,
    },
    Ping {
        server: String,
    },
    Pong {
        server: String,
    },
    UserNotice {
        channel: String,
        message: Option<String>,
        msg_id: Option<String>,
        user_info: Option<UserInfo>,
    },
    UserState {
        channel: String,
        message: Option<String>,
        msg_id: Option<String>,
        user_info: Option<UserInfo>,
    },
    ClearChat {
        channel: String,
        target_user: Option<String>,
        duration: Option<u64>,
    },
    ClearMsg {
        channel: String,
        target_msg_id: String,
    },
    Numeric {
        code: u16,
        params: Vec<String>,
    },
    Unknown {
        command: String,
        params: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserInfo {
    pub user_id: Option<String>,
    pub login: Option<String>,
    pub display_name: Option<String>,
    pub color: Option<String>,
    pub badges: Vec<String>,
    pub subscriber: bool,
    pub moderator: bool,
    pub vip: bool,
    pub broadcaster: bool,
}
