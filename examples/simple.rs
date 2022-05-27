use shr3::prelude::*;

/// Example with default seed and simple get().
fn example_1() {
    println!("example_1:");

    let mut shr3 = Shr3::new();
    for i in 0..10 {
        let value: u8 = shr3.get();
        println!("{}: 0x{:02X}", i, value);
    }
}

/// Example with default seed and get_range().
fn example_2() {
    println!("example_2:");

    let mut shr3 = Shr3::new();
    for i in 0..10 {
        let value: u16 = shr3.get_range(100..200);
        println!("{}: {}", i, value);
    }
}

/// Example with custom seed and get_bits().
fn example_3() {
    println!("example_3:");

    let mut shr3 = Shr3::new_state(12345);
    for i in 0..10 {
        let value: u16 = shr3.get_bits(10);
        println!("{}: 0x{:03X}", i, value);
    }
}

fn main() {
    example_1();
    example_2();
    example_3();
}

// vim: ts=4 sw=4 expandtab
