mod formatter;
mod model;

use model::{Root, Waypoint};
use std::{
    env,
    error::Error,
    fs,
    io::{self, BufRead, Write},
    path::Path,
};
use time::OffsetDateTime;
use time_tz::{OffsetDateTimeExt, TimeZone};

fn main() -> Result<(), Box<dyn Error>> {
    let Some(provided_path) = env::args().nth(1) else {
        println!("Path to the GPX file must be provided.");
        return Ok(());
    };

    let filepath = Path::new(&provided_path);
    let contents = fs::read_to_string(filepath)?;
    let gpx = quick_xml::de::from_str::<Root>(&contents)?;

    let (date_format, time_format) = (formatter::date(), formatter::time());
    let mut waypoints = gpx
        .tracks
        .into_iter()
        .flat_map(|track| track.segments.into_iter().map(|segment| segment.waypoints))
        .flatten()
        .collect::<Vec<Waypoint>>();
    waypoints.sort_by(|a, b| a.time.cmp(&b.time));
    let from = waypoints.first().unwrap().time;
    let to = waypoints.last().unwrap().time;
    let timezone = time_tz::system::get_timezone()?;
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "System timezone is {}", timezone.name())?;
    writeln!(
        stdout,
        "Opened {filepath} with {waypoints} points on {day} from {from} to {to}",
        filepath = filepath.display(),
        waypoints = waypoints.len(),
        day = from.date().format(&date_format)?,
        from = from.to_timezone(timezone).time().format(&time_format)?,
        to = to.to_timezone(timezone).time().format(&time_format)?,
    )?;
    writeln!(stdout,
        "Enter a time to print information about the most recent track in 24-hour time (eg 14:32:07)."
    )?;
    let mut stdin = io::stdin().lock();
    let mut buffer = String::new();
    let date = waypoints.first().unwrap().time;

    loop {
        write!(stdout, "> ")?;
        stdout.flush()?;
        stdin.read_line(&mut buffer)?;
        let (hour, minute, second) = parse_time(&buffer);
        buffer.clear();
        let requested = date
            .to_timezone(timezone)
            .replace_hour(hour)?
            .replace_minute(minute)?
            .replace_second(second)?;
        if let Some(most_recent) = find_most_recent_waypoint(&waypoints, requested) {
            writeln!(
                stdout,
                "Found point: {latitude}, {longitude}",
                latitude = most_recent.latitude(),
                longitude = most_recent.longitude(),
            )?;
            let local = most_recent.time.to_timezone(timezone);
            writeln!(
                stdout,
                "  Time: {local} / {utc}Z",
                local = local.format(&time_format)?,
                utc = most_recent.time.format(&time_format)?,
            )?;
            if let Some(name) = most_recent.name {
                writeln!(stdout, "  Name: {name}")?;
            }
            if let Some(elevation) = most_recent.elevation() {
                writeln!(stdout, "  Elevation: {} meters", elevation.round() as u64)?;
            }
            if let Some(description) = most_recent.description {
                writeln!(stdout, "  Description: {description}")?;
            }
            if let Some(comment) = most_recent.comment {
                writeln!(stdout, "  Comment: {comment}")?;
            }
        } else {
            writeln!(stdout, "No point found.")?;
        }
    }
}

pub fn find_most_recent_waypoint<'a>(
    points: &'a [Waypoint<'a>],
    requested: OffsetDateTime,
) -> Option<&'a Waypoint<'a>> {
    let mut most_recent = None;

    for waypoint in points {
        if waypoint.time > requested {
            break;
        }

        most_recent = Some(waypoint);
    }

    most_recent
}

fn parse_time(buffer: &str) -> (u8, u8, u8) {
    let mut parts = buffer.trim().split(':');
    let hour = parts
        .next()
        .map_or(0, |part| part.parse().expect("Hour is invalid"));
    let minute = parts
        .next()
        .map_or(0, |part| part.parse().expect("Minute is invalid"));
    let second = parts
        .next()
        .map_or(0, |part| part.parse().expect("Second is invalid"));

    (hour, minute, second)
}
