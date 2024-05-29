# read-gpx-cli

Interactive CLI to find the GPS location for given times in a GPX file.

```
$ read-gpx /Volumes/home/gpx/2024-05-28_13-41_Tue.gpx
System timezone is America/New_York
Opened /Volumes/home/gpx/2024-05-28_13-41_Tue.gpx with 1168 points on 2024-05-28 from 13:41:43 to 17:42:05
Enter a time to print information about the most recent track in 24-hour time (eg 14:32:07).
> 15:04
Found point: 29.0338979, -50.7510621
  Time: 15:03:51 / 19:03:51Z
  Elevation: 1014 meters
> 16:14
Found point: 29.3829786, -49.9185463
  Time: 16:13:52 / 20:13:52Z
  Elevation: 987 meters
```

## Motivation

After nature photography trips I will often have multitudes of observations of
species to upload, for which I need the time and GPS location of when and where
the species was observed. I use OsmAnd to record GPX tracks, and would have to
manually search the XML files for waypoint times to find the latitudes and
longitudes. This prompted me to build a CLI to find the most recent recorded
location for a given time.

## Install

Cargo:

```sh
$ cargo install --git https://github.com/zeylahellyer/read-gpx-cli
```
