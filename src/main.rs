pub mod cmd;

use cmd::*;

fn main() {
    let cmd: Cmd = Cmd::from_args();
    if cmd.version {
        println!("version 0.0.1")
    } else if cmd.help {
        println!("help")
    } else {
        println!("starting jvm...")
    }
    // println!("{:#?}", cmd);
}