//! Deserialization implementation largely based on the [`envy`] crate.
//!
//! This implementation specifically provides a fix allowing us to use `rename`-type derive macros
//! on a struct, as the [`envy`] crate has not been updated in over a year.
//!
//! [`envy`]: https://github.com/softprops/envy

use std::borrow::Cow;
use std::iter::{IntoIterator, empty};
use std::sync::LazyLock;

use serde::Deserialize;
use serde::de::value::{MapDeserializer, SeqDeserializer};
use serde::de::{self, IntoDeserializer};
use thiserror::Error;
use tokio::sync::OnceCell;

static ENV_VARS: LazyLock<OnceCell<Env>> = LazyLock::new(OnceCell::new);
pub async fn get_var(var: Var) -> EnvResult<&'static str> {
    let vars = ENV_VARS.get_or_try_init(|| async { Env::new() }).await?;
    Ok(match var {
        Var::ClientId => &vars.client_id,
        Var::ClientSecret => &vars.client_secret,
        Var::UserLogin => &vars.user_login,
        Var::UserToken => &vars.user_token,
        Var::AppToken => &vars.app_token,
        Var::BrowserId => &vars.browser_id,
        Var::InternalToken => &vars.internal_post_token,
        Var::DatabaseUrl => &vars.database_url,
        Var::RedisUrl => &vars.redis_url,
        Var::CorsAllowOrigins => &vars.cors_allow_origins,
        Var::DiscordWebhookUrl => &vars.discord_webhook_url,
        Var::ServerApiPort => &vars.server_api_port,
        Var::OtelExporterEndpoint => &vars.otel_exporter_otlp_endpoint,
        // Var::OtelTempoGrpc => &vars.otel_tempo_grpc,
        // Var::OtelLokiHttp => &vars.otel_loki_http,
        Var::OtelExporterProto => &vars.otel_exporter_otlp_protocol,
        Var::ApiServiceName => &vars.api_service_name,
        Var::ApiTracerName => &vars.api_tracer_name,
    })
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Env {
    pub client_id: String,
    pub client_secret: String,
    pub user_login: String,
    pub user_token: String,
    pub app_token: String,
    pub browser_id: String,
    pub internal_post_token: String,
    pub database_url: String,
    pub redis_url: String,
    pub cors_allow_origins: String,
    pub discord_webhook_url: String,
    pub server_api_port: String,
    pub otel_exporter_otlp_protocol: String,
    pub otel_exporter_otlp_endpoint: String,
    // pub otel_tempo_grpc: String,
    // pub otel_loki_http: String,
    pub api_service_name: String,
    pub api_tracer_name: String,
}

impl Env {
    pub fn new() -> EnvResult<Self> {
        Ok(from_env::<Env>()?)
    }
}

#[derive(Debug)]
pub enum Var {
    ClientId,
    ClientSecret,
    UserLogin,
    UserToken,
    AppToken,
    BrowserId,
    InternalToken,
    DatabaseUrl,
    RedisUrl,
    CorsAllowOrigins,
    DiscordWebhookUrl,
    ServerApiPort,
    // OtelTempoGrpc,
    // OtelLokiHttp,
    OtelExporterEndpoint,
    OtelExporterProto,
    ApiServiceName,
    ApiTracerName,
}

#[macro_export]
macro_rules! var {
    ($ev:expr) => {
        $crate::util::env::get_var($ev)
    };
}

// ---
//  Deserializer implementation
// ---

pub struct Prefixed<'a>(Cow<'a, str>);
struct Val(String, String);
struct Varname(String);

struct Deserializer<'de, Iter: Iterator<Item = (String, String)>> {
    inner: MapDeserializer<'de, Vars<Iter>, EnvDeserializeError>,
}

struct Vars<Iter>
where
    Iter: IntoIterator<Item = (String, String)>,
{
    inner: Iter,
}

impl<'de> IntoDeserializer<'de, EnvDeserializeError> for Val {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> IntoDeserializer<'de, EnvDeserializeError> for Varname {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<Iter: Iterator<Item = (String, String)>> Iterator for Vars<Iter> {
    type Item = (Varname, Val);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(k, v)| (Varname(k.clone()), Val(k, v)))
    }
}

macro_rules! forward_parsed_vals {
    ($($ty:ident => $method:ident,)*) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, EnvDeserializeError>
            where
                V: de::Visitor<'de>
            {
                match self.1.parse::<$ty>() {
                    Ok(val) => val.into_deserializer().$method(visitor),
                    Err(e) => Err(serde::de::Error::custom(format_args!(
                        "{}: while parsing '{}' (provider: {})",
                        e, self.1, self.0
                    )))
                }
            }
        )*
    };
}

impl<'de> serde::de::Deserializer<'de> for Val {
    type Error = EnvDeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.1.into_deserializer().deserialize_any(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.1.is_empty() {
            SeqDeserializer::new(empty::<Val>()).deserialize_seq(visitor)
        } else {
            let values = self
                .1
                .split(',')
                .map(|v| Val(self.0.clone(), v.trim().to_owned()));
            SeqDeserializer::new(values).deserialize_seq(visitor)
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self.1.into_deserializer())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    forward_parsed_vals! {
        bool => deserialize_bool,
        u8 => deserialize_u8,
        u16 => deserialize_u16,
        u32 => deserialize_u32,
        u64 => deserialize_u64,
        i8 => deserialize_i8,
        i16 => deserialize_i16,
        i32 => deserialize_i32,
        i64 => deserialize_i64,
        f32 => deserialize_f32,
        f64 => deserialize_f64,
    }

    serde::forward_to_deserialize_any! {
        char str string unit bytes byte_buf map
        unit_struct tuple_struct identifier tuple
        ignored_any
        struct
    }
}

impl<'de> serde::de::Deserializer<'de> for Varname {
    type Error = EnvDeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.into_deserializer().deserialize_any(visitor)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    serde::forward_to_deserialize_any! {
        char str string unit seq option bytes byte_buf map
        unit_struct tuple_struct identifier tuple ignored_any
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 enum struct
    }
}

impl<'de, Iter: Iterator<Item = (String, String)>> Deserializer<'de, Iter> {
    fn new(vars: Iter) -> Self {
        Deserializer {
            inner: MapDeserializer::new(Vars { inner: vars }),
        }
    }
}

impl<'de, Iter: Iterator<Item = (String, String)>> serde::de::Deserializer<'de>
    for Deserializer<'de, Iter>
{
    type Error = EnvDeserializeError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self.inner)
    }

    serde::forward_to_deserialize_any! {
        char str string unit seq option bytes byte_buf
        newtype_struct unit_struct tuple_struct identifier
        tuple ignored_any bool u8 u16 u32 u64 i8 i16 i32 i64
        f32 f64 enum struct
    }
}

pub fn from_env<T>() -> Result<T, EnvDeserializeError>
where
    T: serde::de::DeserializeOwned,
{
    let vars = dotenvy::vars();
    from_iter(vars)
}

pub fn from_iter<Iter, T>(iter: Iter) -> Result<T, EnvDeserializeError>
where
    T: serde::de::DeserializeOwned,
    Iter: IntoIterator<Item = (String, String)>,
{
    T::deserialize(Deserializer::new(iter.into_iter()))
}

impl serde::de::Error for EnvDeserializeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        EnvDeserializeError::Custom(msg.to_string())
    }

    fn missing_field(field: &'static str) -> Self {
        EnvDeserializeError::MissingValue(field.into())
    }
}

impl<'a> Prefixed<'a> {
    #[allow(clippy::wrong_self_convention)]
    pub fn from_env<T>(&self) -> Result<T, EnvDeserializeError>
    where
        T: serde::de::DeserializeOwned,
    {
        self.from_iter(dotenvy::vars())
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_iter<Iter, T>(&self, iter: Iter) -> Result<T, EnvDeserializeError>
    where
        T: serde::de::DeserializeOwned,
        Iter: IntoIterator<Item = (String, String)>,
    {
        from_iter(iter.into_iter().filter_map(|(k, v)| {
            if k.starts_with(self.0.as_ref()) {
                Some((k.trim_start_matches(self.0.as_ref()).to_owned(), v))
            } else {
                None
            }
        }))
    }
}

pub fn prefixed<'a, C>(prefix: C) -> Prefixed<'a>
where
    C: Into<Cow<'a, str>>,
{
    Prefixed(prefix.into())
}

pub type EnvResult<T> = core::result::Result<T, EnvErr>;

#[derive(Debug, Error)]
pub enum EnvErr {
    #[error(transparent)]
    Dotenvy(#[from] dotenvy::Error),

    #[error(transparent)]
    DeserializationError(#[from] EnvDeserializeError),
}

#[derive(Debug, Error)]
pub enum EnvDeserializeError {
    #[error("env deserialization error: {0}")]
    Custom(String),

    #[error("{0}")]
    MissingValue(String),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::var;

    #[tokio::test]
    async fn test_vars_macro() {
        let login = var!(Var::UserLogin).await.unwrap();
        assert_eq!(login, "owoplease");
    }
}
