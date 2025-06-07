///! Usage e.g (in `async fn main() .. `):
///! 
///! let settings_rw_lock = &*CONNECTION_SETTINGS;
///! let client = Client::new(settings_rw_lock).await?;
///! client.open(settings_rw_lock).await?;
///! client.loop_read().await;

pub mod settings;
pub mod client;
