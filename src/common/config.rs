use super::{ConfigSnafu, Result};

/// If the value begins with an '@', read the preceding file path,
/// otherwise returns the value.
///
/// prefix is used to provide context in case of an error.
pub(crate) fn key_file_or_string(value: String, prefix: String) -> Result<String> {
    Ok(match value.strip_prefix('@') {
        Some(key_file) => std::fs::read_to_string(key_file)
            .map_err(|err| {
                ConfigSnafu {
                    message: format!("Failed to read key from {key_file}: {err}"),
                    prefix,
                }
                .build()
            })?
            .trim()
            .into(),
        None => value,
    })
}
