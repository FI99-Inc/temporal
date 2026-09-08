//! Stable caller-assigned UUIDv4 identifiers. The evaluator never allocates IDs.
//!
//! Species remain distinct even when their underlying strings happen to match.
//! ```compile_fail
//! use temporal_core::domain::{AnchorId, DeadlineId};
//! let anchor: AnchorId = "00000000-0000-4000-8000-000000000001".parse().unwrap();
//! let deadline: DeadlineId = anchor;
//! ```

use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidIdentity;
impl fmt::Display for InvalidIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("expected a canonical lowercase UUIDv4")
    }
}
impl std::error::Error for InvalidIdentity {}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct CanonicalUuid([u8; 16]);

impl FromStr for CanonicalUuid {
    type Err = InvalidIdentity;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let bytes = value.as_bytes();
        if bytes.len() != 36 || bytes[14] != b'4' || !b"89ab".contains(&bytes[19]) {
            return Err(InvalidIdentity);
        }
        let mut result = [0u8; 16];
        let mut digit = 0;
        for (position, &byte) in bytes.iter().enumerate() {
            if [8, 13, 18, 23].contains(&position) {
                if byte != b'-' {
                    return Err(InvalidIdentity);
                }
                continue;
            }
            let value = match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                _ => return Err(InvalidIdentity),
            };
            result[digit / 2] = (result[digit / 2] << 4) | value;
            digit += 1;
        }
        Ok(Self(result))
    }
}

impl fmt::Display for CanonicalUuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, byte) in self.0.iter().enumerate() {
            if [4, 6, 8, 10].contains(&index) {
                f.write_str("-")?;
            }
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

macro_rules! ids {
    ($($name:ident),+ $(,)?) => {$ (
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(CanonicalUuid);
        crate::string_serde!($name);
        impl FromStr for $name {
            type Err = InvalidIdentity;
            fn from_str(value: &str) -> Result<Self, Self::Err> { value.parse().map(Self) }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
        }
    )+};
}

ids!(
    SourceId,
    AnchorId,
    DeadlineId,
    TaskRefId,
    IntentionId,
    RoutineId,
    AvailabilityId
);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RecordId {
    Anchor(AnchorId),
    Availability(AvailabilityId),
    Deadline(DeadlineId),
    Intention(IntentionId),
    Routine(RoutineId),
    Source(SourceId),
    TaskRef(TaskRefId),
}

impl RecordId {
    pub fn uuid_string(self) -> String {
        match self {
            Self::Anchor(id) => id.to_string(),
            Self::Availability(id) => id.to_string(),
            Self::Deadline(id) => id.to_string(),
            Self::Intention(id) => id.to_string(),
            Self::Routine(id) => id.to_string(),
            Self::Source(id) => id.to_string(),
            Self::TaskRef(id) => id.to_string(),
        }
    }
}
