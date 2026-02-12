#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! argon2 = "0.5"
//! ```

use argon2::{
    Algorithm, Argon2, Params, PasswordHasher, Version,
    password_hash::SaltString,
};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <password>", args[0]);
        std::process::exit(1);
    }

    let password = &args[1];

    // Use same parameters as lib_common
    let params = Params::new(
        65536,    // m_cost: 64 MB memory
        10,       // t_cost: 10 iterations
        4,        // p_cost: 4 parallel threads
        Some(32), // output length: 32 bytes
    ).unwrap();

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let salt = SaltString::from_b64("yy7MPp4/ZZ31p+ykbPuJFg").unwrap();

    let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
    println!("{}", password_hash);
}
