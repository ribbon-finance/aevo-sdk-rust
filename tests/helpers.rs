use aevo_sdk::signing::{bps_to_rate, rate_to_raw, to_raw6};

#[test]
fn raw6_valid_values() {
    assert_eq!(to_raw6("0").unwrap(), "0");
    assert_eq!(to_raw6("1").unwrap(), "1000000");
    assert_eq!(to_raw6("1.234567").unwrap(), "1234567");
    assert_eq!(to_raw6(".5").unwrap(), "500000");
}

#[test]
fn raw6_rejects_malformed_negative_and_too_precise_values() {
    for value in ["", "-", "-1", "1.0000001", "1e-6", "abc", "1.2.3", "+1"] {
        assert!(to_raw6(value).is_err(), "{value}");
    }
}

#[test]
fn rate_helpers_use_six_decimal_raw_units() {
    assert_eq!(bps_to_rate(0).unwrap(), "0");
    assert_eq!(bps_to_rate(3).unwrap(), "0.0003");
    assert_eq!(bps_to_rate(10_000).unwrap(), "1");
    assert_eq!(rate_to_raw("0.0003").unwrap(), "300");
    assert_eq!(rate_to_raw("1").unwrap(), "1000000");
    assert!(rate_to_raw("0.0000001").is_err());
}

#[test]
fn raw6_exact_six_decimal_boundaries() {
    assert_eq!(to_raw6("0.000001").unwrap(), "1");
    assert_eq!(to_raw6("0.999999").unwrap(), "999999");
    assert_eq!(to_raw6("1.").unwrap(), "1000000");
    assert_eq!(to_raw6(" 2.5 ").unwrap(), "2500000"); // surrounding whitespace is trimmed
    assert_eq!(rate_to_raw("0.01").unwrap(), "10000");
}

#[test]
fn raw6_handles_large_values_without_precision_loss() {
    assert_eq!(
        to_raw6("123456789012345678901234567890.123456").unwrap(),
        "123456789012345678901234567890123456"
    );
}

#[test]
fn raw6_rejects_values_that_overflow_u256_instead_of_panicking() {
    // U256::MAX; scaling by 10^6 overflows 256 bits.
    let huge = "115792089237316195423570985008687907853269984665640564039457584007913129639935";
    assert!(to_raw6(huge).is_err());
    assert!(rate_to_raw(huge).is_err());
    // Just past the largest representable value once scaled.
    assert!(to_raw6(
        "115792089237316195423570985008687907853269984665640564039457584007913129.639936"
    )
    .is_err());
}

#[test]
fn raw6_rejection_cases() {
    for value in [" ", ".", "1,5", "0x10", "NaN", "inf", "1 0", "１"] {
        assert!(to_raw6(value).is_err(), "{value:?} should be rejected");
    }
}

#[test]
fn bps_to_rate_boundaries() {
    assert_eq!(bps_to_rate(5).unwrap(), "0.0005");
    assert_eq!(bps_to_rate(100).unwrap(), "0.01");
    assert_eq!(bps_to_rate(u32::MAX).unwrap(), "429496.7295");
}
