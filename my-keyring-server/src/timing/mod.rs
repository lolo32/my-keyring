use std::{fmt, time::Duration};

use saphir::prelude::*;

pub fn extract_timing(req: &Request) -> Timing {
    req.extensions().get::<Timing>().cloned().unwrap()
}

#[derive(Debug, Clone)]
pub struct Timing(Vec<Time>);

impl Timing {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add_timing(
        &mut self,
        name: &str,
        duration: impl Into<Option<Duration>>,
        desc: impl Into<Option<String>>,
    ) -> &Self {
        self.0.push(Time {
            name: name.to_owned(),
            duration: duration.into(),
            desc: desc.into(),
        });
        self
    }
}

impl fmt::Display for Timing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let timing = self
            .0
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{}", timing)
    }
}

#[derive(Debug, Clone)]
struct Time {
    name: String,
    duration: Duration,
    desc: Option<String>,
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(desc) = &self.desc {
            write!(f, ";desc=\"{}\";", desc.replace("\"", "\\\""))?;
        }
        if let Some(dur) = &self.duration {
            let millis = self.duration.as_millis();
            write!(
                f,
                ";dur={}.{:03}",
                millis,
                self.duration.as_micros() - millis * 1_000
            )
        }
    }
}
