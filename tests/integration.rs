//! Integration tests exercising the public API of `dni`.

use dni::{calc_check_digit, is_valid, nie, validate, Error};

#[test]
fn dni_round_trip() {
    for body in ["12345678", "00000001", "99999999", "54362315"] {
        let letter = calc_check_digit(body).unwrap();
        assert!(is_valid(&format!("{body}{letter}")));
    }
}

#[test]
fn dni_formatting_accepted() {
    assert_eq!(validate("  12345678 - z  ").unwrap(), "12345678Z");
    assert!(is_valid("12.345.678-Z".replace('.', "").as_str()));
}

#[test]
fn nie_round_trip() {
    for prefix in ["X", "Y", "Z"] {
        let body = format!("{prefix}1234567");
        let letter = nie::calc_check_digit(&body).unwrap();
        assert!(nie::is_valid(&format!("{body}{letter}")));
    }
}

#[test]
fn cross_rejection() {
    assert!(!is_valid("X1234567L")); // NIE is not a DNI
    assert!(!nie::is_valid("12345678Z")); // DNI is not a NIE
    assert_eq!(validate("Z1234567R"), Err(Error::InvalidFormat)); // letter where digits expected
}
