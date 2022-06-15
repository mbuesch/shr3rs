# SHR3: 3-shift register random number generator

[https://bues.ch/](https://bues.ch/)

The SHR3 generator can be used to generate non-crypto random bits with only very few computations.

It is suitable for running on very small and restricted hardware (e.g. small 8 bit microcontrollers).
The SHR3 function is evaluated once per extracted random bit. The LSB of the SHR3 state is extracted as output.

The generator has a cycle of approximately `4_000_000_000` bits.
Do not use it to extract large amounts of random bits (more than a hundred MiB or so),
unless you can tolerate looping back to the beginning of the random stream.
It will loop back to the beginning after `2**32 - 1` iterations.

This generator is *not* cryptographically secure! Do not use it for cryptographic applications.

# Example usage:

```
    use shr3::prelude::*;

    let mut shr3 = Shr3::new();                 // SHR3 with default seed (1).
    let x: u8 = shr3.get();                     // Extract 8 bits from shr3.
    let y: u16 = shr3.get_bits(10);             // Extract 10 bits from shr3 and store in lower bits of y.
    assert_eq!(x, 0xF8);                        // Extracted random value.
    assert_eq!(y, 0x2CC);                       // Extracted random value.

    let mut shr3 = Shr3::new_state(42);         // SHR3 with custom seed (42).

    let mut shr3: Shr3 = Default::default();    // Alternative to Shr::new().
```

# Example Cargo.toml dependencies

Add this to your Cargo.toml:

```
[dependencies]
shr3 = "0.1"
```

# no_std

This crate does not require the Rust std library. It does not link to std.

# Optimized implementation

This crate includes an optimized implementation for AVR 8-bit.

All other architectures use the generic implementation.
On most architectures, this generic implementation will be compiled to rather efficient code.

# License

Copyright (c) 2022 Michael Buesch <m@bues.ch>

Licensed under the Apache License version 2.0 or the MIT license, at your option.
