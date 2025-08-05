use super::pg::DatabaseLayer;
use crate::database::pg::schema::{Channel, Score, User};

use async_trait::async_trait;
use rand::{Rng, distr::Alphanumeric};

pub trait TestChannel {
    fn generate_test_channel(user: &User) -> Channel {
        let channel_total = rand::random_range(user.total..=500);

        Channel {
            id: user.id.clone(),
            total: channel_total,
            created_at: None,
            updated_at: None,
        }
    }
}

pub trait TestScore {
    fn generate_test_score(user: &User, channel: &Channel) -> Score {
        // let user = User::generate_test_user();
        // let channel = Channel::generate_test_channel();

        let user_channel_score = rand::random_range(1..=user.total);

        Score {
            chatter_id: user.id.clone(),
            channel_id: channel.id.clone(),
            score: user_channel_score,
            created_at: None,
            updated_at: None,
        }
    }
}

pub trait TestChatter {
    fn generate_test_user() -> User {
        let id = format!("{}", rand::random_range(100000000..=999999999));
        let login: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        let color = "#000000".to_string();
        let image = "https://hello.com/".to_string();
        let redact = false;
        let total = rand::random_range(1..=500);

        User {
            id,
            login,
            image: Some(image),
            color,
            redact,
            total,
            created_at: None,
            updated_at: None,
        }
    }
}

impl TestChatter for User {}
impl TestChannel for Channel {}
impl TestScore for Score {}
