mod core;
mod switch;

use switch::*;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    args.next();
    let command = args.next();
    let switcher = Switcher::new()?;

    if let Some(command) = command && command == "switch" {
        let res = switcher.switch_mode();
        println!("{}", res);
    } else {
        let mode = switcher.get_mode();
        println!("{}", mode);
    }
    Ok(())
}
