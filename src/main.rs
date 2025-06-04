use clap::{crate_version, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version = crate_version!(), about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        #[arg(short, long)]
        date: String,
    },
    List,
    Complete {
        habit_name: String,
        #[arg(short, long, default_value="a date")]
        date: String,
    },
    History {
        habit_name: String,
    },
    Rename {
        old_name: String,
        new_name: String,
    },
    Delete {
        habit_name: String,
    },
    Streak {
        habit_name: String,
    },
    Report {
        habit_name: String,
    }
}

fn main() {
    let cli: Cli = Cli::parse();

    if cli.verbose {
        println!("Debugging set to true");
    }

    match &cli.command {
        Some(Commands::Add { date }) => {
            println!("Adding item: {}", date);
            // Call function to add item
        }
        Some(Commands::List) => {
            println!("Listing all habits: ");
            // Call function to add item
        }
        Some(Commands::Complete{ habit_name, date }) => {
            println!("{} marked as complete!", habit_name);
            // Call function to add item
        }
        Some(Commands::History { habit_name }) => {
            println!("History for {}: ", habit_name);
            // Call function to add item
        }
        Some(Commands::Rename { old_name, new_name }) => {
            println!("History for {}: ", old_name);
            // Call function to add item
        }
        Some(Commands::Delete { habit_name }) => {
            println!("History for {}: ", habit_name);
            // Call function to add item
        }
        Some(Commands::Streak { habit_name }) => {
            println!("History for {}: ", habit_name);
            // Call function to add item
        }
        Some(Commands::Report { habit_name }) => {
            println!("History for {}: ", habit_name);
            // Call function to add item
        }
        None => {
            println!("No subcommand was used. Use --help for more info.");
            // Or, you could have default behavior here
        }
    }
}
