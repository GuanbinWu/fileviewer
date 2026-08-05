use clap::Parser;


#[derive(Parser, Debug)]
#[command(name = "?")]
#[command(about = "?", long_about = None)]
pub struct Args {
    /// Input file or directory
    #[arg(short, long)]
    pub input: String,

}

// #[derive(Clone, Debug, PartialEq)]
// pub enum Operation {
//     Resize,
// }