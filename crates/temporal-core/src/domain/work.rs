use crate::time::Instant;
use std::{collections::BTreeSet, fmt, num::NonZeroU32, str::FromStr};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ContextTag(String);
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Effort {
    #[default]
    Unknown,
    Estimate(u32),
    AtLeast(NonZeroU32),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Importance {
    #[default]
    Unspecified,
    Low,
    Normal,
    High,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EnergyRequirement {
    #[default]
    Unrestricted,
    Light,
    Normal,
    Deep,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnergyCapacity {
    Unknown,
    Light,
    Normal,
    Deep,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeclaredContexts {
    Unknown,
    Known(ContextTags),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkMetadata {
    pub effort: Effort,
    pub minimum_chunk_minutes: Option<NonZeroU32>,
    pub earliest_start: Option<Instant>,
    pub required_contexts: ContextTags,
    pub energy_requirement: EnergyRequirement,
}
