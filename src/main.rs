use clap::{crate_version, Parser, Subcommand};
use rusqlite::{Connection, Result};

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

fn initialize_db() -> Result<Connection> {
    let db_path_str = "~/.config/my_cli/data.db";
    let expanded_path = shellexpand::tilde(db_path_str);

    // Create the directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(expanded_path.as_ref()).parent() {
        std::fs::create_dir_all(parent).map_err(|e| rusqlite::Error::SqliteFailure(rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN), Some(e.to_string())))?;
    }
    let conn = Connection::open(expanded_path.as_ref())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS habits (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS completions (
            id INTEGER PRIMARY KEY,
            habit_id INTEGER NOT NULL,
            completion_date TEXT NOT NULL,
            FOREIGN KEY (habit_id) REFERENCES habits (id)
        )",
        [],
    )?;

    Ok(conn)
}

fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    let conn = initialize_db()?;

    if cli.verbose {
        println!("Debugging set to true");
    }

    match &cli.command {
        Some(Commands::Add { date }) => {
            println!("Adding item: {}", date);
            // Example: conn.execute("INSERT INTO habits (name) VALUES (?1)", [date])?;
        }
        Some(Commands::List) => {
            println!("Listing all habits: ");
            // Example: let mut stmt = conn.prepare("SELECT name FROM habits")?;
            // let habit_iter = stmt.query_map([], |row| row.get(0))?;
            // for habit in habit_iter { println!("- {}", habit.unwrap()); }
        }
        Some(Commands::Complete{ habit_name, date: _date_str }) => { // Renamed to avoid conflict, use _date_str
            println!("{} marked as complete!", habit_name);
            // Find habit_id, then: conn.execute("INSERT INTO completions (habit_id, completion_date) VALUES (?1, ?2)", [habit_id, _date_str])?;
        }
        Some(Commands::History { habit_name }) => {
            println!("History for {}: ", habit_name);
            // Query completions for habit_name
        }
        Some(Commands::Rename { old_name, new_name: _new_name_val }) => { // Renamed to avoid conflict
            println!("History for {}: ", old_name);
            // conn.execute("UPDATE habits SET name = ?1 WHERE name = ?2", [_new_name_val, old_name])?;
        }
        Some(Commands::Delete { habit_name }) => {
            println!("History for {}: ", habit_name);
            // conn.execute("DELETE FROM habits WHERE name = ?1", [habit_name])?;
            // Consider deleting related completions too.
        }
        Some(Commands::Streak { habit_name }) => {
            println!("History for {}: ", habit_name);
            // Calculate streak based on completions
        }
        Some(Commands::Report { habit_name }) => {
            println!("History for {}: ", habit_name);
            // Generate report based on completions
        }
        None => {
            println!("No subcommand was used. Use --help for more info.");
        }
    }

    Ok(())
}
