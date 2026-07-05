//! Types for the sticky events event defined in [MSC4354].
//!
//! [MSC4268]: https://github.com/matrix-org/matrix-spec-proposals/pull/4354

use js_int::UInt;
use serde::{Deserialize, Serialize, Serializer};

/// Sticky duration in milliseconds.
/// Valid values are the integer range 0-3600000 (1 hour).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub struct StickyDurationMs(u32);

impl StickyDurationMs {
    /// The maximum possible sticky duration in millis (1 hour).
    pub const MAX: u32 = 3_600_000;

    /// Creates a `DurationMs` by clamping `v` into `[0, 1h]`.
    pub fn new_clamped<T: Into<u64>>(v: T) -> Self {
        let v = v.into();
        let clamped = v.min(Self::MAX as u64) as u32;
        Self(clamped)
    }

    /// Get the value as a `u32`.
    pub fn get(self) -> u32 {
        self.into()
    }
}

impl From<StickyDurationMs> for u32 {
    fn from(d: StickyDurationMs) -> Self {
        d.0
    }
}

/// Message events can be annotated with a new top-level sticky object,
/// which MUST have a duration_ms, which is the number of milliseconds for the event to be
/// sticky.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum StickyObject {
    /// A valid sticky duration within the allowed range (0 to 1 hour).
    Valid(StickyDurationMs),
    /// An invalid sticky duration outside the allowed range.
    OutOfRange(UInt),
}

#[derive(Deserialize, Serialize)]
struct StickyDeHelper {
    duration_ms: UInt,
}

impl<'de> Deserialize<'de> for StickyObject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let helper = StickyDeHelper::deserialize(deserializer)?;
        let sticky_object = if helper.duration_ms <= StickyDurationMs::MAX.into() {
            Self::Valid(StickyDurationMs::new_clamped(helper.duration_ms))
        } else {
            Self::OutOfRange(helper.duration_ms)
        };

        Ok(sticky_object)
    }
}

impl Serialize for StickyObject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Valid(duration) => {
                StickyDeHelper { duration_ms: duration.get().into() }.serialize(serializer)
            }
            Self::OutOfRange(out_of_range) => {
                StickyDeHelper { duration_ms: *out_of_range }.serialize(serializer)
            }
        }
    }
}

impl StickyObject {
    /// Returns the sticky duration if the sticky event is valid.
    /// Returns `None` if the sticky property is invalid (out of range).
    pub fn sticky_duration(&self) -> Option<StickyDurationMs> {
        match self {
            StickyObject::Valid(duration) => Some(*duration),
            StickyObject::OutOfRange(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StickyObject;

}