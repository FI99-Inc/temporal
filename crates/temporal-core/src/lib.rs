//! Internal temporal domain and deterministic evaluation for Temporal Engine.
//!
//! The core receives normalized data and explicit time. It has no dependency on
//! the desktop shell, source adapters, persistence, or presentation.

macro_rules! string_serde {
    ($($name:ty),+ $(,)?) => {$ (
        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.to_string())
            }
        }
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = <String as serde::Deserialize>::deserialize(deserializer)?;
                value.parse().map_err(serde::de::Error::custom)
            }
        }
    )+};
}
pub(crate) use string_serde;

pub mod clock;
pub mod codec;
pub mod domain;
pub mod fit;
pub mod fulfillment;
pub mod lifecycle;
pub mod opportunity;
pub mod reasons;
pub mod recurrence;
pub mod results;
pub mod source_health;
pub mod time;
pub mod validation;
