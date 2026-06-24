//! # dni — validate Spanish DNI and NIE numbers
//!
//! Validate Spanish identification numbers: the **DNI** (*Documento Nacional de Identidad*,
//! for citizens) and the **NIE** (*Número de Identidad de Extranjero*, for foreigners). A
//! faithful Rust port of the algorithms used by
//! [`python-stdnum`](https://arthurdejong.org/python-stdnum/).
//!
//! ```
//! use dni::{is_valid, calc_check_digit};
//!
//! // DNI: 8 digits + a check letter.
//! assert!(is_valid("12345678Z"));
//! assert!(!is_valid("12345678A"));
//! assert_eq!(calc_check_digit("12345678").unwrap(), 'Z');
//!
//! // NIE: X/Y/Z + 7 digits + a check letter.
//! assert!(dni::nie::is_valid("X1234567L"));
//! ```
//!
//! Surrounding whitespace and `-`/space separators are accepted, and input is upper-cased.
//!
//! **Zero dependencies** and `#![no_std]`.

#![no_std]
#![forbid(unsafe_code)]
#![doc(html_root_url = "https://docs.rs/dni/0.1.0")]

extern crate alloc;

use alloc::string::{String, ToString};
use core::fmt;

// Compile-test the README's examples as part of `cargo test`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

/// The check-letter table, indexed by `number % 23`.
const CHECK_LETTERS: &[u8; 23] = b"TRWAGMYFPDXBNJZSQVHLCKE";

/// Why a number is not valid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The number has the wrong length.
    InvalidLength,
    /// The number is malformed (bad characters or prefix).
    InvalidFormat,
    /// The check letter does not match.
    InvalidChecksum,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Error::InvalidLength => "number has an invalid length",
            Error::InvalidFormat => "number has an invalid format",
            Error::InvalidChecksum => "check letter does not match",
        };
        f.write_str(message)
    }
}

impl core::error::Error for Error {}

/// Strip `-`/space separators and surrounding whitespace, and upper-case.
#[must_use]
pub fn compact(number: &str) -> String {
    let filtered: String = number.chars().filter(|&c| c != ' ' && c != '-').collect();
    filtered.to_uppercase().trim().to_string()
}

/// `isdigits`: a non-empty run of ASCII digits.
fn is_digits(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit())
}

/// `number % 23` computed incrementally (so any length is safe), or `None` for a non-digit
/// or empty string.
fn modulo_23(digits: &str) -> Option<u32> {
    if digits.is_empty() {
        return None;
    }
    let mut remainder = 0u32;
    for character in digits.chars() {
        remainder = (remainder * 10 + character.to_digit(10)?) % 23;
    }
    Some(remainder)
}

/// Calculate the DNI check letter for the eight-digit `number` (without the check letter).
///
/// # Errors
/// Returns [`Error::InvalidFormat`] if `number` is empty or contains a non-digit.
pub fn calc_check_digit(number: &str) -> Result<char, Error> {
    let remainder = modulo_23(number).ok_or(Error::InvalidFormat)?;
    Ok(char::from(CHECK_LETTERS[remainder as usize]))
}

/// Validate a DNI, returning the compacted number on success.
///
/// # Errors
/// Returns [`Error::InvalidLength`], [`Error::InvalidFormat`], or [`Error::InvalidChecksum`].
pub fn validate(number: &str) -> Result<String, Error> {
    let number = compact(number);
    let chars: alloc::vec::Vec<char> = number.chars().collect();
    let body: String = body_before_last(&chars);

    if !is_digits(&body) {
        return Err(Error::InvalidFormat);
    }
    if chars.len() != 9 {
        return Err(Error::InvalidLength);
    }
    if chars[chars.len() - 1] != calc_check_digit(&body)? {
        return Err(Error::InvalidChecksum);
    }
    Ok(number)
}

/// Whether `number` is a valid DNI.
///
/// ```
/// # use dni::is_valid;
/// assert!(is_valid("00000000T"));
/// assert!(!is_valid("X1234567L")); // that's a NIE, not a DNI
/// ```
#[must_use]
pub fn is_valid(number: &str) -> bool {
    validate(number).is_ok()
}

/// The characters before the last one (Python's `number[:-1]`), or `""` if empty.
fn body_before_last(chars: &[char]) -> String {
    if chars.is_empty() {
        String::new()
    } else {
        chars[..chars.len() - 1].iter().collect()
    }
}

/// NIE (*Número de Identidad de Extranjero*) — Spanish foreigner numbers.
pub mod nie {
    use super::{
        body_before_last, calc_check_digit as dni_calc, compact as dni_compact, is_digits, Error,
    };
    use alloc::string::String;
    use alloc::{format, vec::Vec};

    /// Strip separators and upper-case (shared with DNI).
    #[must_use]
    pub fn compact(number: &str) -> String {
        dni_compact(number)
    }

    /// Calculate the NIE check letter for the eight-character `number` (`X/Y/Z` + 7 digits,
    /// without the check letter).
    ///
    /// # Errors
    /// Returns [`Error::InvalidFormat`] if the prefix is not `X`/`Y`/`Z` or the rest is not
    /// digits.
    pub fn calc_check_digit(number: &str) -> Result<char, Error> {
        let mut chars = number.chars();
        let prefix = match chars.next() {
            Some('X') => '0',
            Some('Y') => '1',
            Some('Z') => '2',
            _ => return Err(Error::InvalidFormat),
        };
        let rest: String = chars.collect();
        dni_calc(&format!("{prefix}{rest}"))
    }

    /// Validate a NIE, returning the compacted number on success.
    ///
    /// # Errors
    /// Returns [`Error::InvalidLength`], [`Error::InvalidFormat`], or
    /// [`Error::InvalidChecksum`].
    pub fn validate(number: &str) -> Result<String, Error> {
        let number = dni_compact(number);
        let chars: Vec<char> = number.chars().collect();

        // `number[1:-1]` — the middle characters.
        let middle: String = if chars.len() >= 2 {
            chars[1..chars.len() - 1].iter().collect()
        } else {
            String::new()
        };
        let first = chars.first().copied();

        if !is_digits(&middle) || !matches!(first, Some('X' | 'Y' | 'Z')) {
            return Err(Error::InvalidFormat);
        }
        if chars.len() != 9 {
            return Err(Error::InvalidLength);
        }
        if chars[chars.len() - 1] != calc_check_digit(&body_before_last(&chars))? {
            return Err(Error::InvalidChecksum);
        }
        Ok(number)
    }

    /// Whether `number` is a valid NIE.
    ///
    /// ```
    /// assert!(dni::nie::is_valid("X1234567L"));
    /// assert!(!dni::nie::is_valid("12345678Z")); // that's a DNI
    /// ```
    #[must_use]
    pub fn is_valid(number: &str) -> bool {
        validate(number).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dni_valid() {
        assert!(is_valid("12345678Z"));
        assert!(is_valid("00000000T"));
        assert!(is_valid("54362315K"));
        assert_eq!(validate("12345678-z").unwrap(), "12345678Z"); // separators + lowercase
        assert_eq!(calc_check_digit("12345678").unwrap(), 'Z');
    }

    #[test]
    fn dni_invalid() {
        assert_eq!(validate("12345678A"), Err(Error::InvalidChecksum));
        assert_eq!(validate("1234567Z"), Err(Error::InvalidLength));
        assert_eq!(validate("1234567XZ"), Err(Error::InvalidFormat)); // non-digit body
        assert!(!is_valid("X1234567L")); // NIE, not DNI
        assert!(!is_valid(""));
    }

    #[test]
    fn nie_valid() {
        assert!(nie::is_valid("X1234567L"));
        assert!(nie::is_valid("Y1234567X"));
        assert!(nie::is_valid("Z1234567R"));
        assert_eq!(nie::validate("x-2482300-w").unwrap(), "X2482300W");
        assert_eq!(nie::calc_check_digit("X1234567").unwrap(), 'L');
    }

    #[test]
    fn nie_invalid() {
        assert_eq!(nie::validate("X2482300A"), Err(Error::InvalidChecksum));
        assert_eq!(nie::validate("X2482300"), Err(Error::InvalidLength));
        assert_eq!(nie::validate("A1234567L"), Err(Error::InvalidFormat)); // bad prefix
        assert!(!nie::is_valid("12345678Z")); // DNI, not NIE
    }
}
