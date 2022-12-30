use chrono::{Months, Utc, Datelike};
use directories::ProjectDirs;
use owo_colors::OwoColorize;
use rusqlite::{Connection, Result};
use std::{fs::create_dir, path::PathBuf};

mod app;
use app::*;

fn print_row(row: &rusqlite::Row, date: bool, time: bool) -> Result<()> {
    Ok(println!(
        "{}{}Name: {} Note: {} Complete: {}",
        if date {
            format!("Date: {} ", row.get::<usize, String>(0)?.red())
        } else {
            "".to_string()
        },
        if time {
            format!("Time: {} ", row.get::<usize, String>(1)?.red())
        } else {
            "".to_string()
        },
        row.get::<usize, String>(2)?.green(),
        row.get::<usize, String>(3)?.blue(),
        row.get::<usize, u32>(4)?.magenta(),
    ))
}

fn complete_command(conn: &Connection, command: EventCommand) -> Result<()> {
    match command {
        EventCommand::Show(show_event) => {
            if show_event.all_none() {
                list(conn)
            } else {
                conn.query_row(
                    &*format!(
                        "SELECT date, time, name, note, complete FROM events WHERE {}",
                        show_event.as_sql_where()
                    ),
                    (),
                    |row| print_row(row, true, true),
                )
            }
        }
        EventCommand::Create(create_event) => {
            let note = match create_event.note {
                Some(note) => note,
                None => "None".to_string(),
            };
            conn.execute(
                "INSERT INTO events (date, time, name, note, complete) VALUES (?, ?, ?, ?, 0)",
                [
                    create_event.date,
                    create_event.time,
                    create_event.name,
                    note,
                ],
            )?;
            Ok(())
        }
        EventCommand::Remove(remove_event) => {
            if remove_event.all_none() {
                Ok(())
            } else {
                conn.execute(
                    &*format!("DELETE FROM events WHERE {}", remove_event.as_sql_where()),
                    (),
                )?;
                Ok(())
            }
        }
        EventCommand::Complete(complete_event) => {
            if complete_event.all_none() {
                Ok(())
            } else {
                conn.execute(
                    &*format!(
                        "UPDATE events SET complete = 1 WHERE {}",
                        complete_event.as_sql_where()
                    ),
                    (),
                )?;
                Ok(())
            }
        }
    }
}

fn show_calendar(conn: &Connection, prev: Option<u32>, next: Option<u32>) -> Result<()> {
    let date = Utc::now()
        .date_naive()
        .checked_add_months(Months::new(next.unwrap_or(0)))
        .unwrap_or(Utc::now().date_naive())
        .checked_sub_months(Months::new(prev.unwrap_or(0)))
        .unwrap_or(Utc::now().date_naive())
        .with_day(1)
        .unwrap_or(Utc::now().date_naive());
    let month = date.month();
    for day in date.iter_days().take_while(|date| date.month() == month) {
        // Temporary `show_date`
        // TODO: More beautiful table implementation
        show_date(conn, day.format("%Y-%m-%d").to_string(), None)?;
    }
    Ok(())
}

fn show_date(conn: &Connection, date: String, time: Option<String>) -> Result<()> {
    let prep = if let Some(time) = &time {
        println!("Date: {}, Time: {}", date, time);
        format!("date = '{}', time = '{}'", date, time)
    } else {
        println!("Date: {}", date);
        format!("date = '{}'", date)
    };
    Ok(conn
        .prepare(&*format!(
            "SELECT date, time, name, note, complete FROM events WHERE {}",
            prep
        ))?
        .query_map([], |row| {
            print_row(
                row,
                false,
                match time {
                    Some(_) => false,
                    _ => true,
                },
            )
        })?
        .for_each(drop))
}

fn list(conn: &Connection) -> Result<()> {
    Ok(conn
        .prepare("SELECT date, time, name, note, complete FROM events")?
        .query_map([], |row| print_row(row, true, true))?
        .for_each(drop))
}

/// Setup function that checks for filepaths, creates a connection with a database, and creates the
/// table used across the program
fn setup_conn() -> Result<Connection> {
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

    // FORMAT: DATE, "EVENT NAME", "NOTE", COMPLETE?
    conn.execute(
        "CREATE TABLE IF NOT EXISTS events (date TEXT, time TEXT, name TEXT UNIQUE, note TEXT, complete INTEGER)",
        (),
    )?;
    Ok(conn)
}

fn main() -> Result<()> {
    let conn = setup_conn()?;
    let app: Scheduler = argh::from_env();

    match app.command {
        // list all events
        Command::List(_) => list(&conn),
        // TODO: calendar
        #[allow(unused_variables)]
        Command::Show(Show { prev, next }) => show_calendar(&conn, prev, next),
        // list events for date or time
        Command::Date(Date { date, time }) => show_date(&conn, date, time),
        // events and all related commands
        Command::Event(Event { command }) => complete_command(&conn, command),
        // clear completed events in a date
        Command::Clear(Clear { date }) => Ok({
            conn.execute("DELETE FROM events WHERE date = ?, complete = 1", [date])?;
            ()
        }),
        // complete events in a date
        Command::Complete(Complete { date }) => Ok({
            conn.execute("UPDATE events SET complete = 1 WHERE date = ?", [date])?;
            ()
        }),
        // delete all events in a date
        Command::Delete(Delete { date }) => Ok({
            conn.execute("DELETE FROM events WHERE date = ?", [date])?;
            ()
        }),
    }
}
