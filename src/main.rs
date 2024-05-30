mod formatter;

use gpx::{Gpx, Waypoint};
use std::{
    env,
    error::Error,
    fs,
    io::{self, BufRead, BufReader, Write},
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
    let gpx = read_gpx_file(filepath)?;

    let (date_format, time_format) = (formatter::date(), formatter::time());
    let points = gpx
        .tracks
        .into_iter()
        .flat_map(|track| track.segments.into_iter().map(|segment| segment.points))
        .flatten()
        .collect::<Vec<Waypoint>>();
    let from = OffsetDateTime::from(points.first().unwrap().time.unwrap());
    let to = OffsetDateTime::from(points.last().unwrap().time.unwrap());
    let timezone = time_tz::system::get_timezone()?;
    println!("System timezone is {}", timezone.name());
    println!(
        "Opened {filepath} with {points} points on {day} from {from} to {to}",
        filepath = filepath.display(),
        points = points.len(),
        day = from.date().format(&date_format)?,
        from = from.to_timezone(timezone).time().format(&time_format)?,
        to = to.to_timezone(timezone).time().format(&time_format)?,
    );
    println!(
        "Enter a time to print information about the most recent track in 24-hour time (eg 14:32:07)."
    );
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut buffer = String::new();
    let date = OffsetDateTime::from(points.first().unwrap().time.unwrap());

    loop {
        print!("> ");
        io::stdout().flush()?;
        stdin.read_line(&mut buffer)?;
        let (hour, minute, second) = parse_time(&buffer);
        buffer.clear();
        let requested = date
            .to_timezone(timezone)
            .replace_hour(hour)?
            .replace_minute(minute)?
            .replace_second(second)?;
        // TODO: This is horribly inefficient, but I wrote a working program in
        // an hour.
        let mut points = points.clone();
        points.retain(|point| OffsetDateTime::from(point.time.unwrap()) < requested);
        points.sort_by(|a, b| {
            let a = OffsetDateTime::from(a.time.unwrap());
            let b = OffsetDateTime::from(b.time.unwrap());

            b.cmp(&a)
        });
        if let Some(most_recent) = points.first() {
            let point = most_recent.point();
            println!(
                "Found point: {latitude}, {longitude}",
                latitude = point.y(),
                longitude = point.x(),
            );
            if let Some(time) = most_recent.time {
                let utc = OffsetDateTime::from(time);
                let local = utc.to_timezone(timezone);
                println!(
                    "  Time: {local} / {utc}Z",
                    local = local.format(&time_format)?,
                    utc = utc.format(&time_format)?,
                );
            }
            if let Some(name) = most_recent.name.as_deref() {
                println!("  Name: {name}");
            }
            if let Some(elevation) = most_recent.elevation {
                println!("  Elevation: {} meters", elevation.round() as u64);
            }
            if let Some(speed) = most_recent.speed {
                println!("  Speed: {speed} meters/second");
            }
            if let Some(description) = most_recent.description.as_deref() {
                println!("  Description: {description}");
            }
            if let Some(comment) = most_recent.comment.as_deref() {
                println!("  Comment: {comment}");
            }
        } else {
            println!("No point found.");
        }
    }
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

/// Read GPX from a filepath.
fn read_gpx_file(filepath: &Path) -> Result<Gpx, Box<dyn Error>> {
    // The gpx crate doesn't support empty
    // <copyright><author></author></copyright> tags produced by OsmAnd, so we
    // strip those out.
    let contents = fs::read_to_string(filepath)?.replace(
        "<copyright>\n            <author></author>\n        </copyright>",
        "",
    );
    let mut reader = BufReader::new(contents.as_bytes());
    gpx::read(&mut reader).map_err(Into::into)
}
