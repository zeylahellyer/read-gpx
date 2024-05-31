use serde::Deserialize;
use time::OffsetDateTime;

#[derive(Clone, Debug, Deserialize)]
pub struct Root<'a> {
    #[serde(borrow, rename = "trk")]
    pub tracks: Vec<Track<'a>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Track<'a> {
    #[serde(borrow, rename = "trkseg")]
    pub segments: Vec<TrackSegment<'a>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TrackSegment<'a> {
    #[serde(borrow, rename = "trkpt")]
    pub waypoints: Vec<Waypoint<'a>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Waypoint<'a> {
    pub comment: Option<&'a str>,
    pub description: Option<&'a str>,
    pub name: Option<&'a str>,
    #[serde(rename = "@lat")]
    latitude: &'a str,
    #[serde(rename = "@lon")]
    longitude: &'a str,
    #[serde(rename = "ele")]
    elevation: Option<&'a str>,
    #[serde(with = "time::serde::rfc3339")]
    pub time: OffsetDateTime,
}

impl Waypoint<'_> {
    pub fn elevation(&self) -> Option<f32> {
        Some(self.elevation?.parse().unwrap())
    }

    pub fn latitude(&self) -> f32 {
        self.latitude.parse().unwrap()
    }

    pub fn longitude(&self) -> f32 {
        self.longitude.parse().unwrap()
    }
}
