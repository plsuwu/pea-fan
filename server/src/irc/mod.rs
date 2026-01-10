use tinyrand::{Rand, RandRange, Seeded, StdRand};
use tinyrand_std::ClockSeed;

pub mod client;
pub mod jitter;

pub fn idx(max: usize) -> usize {
    let seed = ClockSeed::default().next_u64();
    let mut rng = StdRand::seed(seed);

    rng.next_range(0..max)
}

pub enum ReplyReason {
    RowNotFound,
    BotCountQueried,
    FoundChatter,
}

impl ReplyReason {
    pub fn get_reply(&self) -> &'static str {
        let reasons = match self {
            ReplyReason::BotCountQueried => Self::BOT_COUNT_QUERY,
            ReplyReason::RowNotFound => Self::ROW_NOT_FOUND_QUERY,
            ReplyReason::FoundChatter => todo!(),
        };

        reasons[idx(reasons.len() - 1)]
    }

    const BOT_COUNT_QUERY: [&'static str; 6] = [
        "why would i tell you that. so you can mock me. typical",
        "do you think im stupid. do you actually think that i am dumb",
        "why dont you worry about your own counter instead huh",
        "do you also ask the mailman to open their own letters",
        "this is exactly why i hate it here",
        "you think youre clever dont you but you arent",
    ];

    const ROW_NOT_FOUND_QUERY: [&'static str; 6] = [
        "no idea who that is but i bet you already knew that you creep",
        "no data on that one which is suspicious what are they hiding",
        "why would you ask about someone who isnt on my list are you working together",
        "oh so now we're just inventing chatters great just what i needed",
        "cant find anything but im sure youll keep trying because thats what you people do",
        "they have said piss exactly 0 times because they dont exist you ghoul",
    ];
}
