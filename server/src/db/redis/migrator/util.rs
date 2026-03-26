use chrono::NaiveDateTime;
use tracing::instrument;

use crate::util::helix::HelixUser;

use std::{collections::HashMap, sync::LazyLock};

pub static LEGACY_REMAPS: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    HashMap::from([
        ("cchiko_", "chikogaki"),
        ("pekoe_bunny", "dearpekoe"),
        ("sheriff_baiken", "baikenvt"),
        ("haelpc", "netuserhael"),
        ("netaccount", "netuserhael"),
    ])
});

pub static MAPPED_IDS: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    HashMap::from([
        ("aaallycat", "276477565"),
        ("b0barley", "600818743"),
        ("baikenvt", "62127668"),
        ("sheriff_baiken", "62127668"),
        ("batatvideogames", "539128874"),
        ("bexvalentine", "1013832529"),
        ("bibibiscuitch", "1335538461"),
        ("byebi", "112887047"),
        ("cchiko_", "413015060"),
        ("chikogaki", "413015060"),
        ("chocojax", "26189911"),
        ("dearpekoe", "960172116"),
        ("pekoe_bunny", "960172116"),
        ("flippersphd", "130738371"),
        ("gibbbons", "51845736"),
        ("harupi", "899965170"),
        ("hempievt", "172265161"),
        ("herakita", "766308647"),
        ("kkcyber", "110505559"),
        ("kokopimento", "24714810"),
        ("krumroll", "782458136"),
        ("kumomomomomomomo", "786298312"),
        ("kyoharuvt", "741293014"),
        ("kyundere", "141880295"),
        ("lcolonq", "866686220"),
        ("liljuju", "533612086"),
        ("madmad01", "864287979"),
        ("meiya", "89007125"),
        ("miaelou", "605418870"),
        ("miffygeist", "795478771"),
        ("milia", "188503312"),
        ("misspeggyx", "818067359"),
        ("myramors", "478187203"),
        ("myrmidonvt", "83255335"),
        ("nanolather", "31086482"),
        ("netuserhael", "592547707"),
        ("haelpc", "592547707"),
        ("netaccount", "592547707"),
        ("niupao", "512796146"),
        ("noi_vt", "675393188"),
        ("pachi", "48807896"),
        ("parasi", "834137500"),
        ("plss", "103033809"),
        ("rena_chuu", "759166226"),
        ("saltae", "461736095"),
        ("sleepiebug", "610533290"),
        ("snoozy", "446955795"),
        ("souly_ch", "94316536"),
        ("tini", "122338258"),
        ("unipiu", "874233986"),
        ("vacu0usly", "54833441"),
        ("walfas", "23075617"),
        ("womfyy", "263446776"),
    ])
});

/// Legacy-to-current channel name lookups (based on the `LEGACY_REMAPS` HashMap)
#[instrument]
pub fn resolve_channel_login(raw: &str) -> String {
    LEGACY_REMAPS.get(raw).copied().unwrap_or(raw).to_string()
}

#[instrument]
pub fn resolve_channel_id(raw: &str) -> String {
    let original = raw.to_string();
    let lowered = raw.to_lowercase();

    MAPPED_IDS
        .get(lowered.as_str())
        .copied()
        .unwrap_or(original.as_str())
        .to_string()
}

pub trait KeyList
where
    // perhaps we just want to implement this directly for an Iterator (as opposed to something
    // that is IntoIterator)?
    Self: IntoIterator,
{
    fn dedup(&self) -> Self;
    fn lowercase(&self) -> Self;
    fn parse(&self, e: fn(&str) -> Option<String>) -> Self;
}

impl KeyList for Vec<String> {
    #[instrument(skip(self))]
    fn lowercase(&self) -> Self {
        self.into_iter().map(|val| val.to_lowercase()).collect()
    }

    #[instrument(skip(self))]
    fn dedup(&self) -> Self {
        let mut keys = self.to_owned();

        keys.sort();
        keys.dedup_by(|a, b| a == b);

        keys
    }

    #[instrument(skip(self, e))]
    fn parse(&self, e: fn(&str) -> Option<String>) -> Self {
        self.iter().filter_map(|v| e(v)).collect()
    }
}

#[instrument]
pub fn create_timestamp(offset_days: i64) -> NaiveDateTime {
    chrono::Utc::now()
        .naive_utc()
        .checked_sub_signed(chrono::Duration::days(offset_days))
        .unwrap()
}

#[derive(Debug, thiserror::Error)]
#[error("alignment mismatch at index {index}: expected {expected}, got {actual}")]
pub struct AlignmentError {
    index: usize,
    expected: String,
    actual: String,
}

#[instrument(skip(logins, users))]
pub fn validate_alignment(
    logins: &mut [String],
    users: &mut [HelixUser],
) -> Result<(), AlignmentError> {
    logins.sort_by_key(|a| a.to_lowercase());
    users.sort_by(|a, b| a.login.to_lowercase().cmp(&b.login.to_lowercase()));

    // perform validation on entire vector if debugging, otherwise validate at first element,
    // center element, and last element
    let indices: Vec<usize> = if cfg!(debug_assertions) {
        (0..logins.len()).collect()
    } else {
        let len = logins.len();
        vec![0, len / 2, len.saturating_sub(1)]
    };

    for &i in &indices {
        if i < logins.len() && logins[i].to_lowercase() != users[i].login.to_lowercase() {
            return Err(AlignmentError {
                index: i,
                expected: logins[i].clone(),
                actual: users[i].login.clone(),
            });
        }
    }

    Ok(())
}
