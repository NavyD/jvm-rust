pub use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Cmd {
    /// help docs
    #[structopt(short, long)]
    pub help: bool,
    /// show version
    #[structopt(short, long)]
    pub version: bool,
    /// classpath
    #[structopt(long, default_value = "")]
    pub classpath: String,
    /// other arguments
    #[structopt(long)]
    pub args: Vec<String>,
}
