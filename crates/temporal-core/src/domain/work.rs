use crate::time::Instant;
use std::{collections::BTreeSet, fmt, num::NonZeroU32, str::FromStr};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ContextTag(String);
crate::string_serde!(ContextTag);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidContextTag;
impl fmt::Display for InvalidContextTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("context must be a nonempty lowercase ASCII slug")
    }
}
impl std::error::Error for InvalidContextTag {}
impl FromStr for ContextTag {
    type Err = InvalidContextTag;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty()
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(InvalidContextTag);
        }
        Ok(Self(value.into()))
    }
}
impl ContextTag {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Display for ContextTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
pub type ContextTags = BTreeSet<ContextTag>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Effort {
    #[default]
    Unknown,
    Estimate(u32),
    AtLeast(NonZeroU32),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Importance {
    #[default]
    Unspecified,
    Low,
    Normal,
    High,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnergyRequirement {
    #[default]
    Unrestricted,
    Light,
    Normal,
    Deep,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnergyCapacity {
    Unknown,
    Light,
    Normal,
    Deep,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum DeclaredContexts {
    Unknown,
    Known(ContextTags),
}

#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkMetadata {
    pub effort: Effort,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_chunk_minutes: Option<NonZeroU32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earliest_start: Option<Instant>,
    pub required_contexts: ContextTags,
    pub energy_requirement: EnergyRequirement,
}
