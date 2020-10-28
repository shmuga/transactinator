extern crate clap;
extern crate serde;
extern crate csv;
extern crate rust_decimal;
extern crate itertools;

mod transaction;
mod account;
mod errors;
mod engine;

use clap::{App};

fn main() {
    let matches = App::new("transactinator")
        .version("1.0")
        .author("Mark O.")
        .about("Validates and processes transactions")
        .args_from_usage("
            <INPUT>              'Sets the input file to use'")

        .get_matches();

    matches.value_of("INPUT")
        .ok_or_else(|| panic!("Missing filename"))
        .and_then(engine::run)
        .map_err(|e| eprintln!("Error: {:?}", e))
        .unwrap();
}
