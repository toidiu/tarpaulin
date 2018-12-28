extern crate curl;

use std::collections::{HashMap};
use std::error;
use std::fmt;

use coveralls_api::*;

use config::{Config};
use traces::{TraceMap, CoverageStat};

pub fn report(config: &Config, traces: &TraceMap) -> Result<(), Error> {
    let mut report = Report::render(config, traces)?;
    report.export(config)
}

fn get_coveralls_id(config: &Config) -> Result<Identity, Error> {
    let key = config.coveralls.clone().ok_or(Error::NoKey)?;

    Ok(match config.ci_tool {
        Some(ref service)   =>
            Identity::ServiceToken(Service {
                service_name:   service.clone(),
                service_job_id: key.clone(),
            }),

        None                =>
            Identity::RepoToken(key.clone())
    })
}


pub struct Report {
    report: CoverallsReport,
}


impl Report {

    pub fn render(config: &Config, traces: &TraceMap) -> Result<Self, Error> {
        let id = get_coveralls_id(config)?;
        let mut report = CoverallsReport::new(id);

        for file in traces.files() {
            let rel_path = config.strip_project_path(file);
            let file_cov = traces.get_child_traces(file);
            let mut lines = HashMap::new();

            for cov in file_cov {
                match cov.stats {
                    CoverageStat::Line(hits) => {
                        lines.insert(cov.line as usize, hits as usize);
                    },
                    _ => {
                        // TODO: Support other coverage stats
                        println!("Unsupported coverage statistic");
                    },
                }

                if let Ok(source) = Source::new(&rel_path, file, &lines, &None, false) {
                    report.add_source(source);
                }
            }
        }

        Ok(Report { report: report })
    }

    pub fn export(&mut self, config: &Config) -> Result<(), Error> {
        match config.report_uri {
            Some(ref uri) => {
                println!("Sending report to {}", uri);
                self.report.send_to_endpoint(uri).map_err(Error::Export)
            },
            None => {
                println!("Sending report to coveralls.io");
                self.report.send_to_coveralls().map_err(Error::Export)
            },
        }
    }
}


#[derive(Debug)]
pub enum Error {
    Export(curl::Error),
    NoKey,
}

impl error::Error for Error {

    #[inline]
    fn description(&self) -> &str {
        match self {
            Error::Export(e)    => e.description(),
            Error::NoKey        => "No Coveralls Key Provided",
        }
    }

    #[inline]
    fn cause(&self) -> Option<&error::Error> {
        match self {
            Error::Export(e)    => Some(e),
            Error::NoKey        => None,
        }
    }
}

impl fmt::Display for Error {

    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Export(_)    => write!(f, "Could Not Export Report"),
            Error::NoKey        => write!(f, "No Coveralls Key Provided"),
        }
    }
}

