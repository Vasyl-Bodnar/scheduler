use argh::FromArgs;
use chrono::{Datelike, Utc};
use directories::ProjectDirs;
use owo_colors::OwoColorize;
use rusqlite::{Connection, Result};
use std::{fs::create_dir, path::PathBuf};

// TODO: Work on descriptions and names
/// Scheduler App
#[derive(FromArgs)]
struct Scheduler {
    #[argh(subcommand)]
    command: Commands,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Commands {
    List(List),
    Add(Add),
    Remove(Remove),
    Complete(Completes),
    Clear(Clear),
    Delete(Delete),
}

/// Clear complted tasks
#[derive(FromArgs)]
#[argh(subcommand, name = "clear")]
struct Clear {}

/// Delete all tasks
#[derive(FromArgs)]
#[argh(subcommand, name = "delete")]
struct Delete {}

/// Add a new task
#[derive(FromArgs)]
#[argh(subcommand, name = "add")]
struct Add {
    /// name
    #[argh(option)]
    name: String,
    /// date
    #[argh(option)]
    date: Option<String>,
    /// note
    #[argh(option)]
    note: Option<String>,
    /// importance
    #[argh(option)]
    importance: Option<i32>,
}

/// Complete tasks by filter or id
#[derive(FromArgs)]
#[argh(subcommand, name = "complete")]
struct Completes {
    #[argh(subcommand)]
    complete: UpdateCommands,
}

/// Remove tasks by filter or id
#[derive(FromArgs)]
#[argh(subcommand, name = "remove")]
struct Remove {
    #[argh(subcommand)]
    remove: UpdateCommands,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum UpdateCommands {
    ID(ID),
    Name(Name),
    Date(Date),
    Note(Note),
    Importance(Importance),
    Complete(Complete),
}

impl UpdateCommands {
    fn to_sql_filter(&self) -> String {
        match self {
            Self::ID(_) => "".to_string(),
            Self::Date(date) => format!("date = \"{}\"", date),
            Self::Name(Name { name }) => format!("name = {}", name),
            Self::Note(Note { note }) => format!("note = {}", note),
            Self::Importance(Importance { importance }) => format!("importance = {}", importance),
            Self::Complete(Complete { complete }) => {
                format!("complete = {}", if *complete { 1 } else { 0 })
            }
        }
    }
}

/// Set the id to be used
#[derive(FromArgs)]
#[argh(subcommand, name = "id")]
struct ID {
    /// id
    #[argh(option)]
    id: u32,
}

/// Set the name to be used
#[derive(FromArgs)]
#[argh(subcommand, name = "name")]
struct Name {
    /// name
    #[argh(option)]
    name: String,
}

/// Set the date to be used
#[derive(FromArgs, PartialEq, PartialOrd)]
#[argh(subcommand, name = "date")]
struct Date {
    /// year-month-day to modify
    #[argh(option)]
    date: Option<String>,
    /// hour:minute:second to modify
    #[argh(option)]
    time: Option<String>,
}

impl Date {
    fn default() -> Self {
        Date {
            date: None,
            time: None,
        }
    }

    fn new(s: &str) -> Self {
        let vec: Vec<&str> = s.split_whitespace().collect();
        Date {
            date: Some(vec[0].to_string()),
            time: Some(vec[1].to_string()),
        }
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let time = Utc::now();
        f.write_fmt(format_args!(
            "{} {}",
            self.date
                .clone()
                .unwrap_or(format!("{}-{}-{}", time.year(), time.month(), time.day())),
            self.time.clone().unwrap_or("11:59:59".to_string())
        ))?;
        Ok(())
    }
}

/// Set the note to be used
#[derive(FromArgs)]
#[argh(subcommand, name = "note")]
struct Note {
    /// note
    #[argh(option)]
    note: String,
}

/// Set the importance to use
#[derive(FromArgs)]
#[argh(subcommand, name = "importance")]
struct Importance {
    /// importance
    #[argh(option)]
    importance: u32,
}

/// Set the completeness to use
#[derive(FromArgs)]
#[argh(subcommand, name = "complete")]
struct Complete {
    /// complete
    #[argh(switch)]
    complete: bool,
}

/// List all the current and completed tasks
#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
struct List {
    /// option to sort by
    #[argh(option, from_str_fn(str_to_sort))]
    filter: Option<SortBy>,
}

/// Options for sorting
#[derive(Copy, Clone, Debug)]
enum SortBy {
    ID,
    Name,
    /// options for date are:
    /// date - gives year-month-day
    /// time - gives hour:minute:second
    /// full-date - gives both date and time, and is default
    Date(bool, bool),
    Note,
    Importance,
    Complete,
}

fn str_to_sort(s: &str) -> Result<SortBy, String> {
    Ok(match &*s.to_lowercase() {
        "id" => SortBy::ID,
        "name" => SortBy::Name,
        "date" => SortBy::Date(true, false),
        "time" => SortBy::Date(false, true),
        "full-date" => SortBy::Date(true, true),
        "note" => SortBy::Note,
        "importance" => SortBy::Importance,
        "complete" => SortBy::Complete,
        _ => SortBy::ID, // default
    })
}

/// Task row, exactly as it appears in the tasks table,
/// ```id``` is not the same id as ```rowid``` in the table, it fills and shifts empty rows
/// and is updated for each list iteration
struct Task {
    id: i32,
    name: String,
    date: Date,
    note: String,
    importance: i32,
    complete: bool,
}

/// Setup function that checks for filepaths, creates a connection with a database, and creates the
/// table used across the program
fn setup_conn() -> Result<Connection, rusqlite::Error> {
    let loc = match ProjectDirs::from("rs", "Fortuna", "scheduler") {
        Some(loc) => loc.data_dir().to_owned(),
        None => PathBuf::from(""),
    };
    let conn = match Connection::open(loc.join("schedule.db")) {
        Ok(conn) => conn,
        Err(_) => {
            create_dir(&loc).unwrap();
            Connection::open(loc.join("schedule.db"))?
        }
    };
    // FORMAT: 1, 2022-12-14 11:55:30.000, "Math Due", "WebWorks Math Assignment Due"@ 5
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (id INTEGER PRIMARY KEY, date TEXT, name TEXT, note TEXT, importance INTEGER NOT NULL, complete INTEGER)", (),)?;
    Ok(conn)
}

/// Extracted List Command for simpler main function and code readibility
fn list_rows(conn: Connection, filter: Option<SortBy>) -> Result<(), rusqlite::Error> {
    let mut prep = conn.prepare("SELECT name, date, note, importance, complete FROM tasks")?;
    let mut i = 0;
    let mut tasks: Vec<Task> = prep
        .query_map([], |row| {
            i += 1;
            Ok(Task {
                id: i,
                name: row.get::<usize, String>(0)?,
                date: Date::new(&*row.get::<usize, String>(1)?),
                note: row.get::<usize, String>(2)?,
                importance: row.get::<usize, i32>(3)?,
                complete: if row.get::<usize, i32>(4)? == 1 {
                    true
                } else {
                    false
                },
            })
        })?
        .filter_map(|x| x.ok())
        .collect();

    if let Some(f) = filter {
        tasks.sort_by(|a, b| match f {
            SortBy::ID => a.id.partial_cmp(&b.id).unwrap(),
            SortBy::Name => a.name.partial_cmp(&b.name).unwrap(),
            SortBy::Date(..) => a.date.partial_cmp(&b.date).unwrap(),
            SortBy::Note => a.note.partial_cmp(&b.note).unwrap(),
            SortBy::Importance => a.importance.partial_cmp(&b.importance).unwrap(),
            SortBy::Complete => a.complete.partial_cmp(&b.complete).unwrap(),
        });
    }

    // TODO: Work on colors
    for t in tasks {
        println!(
            "#{}, Name: {}, Due {}, Note: {}, Level: {}, Complete: {}",
            t.id.red(),
            t.name.blue(),
            match filter {
                Some(SortBy::Date(true, false)) => t.date.date.unwrap(),
                Some(SortBy::Date(false, true)) => t.date.time.unwrap(),
                _ => t.date.to_string(),
            }
            .green(),
            t.note.blue(),
            t.importance.to_string().green(),
            t.complete.to_string().red()
        );
    }
    Ok(())
}

fn modify_with_id(
    conn: Connection,
    filter: UpdateCommands,
    modf: String,
) -> Result<(), rusqlite::Error> {
    match filter {
        UpdateCommands::ID(ID { ref id }) => {
            let mut prep = conn.prepare("SELECT id FROM tasks")?;
            let mut i = 0;
            prep.query_map([], |row| {
                i += 1;
                if &i == id {
                    Ok(conn.execute(
                        &*format!("{} WHERE id = {}", modf, row.get::<usize, u32>(0)?),
                        (),
                    )?)
                } else {
                    Ok(5)
                }
            })?
            .for_each(drop);
        }
        UpdateCommands::Date(date) => {
            conn.execute(
                &*format!(
                    "{} WHERE date LIKE '%{}%'",
                    modf,
                    match date {
                        Date {
                            date: None,
                            time: Some(time),
                        } => time,
                        Date {
                            date: Some(date),
                            time: None,
                        } => date,
                        _ => date.to_string(),
                    }
                ),
                (),
            )?;
        }
        _ => {
            conn.execute(&*format!("{} WHERE {}", modf, filter.to_sql_filter()), ())?;
        }
    };
    Ok(())
}

fn main() -> Result<()> {
    let conn = setup_conn()?;

    let app: Scheduler = argh::from_env();

    match app.command {
        Commands::List(List { filter }) => list_rows(conn, filter)?,
        Commands::Add(Add {
            date,
            name,
            note,
            importance,
        }) => {
            conn.execute(
                &*format!(
                    "INSERT INTO tasks (date,name,note,importance,complete) VALUES (?,?,?,?,0)",
                ),
                [
                    Date::new(&*date.unwrap_or(Date::default().to_string())).to_string(),
                    name,
                    note.unwrap_or("None".to_string()),
                    importance.unwrap_or(0).to_string(),
                ],
            )?;
        }
        Commands::Remove(Remove { remove }) => {
            modify_with_id(conn, remove, "DELETE FROM tasks".to_string())?
        }
        Commands::Complete(Completes { complete }) => modify_with_id(
            conn,
            complete,
            "UPDATE tasks SET complete = 1 tasks".to_string(),
        )?,
        Commands::Clear(_) => {
            conn.execute("DELETE FROM tasks WHERE complete = 1", ())?;
        }
        Commands::Delete(_) => {
            conn.execute("DROP TABLE tasks", ())?;
        }
    }

    Ok(())
}
