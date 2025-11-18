mod core;
mod switch;
mod parser;

use switch::*;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    args.next();
    let command = args.next();
    let switcher = Switcher::new()?;

    if let Some(command) = command && command == "switch" {
        let res = switcher.switch();
        println!("{}", res);
    } else {
        let mode = switcher.query();
        println!("{}", mode);
    }
    Ok(())
}
