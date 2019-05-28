Sacio
=====

[![Documentation](https://docs.rs/sacio/badge.svg)](https://docs.rs/sacio)

A Rust interface for reading and writing SAC (Seismic Analysis Code) files

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
sacio = "0.1.0"
```

## Example

```rust
use sacio::Sac;
use sacio::SacString;

let mut s = Sac::from_file("tests/file.sac")?;

assert_eq!(s.mean_amp(), -0.09854721);
assert_eq!(s.min_amp(), -1.56928);
assert_eq!(s.max_amp(), 1.52064);

s.y.iter_mut().for_each(|v| *v *= 2.0);

s.extrema_amp();

assert_eq!(s.mean_amp(), -0.09854721 * 2.0);
assert_eq!(s.min_amp(), -1.56928 * 2.0);
assert_eq!(s.max_amp(), 1.52064 * 2.0);

s.set_string(SacString::Network, "CI");
s.set_string(SacString::Station, "PAS");
s.set_string(SacString::Location, "10");
s.set_string(SacString::T1, "PKIKP");
s.set_string(SacString::T1, "SKJKS");

assert_eq!(s.dist_deg(), 3.3574646);

s.to_file("tests/main.sac")?;

```

## License

This version is released under the MIT/X11 License


