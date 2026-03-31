// use std::collections::HashMap;

use tracing::instrument;

// use crate::db::prelude::Chatter;
// use crate::db::redis::migrator::{LeaderboardMap, transform, util};

// /// Takes a `&mut HashMap<channel_id, score>` (`a`) and a `&[(channel_id, score)]` (`b`). Inserts all
// /// scores from `b` into the current leaderboard `a`, adding scores together where `channel_id` is
// /// present in both `a` and `b`
// #[instrument(skip(a, b))]
// pub fn merge_scores(a: &mut HashMap<String, i64>, b: &[(String, i64)]) {
//     b.iter().for_each(|(chan, score)| {
//         a.entry(chan.clone())
//             .and_modify(|curr| *curr += score)
//             .or_insert(*score);
//     });
// }
//
// /// Merges `new_leaderboard` with the user corresponding to `user_id` in the leaderboard hashmap `map`.
// ///
// /// Takes ownership of the final HashMap before returning the owned value
// #[instrument(skip(map, new_leaderboard))]
// pub fn map_leaderboard<'a>(
//     map: &'a mut LeaderboardMap,
//     user_id: &str,
//     new_leaderboard: &[(String, i64)],
// ) {
//     let parsed_channel_leaderboard = new_leaderboard
//         .iter()
//         .map(|(chan_login, score)| {
//             let resolved_id = util::resolve_channel_id(chan_login);
//             (resolved_id, *score)
//         })
//         .collect::<Vec<_>>();
//
//     map.entry(user_id.to_string())
//         .and_modify(|leaderboard| {
//             transform::merge_scores(leaderboard, &parsed_channel_leaderboard);
//         })
//         .or_insert(transform::build_leaderboard(parsed_channel_leaderboard));
// }
//
// /// Collects a raw `Vec<(String, i64)>` leaderboard vector into a `HashMap<String, i64>` scoremap
// #[instrument(skip(vec))]
// pub fn build_leaderboard(vec: Vec<(String, i64)>) -> HashMap<String, i64> {
//     vec.into_iter()
//         .map(|(chan, score)| (chan, score))
//         .collect::<HashMap<_, _>>()
// }
//
// /// Transforms a `&[HelixUser]` into a `HashMap<String, HelixUser>` with that user's login as its
// /// key
// #[instrument(skip(helix_users))]
// pub fn map_login_to_helix_user(helix_users: &[Chatter]) -> HashMap<String, Chatter> {
//     helix_users
//         .into_iter()
//         .map(|user| (user.login.clone(), user.clone()))
//         .collect()
// }
//
// /// Trims the leading '#' from each element in a `Vec<(String, i64)>`
// #[instrument(skip(raw))]
// pub fn trim_channel_octo(raw: Vec<(String, i64)>) -> Vec<(String, i64)> {
//     raw.into_iter()
//         .map(|(channel, score)| (trim_octo(&channel), score))
//         .collect()
// }

#[inline]
#[instrument]
pub fn trim_octo(s: &str) -> String {
    tracing::debug!(s, "trimming octothorpe");
    s.split('#').nth(1).unwrap_or(s).to_owned()
}
