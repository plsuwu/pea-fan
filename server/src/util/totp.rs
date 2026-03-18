use thiserror::Error;
use totp_rs::{Algorithm, Secret, TOTP};

use crate::util::constant_time_cmp;

pub type Result<T> = core::result::Result<T, TOTPError>;

#[derive(Error, Debug)]
pub enum TOTPError {
    #[error(transparent)]
    KeyReadError(#[from] crate::util::env::EnvErr),

    #[error(transparent)]
    SystemTime(#[from] std::time::SystemTimeError),
}

#[derive(Debug, Clone)]
pub struct TOTPHandler {
    generator: TOTP,
    key: Vec<u8>,
}

impl TOTPHandler {
    pub fn new(key: &'static str) -> Self {
        let key = key.as_bytes().to_vec();
        let generator = TOTP::new(
            Algorithm::SHA1,
            6,
            5,
            30,
            Secret::Raw(key.clone()).to_bytes().unwrap(),
        )
        .unwrap();

        let e = generator.get_secret_base32();
        tracing::info!(?e, "SECRET B32:");

        Self { generator, key }
    }

    pub fn totp_cmp(&mut self, input: &str) -> Result<bool> {
        self.write_code_out();

        let current = self.generator.generate_current()?;

        if constant_time_cmp(&current, input) {
            return Ok(true);
        }

        Ok(false)
    }

    pub fn write_code_out(&mut self) {
        let token = self.generator.generate_current().unwrap();
        let ttl = self.generator.ttl().unwrap();

        tracing::info!(token, "TOTP CODE:");
        tracing::info!(?ttl, "TIME-TO-LIVE:");
    }
}
