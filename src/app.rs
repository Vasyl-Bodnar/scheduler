use argh::FromArgs;
// scheduler <list | show | date | event | clear | complete | delete>
// list <limit>
// show [next  [how far] | prev [how far]]
// date <date> [time]
// event <show | create | complete | remove>
// clear <date>
// complete <date>
// delete <date>
/// Main scheduler app
#[derive(FromArgs)]
pub struct Scheduler {
    #[argh(subcommand)]
    pub command: Command,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum Command {
    List(List),
    Show(Show),
    Date(Date),
    Event(Event),
    Clear(Clear),
    Complete(Complete),
    Delete(Delete),
}

/// List all the events with optional limit
#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
pub struct List {}

/// Show the month, optionally previous or next month
#[derive(FromArgs)]
#[argh(subcommand, name = "show")]
pub struct Show {
    /// previous month and optionally how far to prev
    #[argh(option)]
    pub prev: Option<u32>,
    /// next month and optionally how far to next
    #[argh(option)]
    pub next: Option<u32>,
}

/// Display specific date and optionally time
#[derive(FromArgs)]
#[argh(subcommand, name = "date")]
pub struct Date {
    /// specify date, format is YYYY-MM-DD
    #[argh(option)]
    pub date: String,
    /// specify optional time, format is HH:MM:SS
    #[argh(option)]
    pub time: Option<String>,
}

// event <show | create | update | complete | remove>
/// Manipulator for events
#[derive(FromArgs)]
#[argh(subcommand, name = "event")]
pub struct Event {
    #[argh(subcommand)]
    pub command: EventCommand,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum EventCommand {
    Show(ShowEvent),
    Create(CreateEvent),
    Complete(CompleteEvent),
    Remove(RemoveEvent),
}

/// Create events, must have name, date, and time, optionally can include a note
#[derive(FromArgs)]
#[argh(subcommand, name = "create")]
pub struct CreateEvent {
    /// specify date, format is YYYY-MM-DD
    #[argh(option)]
    pub date: String,
    /// specify time, format is HH:MM:SS
    #[argh(option)]
    pub time: String,
    /// specify name, unique
    #[argh(option)]
    pub name: String,
    /// specify note, not unique
    #[argh(option)]
    pub note: Option<String>,
}

pub struct GenericEvent {
    date: Option<String>,
    time: Option<String>,
    name: Option<String>,
    note: Option<String>,
    complete: Option<u32>,
}

pub trait EventAction {
    fn helper(&self) -> GenericEvent;

    fn all_none(&self) -> bool {
        let opts = self.helper();
        match (
            &opts.date,
            &opts.time,
            &opts.name,
            &opts.note,
            &opts.complete,
        ) {
            (None, None, None, None, None) => true,
            _ => false,
        }
    }

    fn as_sql_where(&self) -> String {
        let event = self.helper();
        let f = |x: &Option<String>, form: &str| match x {
            Some(v) => format!("{} = {},", form, v),
            None => "".to_string(),
        };
        String::from("")
            + &*f(&event.date, "date")
            + &*f(&event.time, "time")
            + &*f(&event.name, "name")
            + &*f(&event.note, "note")
            + match &event.complete {
                Some(1) => "complete = 1",
                Some(_) => "complete = 0",
                _ => "",
            }
    }
}

/// Show events that meet the criteria, if none is provided it will list all events
#[derive(FromArgs)]
#[argh(subcommand, name = "show")]
pub struct ShowEvent {
    /// specify date, format is YYYY-MM-DD
    #[argh(option)]
    date: Option<String>,
    /// specify time, format is HH:MM:SS
    #[argh(option)]
    time: Option<String>,
    /// specify name, unique
    #[argh(option)]
    name: Option<String>,
    /// specify note, not unique
    #[argh(option)]
    note: Option<String>,
    /// specify completeness, not unique
    #[argh(option)]
    complete: Option<u32>,
}

// TODO: Clones are ugly
impl EventAction for ShowEvent {
    fn helper(&self) -> GenericEvent {
        GenericEvent {
            date: self.date.clone(),
            time: self.time.clone(),
            name: self.name.clone(),
            note: self.note.clone(),
            complete: self.complete,
        }
    }
}

/// Complete events that meet the criteria, if none is provided it will do nothing
#[derive(FromArgs)]
#[argh(subcommand, name = "complete")]
pub struct CompleteEvent {
    /// specify date, format is YYYY-MM-DD
    #[argh(option)]
    date: Option<String>,
    /// specify time, format is HH:MM:SS
    #[argh(option)]
    time: Option<String>,
    /// specify name, unique
    #[argh(option)]
    name: Option<String>,
    /// specify note, not unique
    #[argh(option)]
    note: Option<String>,
    /// specify completeness, not unique
    #[argh(option)]
    complete: Option<u32>,
}

impl EventAction for CompleteEvent {
    fn helper(&self) -> GenericEvent {
        GenericEvent {
            date: self.date.clone(),
            time: self.time.clone(),
            name: self.name.clone(),
            note: self.note.clone(),
            complete: self.complete,
        }
    }
}

/// Delete events that meet the criteria, if none is provided it will do nothing
#[derive(FromArgs)]
#[argh(subcommand, name = "remove")]
pub struct RemoveEvent {
    /// specify date, format is YYYY-MM-DD
    #[argh(option)]
    date: Option<String>,
    /// specify time, format is HH:MM:SS
    #[argh(option)]
    time: Option<String>,
    /// specify name, unique
    #[argh(option)]
    name: Option<String>,
    /// specify note, not unique
    #[argh(option)]
    note: Option<String>,
    /// specify completeness, not unique
    #[argh(option)]
    complete: Option<u32>,
}

impl EventAction for RemoveEvent {
    fn helper(&self) -> GenericEvent {
        GenericEvent {
            date: self.date.clone(),
            time: self.time.clone(),
            name: self.name.clone(),
            note: self.note.clone(),
            complete: self.complete,
        }
    }
}

/// Clear all completed events from a specific date
#[derive(FromArgs)]
#[argh(subcommand, name = "clear")]
pub struct Clear {
    /// specify date, format is YYYY-MM-DD
    #[argh(option)]
    pub date: String,
}

/// Complete all events in a specific date
#[derive(FromArgs)]
#[argh(subcommand, name = "complete")]
pub struct Complete {
    /// specify date, format is YYYY-MM-DD
    #[argh(option)]
    pub date: String,
}
/// Delete all events from a specific date
#[derive(FromArgs)]
#[argh(subcommand, name = "delete")]
pub struct Delete {
    /// specify date, format is YYYY-MM-DD
    #[argh(option)]
    pub date: String,
}
