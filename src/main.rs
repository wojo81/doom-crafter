use clap::{Parser, Subcommand};

mod create;
mod join;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    commands: Commands,

    //Paths to be used
    #[arg(global = true)]
    paths: Vec<String>,

    ///If paths contain these strings, ignore them
    #[arg(short, long, global = true, num_args = 1 ..)]
    ignore: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create new doom wad with skin(s)
    Create,
    /// Join existing wads into a new wad
    Join,
}

fn ignoring(ignore: Vec<String>) -> impl Fn(&String) -> bool {
    move |p| !ignore.iter().any(|i| p.contains(i))
}

fn main() {
    let args = Args::parse_from(wild::args());

    let (commands, paths) = (args.commands, args.paths.into_iter().filter(ignoring(args.ignore)));
    match commands {
        Commands::Create => create::create_all(paths.into_iter()),
        Commands::Join => join::join_all(paths.into_iter()),
    }
}