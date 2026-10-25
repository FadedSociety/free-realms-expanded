//! One SplitMix64 mixer for every seeded stream.
//!
//! Fishing odds, the NPC ambient firing schedule and the power-up drop stream
//! are all deterministic from a seed; a second copy of these constants drifts
//! silently when one caller is tuned. The T507 ambient schedule and the fishing
//! service each carried a verbatim copy before this module.

pub(crate) const GOLDEN_GAMMA: u64 = 0x9e37_79b9_7f4a_7c15;

/// One SplitMix64 step from `value`: advance by the golden gamma, then finalize.
/// This is the first output of `power_ups::Rng::seeded(value)`.
pub(crate) fn mix64(value: u64) -> u64 {
    finish64(value.wrapping_add(GOLDEN_GAMMA))
}

/// The SplitMix64 finalizer alone, for a stream that advances its own state
/// (`power_ups::Rng` adds the golden gamma itself before calling this).
pub(crate) fn finish64(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pinned SplitMix64 outputs: a change here shifts every seeded stream
    /// (fishing odds, the ambient schedule and power-up drops) at once, so it
    /// must be deliberate and made in this one file.
    #[test]
    fn the_shared_mixer_keeps_the_pinned_splitmix64_outputs() {
        assert_eq!(mix64(0), 0xe220_a839_7b1d_cdaf);
        assert_eq!(mix64(1), 0x910a_2dec_8902_5cc1);
        assert_eq!(finish64(1), 0x5692_161d_100b_05e5);
    }
}
