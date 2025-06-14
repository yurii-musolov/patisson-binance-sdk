use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SensitiveString(String);

impl Display for SensitiveString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "REDACTED")
    }
}

impl std::ops::Deref for SensitiveString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for SensitiveString {
    fn from(s: String) -> Self {
        SensitiveString(s)
    }
}

impl From<&str> for SensitiveString {
    fn from(s: &str) -> Self {
        SensitiveString(s.to_string())
    }
}

impl AsRef<str> for SensitiveString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl SensitiveString {
    pub fn expose(&self) -> &str {
        &self.0
    }
}
