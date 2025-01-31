#[cfg(feature="chrono")]
use chrono::{DateTime, offset::TimeZone};
#[cfg(not(feature="chrono"))]
use std::time::SystemTime;

// Copied from the std libary, that way we are not limited to a minimum of rust 1.47
pub const TAU: f64 = 6.28318530717958647692528676655900577_f64;

const MOON_SYNODIC_PERIOD: f64 = 29.530588853; // Period of moon cycle in days.
const MOON_SYNODIC_OFFSET: f64 = 2451550.26; // Reference cycle offset in days.
const MOON_DISTANCE_PERIOD: f64 = 27.55454988; // Period of distance oscillation
const MOON_DISTANCE_OFFSET: f64 = 2451562.2;
const MOON_LATITUDE_PERIOD: f64 = 27.212220817; // Latitude oscillation
const MOON_LATITUDE_OFFSET: f64 = 2451565.2;
const MOON_LONGITUDE_PERIOD: f64 = 27.321582241; // Longitude oscillation
const MOON_LONGITUDE_OFFSET: f64 = 2451555.8;

// Names of lunar phases
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Phase {
    New,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    Full,
    WainingGibbous,
    LastQuarter,
    WaningCrescent,
}
// Names of Zodiac constellations
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Zodiac {
    Pisces,
    Aries,
    Taurus,
    Gemini,
    Cancer,
    Leo,
    Virgo,
    Libra,
    Scorpio,
    Sagittarius,
    Capricorn,
    Aquarius,
}

// Ecliptic angles of Zodiac constellations
const ZODIAC_ANGLES: [f64; 12] = [
    33.18, 51.16, 93.44, 119.48, 135.30, 173.34, 224.17, 242.57, 271.26,
    302.49, 311.72, 348.58,
];

impl Zodiac {
    pub fn from_long(long: f64) -> Self {
        use crate::Zodiac::*;
        ZODIAC_ANGLES
            .iter()
            .enumerate()
            .find_map(|(i, angle)| {
                if long < *angle {
                    Some(match i {
                        0 => Pisces,
                        1 => Aries,
                        2 => Taurus,
                        3 => Gemini,
                        4 => Cancer,
                        5 => Leo,
                        6 => Virgo,
                        7 => Libra,
                        8 => Scorpio,
                        9 => Sagittarius,
                        10 => Capricorn,
                        11 => Aquarius,
                        _ => unimplemented!(),
                    })
                } else {
                    None
                }
            })
            .unwrap_or_else(|| Pisces)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MoonPhase {
    pub j_date: f64,
    pub phase: f64,                // 0 - 1, 0.5 = full
    pub age: f64,                  // Age in days of current cycle
    pub fraction: f64,             // Fraction of illuminated disk
    pub distance: f64,             // Moon distance in earth radii
    pub latitude: f64,             // Moon ecliptic latitude
    pub longitude: f64,            // Moon ecliptic longitude
    pub phase_name: Phase,          // New, Full, etc.
    pub zodiac_name: Zodiac,        // Constellation
}

#[cfg(feature="chrono")]
fn julian_date<Tz: TimeZone>(time: DateTime<Tz>) -> f64 {
    let secs = time.timestamp_micros() as f64 / 1_000_000.0;
    julian_date_from_seconds(secs)
}

#[cfg(not(feature="chrono"))]
fn julian_date(time: SystemTime) -> f64 {
    let secs = match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => duration.as_secs_f64(),
        Err(earlier) => -1. * earlier.duration().as_secs_f64(),
    };
    julian_date_from_seconds(secs)
}

fn julian_date_from_seconds(secs: f64) -> f64 {
    secs / 86400. + 2440587.5
}

impl MoonPhase {
    #[cfg(feature="chrono")]
    pub fn new<Tz: TimeZone>(time: DateTime<Tz>) -> Self {
        let j_date = julian_date(time);
        Self::_new(j_date)
    }

    #[cfg(not(feature="chrono"))]
    pub fn new(time: SystemTime) -> Self {
        let j_date = julian_date(time);
        Self::_new(j_date)
    }

    pub fn from_secs(secs: i64) -> Self {
        Self::from_secs_float(secs as f64)
    }

    pub fn from_secs_float(secs: f64) -> Self {
        let j_date = julian_date_from_seconds(secs);
        Self::_new(j_date)
    }

    fn _new(j_date: f64) -> Self {
        // Calculate illumination (synodic) phase.
        // From number of days since new moon on Julian date MOON_SYNODIC_OFFSET
        // (1815UTC January 6, 2000), determine remainder of incomplete cycle.
        let phase =
            ((j_date - MOON_SYNODIC_OFFSET) / MOON_SYNODIC_PERIOD).fract();
        // Calculate age and illuination fraction.
        let age = phase * MOON_SYNODIC_PERIOD;
        let fraction = (1. - (TAU * phase)).cos() / 2.;
        let mut phase_mod = (phase * 8.).round() % 8.;
        if phase_mod < 0. { // Otherwise, values lower than 0 would simply cause New
            phase_mod += 8.;
        }
        let phase_name = match phase_mod as usize {
            0 => Phase::New,
            1 => Phase::WaxingCrescent,
            2 => Phase::FirstQuarter,
            3 => Phase::WaxingGibbous,
            4 => Phase::Full,
            5 => Phase::WainingGibbous,
            6 => Phase::LastQuarter,
            7 => Phase::WaningCrescent,
            _ => {panic!("This should be unreachable")}
        };
        // Calculate distance fro anoalistic phase.
        let distance_phase =
            ((j_date - MOON_DISTANCE_OFFSET) / MOON_DISTANCE_PERIOD).fract();
        let distance_phase_tau = TAU * distance_phase;
        let phase_tau = 2. * TAU * phase;
        let phase_distance_tau_difference = phase_tau - distance_phase_tau;
        let distance = 60.4
            - 3.3 * distance_phase_tau.cos()
            - 0.6 * (phase_distance_tau_difference).cos()
            - 0.5 * (phase_tau).cos();

        // Calculate ecliptic latitude from nodal (draconic) phase.
        let lat_phase =
            ((j_date - MOON_LATITUDE_OFFSET) / MOON_LATITUDE_PERIOD).fract();
        let latitude = 5.1 * (TAU * lat_phase).sin();

        // Calculate ecliptic longitude ffrom sidereal motion.
        let long_phase =
            ((j_date - MOON_LONGITUDE_OFFSET) / MOON_LONGITUDE_PERIOD).fract();
        let longitude = (360. * long_phase
            + 6.3 * (distance_phase_tau).sin()
            + 1.3 * (phase_distance_tau_difference).sin()
            + 0.7 * (phase_tau).sin())
            % 360.;

        let zodiac_name = Zodiac::from_long(longitude);
        MoonPhase {
            j_date,
            phase,
            age,
            fraction,
            distance,
            latitude,
            longitude,
            phase_name,
            zodiac_name,
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use super::Phase::*;
    #[cfg(feature="chrono")]
    use chrono::prelude::*;
    #[cfg(not(feature="chrono"))]
    use std::time::SystemTime;

    //use pretty_assertions::{assert_eq};

    #[cfg(feature="chrono")]
    static CHRONO_TEST_CASES: [(&str, Phase); 13] = [
        ("1999-01-02T02:49:00+00:00", Full),
        ("1999-07-20T09:00:00+00:00", FirstQuarter),

        ("2000-01-06T18:13:00+00:00", New), // Our offset is based on that
        ("2000-01-14T13:34:00+00:00", FirstQuarter),
        ("2000-01-21T04:40:00+00:00", Full),
        ("2000-01-28T07:56:00+00:00", LastQuarter),
        ("2000-12-25T17:21:00+00:00", New),

        ("2022-01-02T18:33:00+00:00", New),
        ("2022-01-15T23:49:00+00:00", WaxingGibbous),
        ("2022-01-16T00:00:00+00:00", Full),
        ("2022-01-17T23:48:00+00:00", Full),
        ("2022-01-18T23:59:00+00:00", Full),
        ("2022-01-19T16:45:00+00:00", WainingGibbous),
    ];

    #[test]
    #[cfg(feature="chrono")]
    fn phase_detection() {
        // Times taken from https://www.timeanddate.com/moon/phases/timezone/utc
        for (time, exp) in &CHRONO_TEST_CASES {
            let time = DateTime::parse_from_rfc3339(time).unwrap();
            let moon_phase = MoonPhase::new(time);
            assert_eq!(moon_phase.phase_name, *exp, "Failed for {}", time);
        }
    }

    #[test]
    #[cfg(feature="chrono")]
    pub fn chrono_seconds_same() {
        for (time, _) in &CHRONO_TEST_CASES {
            let time = DateTime::parse_from_rfc3339(time).unwrap();
            let seconds = time.timestamp();
            let moon_phase_datetime = MoonPhase::new(time);
            let moon_phase_seconds = MoonPhase::from_secs(seconds);
            assert_eq!(
                moon_phase_datetime, moon_phase_seconds,
                "Failed for DateTime: {} / Seconds: {}",
                time, seconds
            );
        }
    }

    #[test]
    #[cfg(not(feature="chrono"))]
    fn phase_detection() {
        let testcases = [
            ( 915245340.0, Full),	            // 1999-01-02T02:49:00+00:00
            ( 932461200.0, FirstQuarter),	    // 1999-07-20T09:00:00+00:00

            ( 947182380.0, New),	            // 2000-01-06T18:13:00+00:00
            ( 947856840.0, FirstQuarter),	    // 2000-01-14T13:34:00+00:00
            ( 948429600.0, Full),	            // 2000-01-21T04:40:00+00:00
            ( 949046160.0, LastQuarter),        // 2000-01-28T07:56:00+00:00
            ( 977764860.0, New),                // 2000-12-25T17:21:00+00:00

            (1641148380.0, New),                // 2022-01-02T18:33:00+00:00
            (1642290540.0, WaxingGibbous),	    // 2022-01-15T23:49:00+00:00
            (1642291200.0, Full),               // 2022-01-16T00:00:00+00:00
            (1642463280.0, Full),               // 2022-01-17T23:48:00+00:00
            (1642550340.0, Full),               // 2022-01-18T23:59:00+00:00
            (1642610700.0, WainingGibbous),     // 2022-01-19T16:45:00+00:00
        ];

        for (secs, exp) in &testcases {
            let moon_phase = MoonPhase::from_secs_float(*secs);
            assert_eq!(&moon_phase.phase_name, exp, "Failed for {}", secs);
        }
    }

    #[test]
    #[cfg(feature="chrono")]
    fn test_create() {
        MoonPhase::new(Local::now()); // Just make sure it's not crashing
        MoonPhase::new(Utc::now()); // Just make sure it's not crashing
    }

    #[test]
    #[cfg(not(feature="chrono"))]
    fn test_create() {
        MoonPhase::new(SystemTime::now()); // Just make sure it's not crashing
    }
}
