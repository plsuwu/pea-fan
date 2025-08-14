use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelCounter {
    /// Total count for a channel/broadcaster
    total: isize,

    /// Top chatters and their counts for a specific channel
    ///  
    /// `<chatter_login, total_in_channel>`
    top: HashMap<String, isize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatterCounter {
    /// Total count for a user
    total: isize,

    /// Chatter activity in tracked channels
    ///
    /// `<broadcaster_login, total_in_channel>`
    activity: HashMap<String, isize>,
}
