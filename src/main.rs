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
        name: String
    },
    List,
    Complete {
        habit_name: String,
        #[arg(short, long)]
        date: Option<String>,
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

fn calculate_streak(conn: &Connection, habit_name: &str) -> Result<i64> {
    let mut stmt = conn.prepare("SELECT c.completion_date FROM completions c JOIN habits h ON c.habit_id = h.id WHERE h.name = ?1 ORDER BY c.completion_date DESC")?;
    let history_iter = stmt.query_map([habit_name], |row| row.get::<_, String>(0))?;

    let dates: Vec<chrono::NaiveDate> = history_iter
        .filter_map(|d| d.ok())
        .filter_map(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        .collect();

    let mut streak = 0;
    if !dates.is_empty() {
        let today = chrono::Local::now().naive_local().date();
        let mut current_date = today;

        // Check if the habit was completed today
        if dates.contains(&current_date) {
            streak += 1;
            current_date = current_date.pred_opt().unwrap();
        }

        // Check for consecutive days before today
        while dates.contains(&current_date) {
            streak += 1;
            current_date = current_date.pred_opt().unwrap();
        }
    }

    Ok(streak)
}

fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    let _conn = initialize_db()?;

    if cli.verbose {
        println!("Debugging set to true");
    }

    match &cli.command {
        Some(Commands::Add { name }) => {
            match _conn.execute("INSERT INTO habits (name) VALUES (?1)", [name]) {
                Ok(_) => println!("Habit '{}' added.", name),
                Err(e) => eprintln!("Error adding habit: {}", e),
            }
        }
        Some(Commands::List) => {
            let mut stmt = match _conn.prepare("SELECT name FROM habits") {
                Ok(stmt) => stmt,
                Err(e) => {
                    eprintln!("Error preparing statement: {}", e);
                    return Ok(());
                }
            };

            let habit_iter = match stmt.query_map([], |row| row.get(0)) {
                Ok(iter) => iter,
                Err(e) => {
                    eprintln!("Error querying habits: {}", e);
                    return Ok(());
                }
            };

            println!("{:<20} {:<10} {:<20}", "Habit", "Streak", "Last Completion");
            println!("{:-<20} {:-<10} {:-<20}", "", "", "");

            for habit_name in habit_iter {
                let habit_name = habit_name.unwrap_or_else(|_| "Error retrieving habit".to_string());
                let streak = calculate_streak(&_conn, &habit_name).unwrap_or(0);

                let mut stmt = _conn.prepare("SELECT MAX(completion_date) FROM completions c JOIN habits h ON c.habit_id = h.id WHERE h.name = ?1")?;
                let last_completion: Result<String, _> = stmt.query_row([&habit_name], |row| row.get(0));

                let last_completion_str = match last_completion {
                    Ok(date) => date,
                    Err(_) => "N/A".to_string(),
                };

                println!("{:<20} {:<10} {:<20}", habit_name, streak, last_completion_str);
            }
        }
        Some(Commands::Complete { habit_name, date }) => { // Renamed to avoid conflict, use _date_str
            let date_str = date.clone().unwrap_or_else(|| chrono::Local::now().naive_local().date().to_string());
            let habit_id: Result<i64, rusqlite::Error> = _conn.query_row(
                "SELECT id FROM habits WHERE name = ?1",
                [habit_name],
                |row| row.get(0),
            );

            match habit_id {
                Ok(id) => {
                    match _conn.execute("INSERT INTO completions (habit_id, completion_date) VALUES (?1, ?2)", [id.to_string(), date_str.clone()]) {
                        Ok(_) => println!("Habit '{}' marked as complete for {}.", habit_name, date_str),
                        Err(e) => eprintln!("Error completing habit: {}", e),
                    }
                }
                Err(e) => eprintln!("Error finding habit '{}': {}", habit_name, e),
            }
        }
        Some(Commands::History { habit_name }) => {
            let mut stmt = match _conn.prepare("SELECT c.completion_date FROM completions c JOIN habits h ON c.habit_id = h.id WHERE h.name = ?1 ORDER BY c.completion_date DESC") {
                Ok(stmt) => stmt,
                Err(e) => {
                    eprintln!("Error preparing statement: {}", e);
                    return Ok(());
                }
            };
            let history_iter = match stmt.query_map([habit_name], |row| row.get(0)) {
                Ok(iter) => iter,
                Err(e) => {
                    eprintln!("Error querying history: {}", e);
                    return Ok(());
                }
            };

            println!("Completion history for '{}':", habit_name);
            for date in history_iter {
                println!("- {}", date.unwrap_or_else(|_| "Error retrieving date".to_string()));
            }
        }
        Some(Commands::Rename { old_name, new_name }) => { 
            match _conn.execute("UPDATE habits SET name = ?1 WHERE name = ?2", [new_name, old_name]) {
                Ok(0) => eprintln!("Habit '{}' not found.", old_name),
                Ok(_) => println!("Habit '{}' renamed to '{}'.", old_name, new_name),
                Err(e) => eprintln!("Error renaming habit: {}", e),
            }
        }
        Some(Commands::Delete { habit_name }) => {
            let habit_id: Result<i64, rusqlite::Error> = _conn.query_row(
                "SELECT id FROM habits WHERE name = ?1",
                [habit_name],
                |row| row.get(0),
            );

            match habit_id {
                Ok(id) => {
                    _conn.execute("DELETE FROM completions WHERE habit_id = ?1", [id])?;
                    _conn.execute("DELETE FROM habits WHERE id = ?1", [id])?;
                    println!("Habit '{}' and its history have been deleted.", habit_name);
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    eprintln!("Habit '{}' not found.", habit_name);
                }
                Err(e) => {
                    eprintln!("Error deleting habit: {}", e);
                }
            }
        }
        Some(Commands::Streak { habit_name }) => {
            let mut stmt = match _conn.prepare("SELECT c.completion_date FROM completions c JOIN habits h ON c.habit_id = h.id WHERE h.name = ?1 ORDER BY c.completion_date DESC") {
                Ok(stmt) => stmt,
                Err(e) => {
                    eprintln!("Error preparing statement: {}", e);
                    return Ok(());
                }
            };

            let history_iter = match stmt.query_map([habit_name], |row| row.get::<_, String>(0)) {
                Ok(iter) => iter,
                Err(e) => {
                    eprintln!("Error querying history: {}", e);
                    return Ok(());
                }
            };

            let dates: Vec<chrono::NaiveDate> = history_iter
                .filter_map(|d| d.ok())
                .filter_map(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
                .collect();

            let mut streak = 0;
            if !dates.is_empty() {
                let today = chrono::Local::now().naive_local().date();
                let mut current_date = today;

                // Check if the habit was completed today
                if dates.contains(&current_date) {
                    streak += 1;
                    current_date = current_date.pred_opt().unwrap();
                }

                // Check for consecutive days before today
                while dates.contains(&current_date) {
                    streak += 1;
                    current_date = current_date.pred_opt().unwrap();
                }
            }

            println!("Current streak for '{}': {} days", habit_name, streak);
        }
        Some(Commands::Report { habit_name }) => {
            let mut stmt = match _conn.prepare("SELECT count(*) FROM completions c JOIN habits h ON c.habit_id = h.id WHERE h.name = ?1") {
                Ok(stmt) => stmt,
                Err(e) => {
                    eprintln!("Error preparing statement: {}", e);
                    return Ok(());
                }
            };
            let count: Result<i64, _> = stmt.query_row([habit_name], |row| row.get(0));

            match count {
                Ok(count) => println!("Habit '{}' has been completed {} times.", habit_name, count),
                Err(e) => eprintln!("Error generating report for '{}': {}", habit_name, e),
            }
        }
        None => {
            println!("No subcommand was used. Use --help for more info.");
        }
    }

    Ok(())
}
