mod formatter;
mod model;

use anyhow::anyhow;
use model::{Root, Waypoint};
use std::{
    env,
    error::Error,
    fs,
    io::{self, BufRead, Write},
    path::Path,
};
use time::{OffsetDateTime, UtcOffset};

fn main() -> Result<(), Box<dyn Error>> {
    let Some(provided_path) = env::args().nth(1) else {
        println!("Path to the GPX file must be provided.");
        return Ok(());
    };

    let filepath = Path::new(&provided_path);
    let contents = fs::read_to_string(filepath)?;
    let gpx = quick_xml::de::from_str::<Root>(&contents)?;

    let (date_format, offset_format, time_format) =
        (formatter::date(), formatter::offset(), formatter::time());
    let mut waypoints = gpx
        .tracks
        .into_iter()
        .flat_map(|track| track.segments.into_iter().map(|segment| segment.waypoints))
        .flatten()
        .collect::<Vec<Waypoint>>();
    waypoints.sort_by(|a, b| a.time.cmp(&b.time));
    let from = waypoints
        .first()
        .ok_or_else(|| anyhow!("there are no waypoints in this file"))?
        .time;
    let to = waypoints
        .last()
        .ok_or_else(|| anyhow!("there are no waypoints in this file"))?
        .time;
    let offset = UtcOffset::current_local_offset()
        .map_err(|source| anyhow!("system timezone can't be determined: {source}"))?;
    let mut stdout = io::stdout().lock();
    writeln!(
        stdout,
        "System timezone is {}",
        offset.format(&offset_format)?
    )?;
    writeln!(
        stdout,
        "Opened {filepath} with {waypoints} points on {day} from {from} to {to}",
        filepath = filepath.display(),
        waypoints = waypoints.len(),
        day = from.date().format(&date_format)?,
        from = from.to_offset(offset).time().format(&time_format)?,
        to = to.to_offset(offset).time().format(&time_format)?,
    )?;
    writeln!(stdout,
        "Enter a time to print information about the most recent track in 24-hour time (eg 14:32:07)."
    )?;
    let mut stdin = io::stdin().lock();
    let mut buffer = String::new();

    loop {
        buffer.clear();
        write!(stdout, "> ")?;
        stdout.flush()?;
        stdin.read_line(&mut buffer)?;
        let (hour, minute, second) = match parse_time(&buffer) {
            Ok(time) => time,
            Err(source) => {
                writeln!(stdout, "{source}")?;
                continue;
            }
        };
        let requested = from
            .to_offset(offset)
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
            let local = most_recent.time.to_offset(offset);
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

fn parse_time(buffer: &str) -> anyhow::Result<(u8, u8, u8)> {
    let mut parts = buffer.trim().split(':');
    let hour = parts
        .next()
        .ok_or_else(|| anyhow!("hour must be provided"))?
        .parse()
        .map_err(|_| anyhow!("hour is invalid"))?;
    let minute = parts
        .next()
        .ok_or_else(|| anyhow!("minute must be provided"))?
        .parse()
        .map_err(|_| anyhow!("minute is invalid"))?;
    let second = match parts.next() {
        Some(value) => value.parse().map_err(|_| anyhow!("second is invalid"))?,
        None => 0,
    };

    Ok((hour, minute, second))
}
