use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

/// Serializes a map as a JSON array of key-value pairs instead of a JSON
/// object, so keys that are not strings (`Point`, `(i32, i32)`, ...) do not
/// fail with `serde_json`'s "key must be a string" error.
///
/// # Errors
/// Returns an error if the underlying serializer fails.
pub fn serialize<K, V, H, S>(map: &HashMap<K, V, H>, serializer: S) -> Result<S::Ok, S::Error>
where
    K: Serialize,
    V: Serialize,
    H: BuildHasher,
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(map.len()))?;
    for (key, value) in map {
        seq.serialize_element(&(key, value))?;
    }
    seq.end()
}

/// Deserializes either the new array-of-pairs form or the legacy JSON object
/// form (which only ever held empty maps for non-string keys).
#[derive(Deserialize)]
#[serde(untagged)]
#[serde(bound(deserialize = "K: Deserialize<'de> + Eq + Hash, V: Deserialize<'de>"))]
enum MapOrPairs<K, V> {
    Map(HashMap<K, V>),
    Pairs(Vec<(K, V)>),
}

/// Deserializes a map written by [`serialize`], accepting both the
/// array-of-pairs form and the legacy JSON object form.
///
/// # Errors
/// Returns an error if the input is neither a map nor an array of pairs.
pub fn deserialize<'de, K, V, D>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
where
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Ok(match MapOrPairs::deserialize(deserializer)? {
        MapOrPairs::Map(map) => map,
        MapOrPairs::Pairs(pairs) => pairs.into_iter().collect(),
    })
}
