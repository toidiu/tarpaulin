extern crate tera;

use std::error;
use std::fmt;

use config::{Config};
use traces::{TraceMap, CoverageStat};


lazy_static! {
    static ref TERA: tera::Tera = compile_templates!("templates/**/*");
}

pub fn report(config: &Config, traces: &TraceMap) -> Result<(), Error> {
    unimplemented!()
}

pub struct Report {
    data: String,
}

impl Report {

    #[inline]
    pub fn render(_config: &Config, traces: &TraceMap) -> Result<Self, Error> {
        TERA.render("templates/report.html", traces)
            .map(|s| Report { data: s })
            .map_err(|_| Error::Unknown)
    }

    pub fn export(&self, config: &Config) -> Result<(), Error> {
        unimplemented!();
    }
}


#[derive(Debug)]
pub enum Error {
    Unknown,
}

impl error::Error for Error {

    #[inline]
    fn description(&self) -> &str {
        ""
    }

    #[inline]
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl fmt::Display for Error {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

