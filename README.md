# dni

[![crates.io](https://img.shields.io/crates/v/dni.svg)](https://crates.io/crates/dni)
[![docs.rs](https://docs.rs/dni/badge.svg)](https://docs.rs/dni)
[![CI](https://github.com/trananhtung/dni/actions/workflows/ci.yml/badge.svg)](https://github.com/trananhtung/dni/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/dni.svg)](#license)

**Validate Spanish DNI and NIE identification numbers.**

- **DNI** (*Documento Nacional de Identidad*) — 8 digits + a check letter.
- **NIE** (*Número de Identidad de Extranjero*) — `X`/`Y`/`Z` + 7 digits + a check letter.

A faithful Rust port of the algorithms used by
[`python-stdnum`](https://arthurdejong.org/python-stdnum/).

- **Zero dependencies**, **`#![no_std]`**
- `is_valid`, `validate`, `calc_check_digit`, `compact` for both DNI and NIE
- Accepts `-`/space separators and lower-case input
- Differential-tested against `python-stdnum` (60k cases)

## Install

```toml
[dependencies]
dni = "0.1"
```

## Usage

```rust
use dni::{is_valid, validate, calc_check_digit};

// DNI:
assert!(is_valid("12345678Z"));
assert!(!is_valid("12345678A"));         // wrong check letter
assert_eq!(validate("12345678-z").unwrap(), "12345678Z");
assert_eq!(calc_check_digit("12345678").unwrap(), 'Z');

// NIE:
assert!(dni::nie::is_valid("X1234567L"));
assert_eq!(dni::nie::validate("x-2482300-w").unwrap(), "X2482300W");
```

The check letter is `"TRWAGMYFPDXBNJZSQVHLCKE"[number % 23]`; for a NIE the leading `X`/`Y`/`Z`
is first replaced with `0`/`1`/`2`.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
