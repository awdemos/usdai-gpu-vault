use crate::errors::VaultError;
use anchor_lang::prelude::*;

/// Compute `value * bps / 10_000` with overflow protection.
///
/// Used for LTV ceilings and protocol fee calculations.
pub fn checked_bps_mul(value: u64, bps: u16) -> Result<u64> {
    (value as u128)
        .checked_mul(bps as u128)
        .and_then(|v| v.checked_div(10_000))
        .ok_or(VaultError::MathOverflow.into())
        .map(|v| v as u64)
}

/// Compute LTV in basis points: `borrowed * 10_000 / valuation`.
///
/// Returns `Err(VaultError::MathOverflow)` on overflow or division by zero.
pub fn checked_ltv_bps(borrowed: u64, valuation: u64) -> Result<u16> {
    if valuation == 0 {
        return Err(VaultError::ZeroValuation.into());
    }
    (borrowed as u128)
        .checked_mul(10_000)
        .and_then(|v| v.checked_div(valuation as u128))
        .ok_or(VaultError::MathOverflow.into())
        .map(|v| v as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // checked_bps_mul
    // ------------------------------------------------------------------
    #[test]
    fn bps_mul_happy_path() {
        // 1_000_000 * 7000 / 10000 = 700_000
        assert_eq!(checked_bps_mul(1_000_000, 7000).unwrap(), 700_000);
    }

    #[test]
    fn bps_mul_full_percent() {
        assert_eq!(checked_bps_mul(1_000_000, 10_000).unwrap(), 1_000_000);
    }

    #[test]
    fn bps_mul_zero_value() {
        assert_eq!(checked_bps_mul(0, 7000).unwrap(), 0);
    }

    #[test]
    fn bps_mul_zero_bps() {
        assert_eq!(checked_bps_mul(1_000_000, 0).unwrap(), 0);
    }

    #[test]
    fn bps_mul_tiny_fraction() {
        // 1 * 1 / 10000 = 0 (integer truncation)
        assert_eq!(checked_bps_mul(1, 1).unwrap(), 0);
    }

    #[test]
    fn bps_mul_fee_calculation() {
        // 500_000 * 10 / 10000 = 500
        assert_eq!(checked_bps_mul(500_000, 10).unwrap(), 500);
    }

    #[test]
    fn bps_mul_max_u64() {
        // u64::MAX * 1 / 10000 should not overflow (uses u128 internally)
        let result = checked_bps_mul(u64::MAX, 1).unwrap();
        assert_eq!(result, u64::MAX / 10_000);
    }

    #[test]
    fn bps_mul_overflow_protection() {
        // u64::MAX * 10000 would overflow u64, but u128 handles it
        let result = checked_bps_mul(u64::MAX, 10_000).unwrap();
        assert_eq!(result, u64::MAX);
    }

    // ------------------------------------------------------------------
    // checked_ltv_bps
    // ------------------------------------------------------------------
    #[test]
    fn ltv_happy_path() {
        // 700_000 / 1_000_000 * 10000 = 7000 bps (70%)
        assert_eq!(checked_ltv_bps(700_000, 1_000_000).unwrap(), 7000);
    }

    #[test]
    fn ltv_exactly_100_percent() {
        assert_eq!(checked_ltv_bps(1_000_000, 1_000_000).unwrap(), 10_000);
    }

    #[test]
    fn ltv_zero_borrowed() {
        assert_eq!(checked_ltv_bps(0, 1_000_000).unwrap(), 0);
    }

    #[test]
    fn ltv_zero_valuation_fails() {
        let err = checked_ltv_bps(100, 0).unwrap_err();
        assert!(matches!(err, anchor_lang::error::Error::AnchorError(ref e) if e.error_name == "ZeroValuation"));
    }

    #[test]
    fn ltv_very_small_valuation() {
        // 6 / 1 * 10000 = 60_000 bps (way over 100%, but fits in u16)
        assert_eq!(checked_ltv_bps(6, 1).unwrap(), 60_000);
    }

    #[test]
    fn ltv_max_u64_values() {
        // u64::MAX / u64::MAX * 10000 = 10000
        assert_eq!(checked_ltv_bps(u64::MAX, u64::MAX).unwrap(), 10_000);
    }

    #[test]
    fn ltv_fractional_rounding() {
        // 1 / 3 * 10000 = 3333.33... → 3333
        assert_eq!(checked_ltv_bps(1, 3).unwrap(), 3333);
    }
}
