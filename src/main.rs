pub mod cmd;
pub mod classpath;
use cmd::*;
use classpath::*;

fn main() {
    let cmd: Cmd = Cmd::from_args();
    if cmd.version {
        println!("version 0.0.1")
    } else if cmd.help {
        println!("help")
    } else {
        println!("starting jvm...")
    }
    // let a = new_entry("path");
    println!("{:#?}", cmd);
}