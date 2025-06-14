use serde::Deserialize;

pub fn deserialize_str<'de, T>(
    json: &'de str,
) -> Result<T, serde_path_to_error::Error<serde_json::Error>>
where
    T: Deserialize<'de>,
{
    let deserializer = &mut serde_json::Deserializer::from_str(json);

    serde_path_to_error::deserialize(deserializer)
}
