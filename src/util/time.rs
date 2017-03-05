use std::time::Duration;
use std::fmt;

pub trait DurationExt {
    fn display_ms(&self) -> DisplayMs;
}

impl DurationExt for Duration {
    fn display_ms(&self) -> DisplayMs {
        DisplayMs(*self)
    }
}

pub struct DisplayMs(Duration);

impl fmt::Display for DisplayMs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let secs = self.0.as_secs();
        let nanos = self.0.subsec_nanos();

        let ms = (secs as f64) * 1000.0 + ((nanos / 1000) as f64 / 1000.0);
        // TODO: precision!
        format!("{:.3}ms", ms).fmt(f)
    }
}
