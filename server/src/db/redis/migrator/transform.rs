use tracing::instrument;

#[inline]
#[instrument]
pub fn trim_octo(s: &str) -> String {
    tracing::debug!(s, "trimming octothorpe");
    s.split('#').nth(1).unwrap_or(s).to_owned()
}
