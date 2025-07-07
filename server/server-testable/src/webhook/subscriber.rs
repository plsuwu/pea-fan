use async_trait::async_trait;

use crate::webhook::types::EventType;

#[async_trait]
pub trait Subscriber {
    // todo: these need `Result<_, _>` return types
    
    fn create(broadcaster: &str, notification: EventType, token: &str, key: &str);
    fn delete(subscription_id: &str, token: &str);
    fn get_current(token: &str);
}

