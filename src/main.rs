use clap::{Parser, Subcommand};

mod convert;
mod doom;
mod join;
mod minecraft;

use itertools::Itertools;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    commands: Commands,

    //Strings to be used
    #[arg(global = true)]
    strings: Vec<String>,

    ///If any strings contain any of these substrings, remove them
    #[arg(short, long, global = true, num_args = 1 ..)]
    ignore: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a minecraft skin to a doom skin
    Convert,
    /// Join existing wads with skins into a new wad
    Join,
}

fn ignoring(ignore: Vec<String>) -> impl Fn(&String) -> bool {
    move |p| !ignore.iter().any(|i| p.contains(i))
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse_from(wild::args());

    let (commands, strings) = (args.commands, args.strings.into_iter().filter(ignoring(args.ignore)));
    match commands {
        Commands::Convert => Ok(convert::convert_all(
            strings
                .chunks(2)
                .into_iter()
                .map(|mut p| (p.next().unwrap(), p.next().unwrap().to_uppercase()))
                .into_iter())?
        ),
        Commands::Join => Ok(join::join_all(strings.into_iter())?),
    }
}