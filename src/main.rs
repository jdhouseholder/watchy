use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, about = "Watches a set of files and runs a command with the file name passed as an argument on change.", long_about = None)]
struct Args {
    #[arg(long)]
    watch: Vec<String>,

    #[arg(long)]
    then: String,
}

fn main() {
    let mut args = Args::parse();
    args.watch.dedup();
    match watchy::watch(&args.watch, args.then) {
        Ok(()) => {}
        Err(e) => eprintln!("{}", e),
    }
}
