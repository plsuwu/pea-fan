use crate::ws::client::{Store, WsClientError, WsClientResult};
use async_trait::async_trait;

#[derive(Debug)]
pub struct MockRedisLayer {
    pub client: redis::Client,
}

impl MockRedisLayer {
    pub async fn new(redis_url: &str) -> WsClientResult<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| WsClientError::Redis(format!("Redis connection failure: {}", e)))?;

        Ok(Self { client })
    }
}

#[async_trait]
impl Store for MockRedisLayer {
    async fn increment_counter(&self, _channel: &str, _user: &str) -> WsClientResult<()> {
        //
        Ok(())
    }
}
