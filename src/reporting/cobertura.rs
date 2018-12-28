use std::error;
use std::fmt;
use std::fs::File;
use std::io::{Cursor, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::{Path, PathBuf};

use quick_xml::{Writer, events::{BytesDecl, BytesEnd, BytesStart, Event}};

use chrono::offset::{Utc};

use config::{Config};
use traces::{CoverageStat, Trace, TraceMap};


// The XML structure for a cobatura report is roughly as follows:
//
// <coverage line-rate="0.0" branch-rate="0.0" version="1.9" timestamp="...">
//   <sources>
//     <source>PATH</source>
//     ...
//   </sources>
//
//   <packages>
//     <package name=".." line-rate="0.0" branch-rate="0.0" complexity="0.0">
//       <classes>
//         <class name="Main" filename="main.rs" line-rate="0.0" branch-rate="0.0" complexity="0.0">
//           <methods>
//             <method name="main" signature="()" line-rate="0.0" branch-rate="0.0">
//               <lines>
//                 <line number="1" hits="5" branch="false"/>
//                 <line number="3" hits="2" branch="true">
//                   <conditions>
//                     <condition number="0" type="jump" coverage="50%"/>
//                     ...
//                   </conditions>
//                 </line>
//               </lines>
//             </method>
//             ...
//           </methods>
//
//           <lines>
//             <line number="10" hits="4" branch="false"/>
//           </lines>
//         </class>
//         ...
//       </classes>
//     </package>
//     ...
//   </packages>
// </coverage>


pub fn report(config: &Config, traces: &TraceMap) -> Result<(), Error> {
    let result = Report::render(config, traces)?;
    result.export(config)
}


#[derive(Debug)]
pub enum Error {
    Unknown,
    ExportError,
}

impl error::Error for Error {

    #[inline]
    fn description(&self) -> &str {
        match self {
            Error::ExportError => "Export Error",
            Error::Unknown => "Unknown Error",
        }
    }

    #[inline]
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}


impl fmt::Display for Error {

    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ExportError => write!(f, "Export Error"),
            Error::Unknown => write!(f, "Unknown Error"),
        }
    }
}


#[derive(Debug)]
pub struct Report {
    timestamp:      i64,
    line_rate:      f64,
    branch_rate:    f64,
    sources:        Vec<PathBuf>,
    packages:       Vec<Package>,
}

impl Report {

    pub fn render(config: &Config, traces: &TraceMap) -> Result<Self, Error> {
        let timestamp   = Utc::now().timestamp();
        let sources     = render_sources(config);
        let packages    = render_packages(config, traces);
        let mut line_rate   = 0.0;
        let mut branch_rate = 0.0;

        if packages.len() > 0 {
            line_rate   = packages.iter()
                .map(|x| x.line_rate).sum::<f64>() / packages.len() as f64;
            branch_rate = packages.iter()
                .map(|x| x.branch_rate).sum::<f64>() / packages.len() as f64;
        }

        Ok(Report {
            timestamp:      timestamp,
            line_rate:      line_rate,
            branch_rate:    branch_rate,
            sources:        sources,
            packages:       packages,
        })
    }

    pub fn export(&self, config: &Config) -> Result<(), Error> {
        let mut file = File::create("cobertura.xml")
            .map_err(|_| Error::ExportError)?;

        let mut writer = Writer::new(Cursor::new(vec![]));
        writer.write_event(Event::Decl(BytesDecl::new(b"1.0", None, None)))
            .map_err(|_| Error::ExportError)?;
       
        let cov_tag = b"coverage";
        let mut cov = BytesStart::borrowed(cov_tag, cov_tag.len());
        cov.push_attribute(("line-rate", self.line_rate.to_string().as_ref()));
        cov.push_attribute(("branch-rate", self.branch_rate.to_string().as_ref()));
        cov.push_attribute(("version", "1.9"));
        let secs = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(s) => s.as_secs().to_string(),
            Err(_) => String::from("0"),
        };
        cov.push_attribute(("timestamp", secs.as_ref()));
        
        writer.write_event(Event::Start(cov))
            .map_err(|_| Error::ExportError)?;

        writer.write_event(Event::End(BytesEnd::borrowed(cov_tag)))
            .map_err(|_| Error::ExportError)?;

        let result = writer.into_inner().into_inner();
        file.write_all(&result)
            .map_err(|_| Error::ExportError)
    }
}

fn render_sources(config: &Config) -> Vec<PathBuf> {
    let render_source = |&x| Path::to_path_buf(x);

    config.manifest.parent().iter().map(render_source).collect()
}

#[derive(Debug)]
struct Package {
    name:           String,
    line_rate:      f64,
    branch_rate:    f64,
    complexity:     f64,
    classes:        Vec<Class>,
}


fn render_packages(config: &Config, traces: &TraceMap) -> Vec<Package> {
    let mut dirs: Vec<&Path> = traces.files().into_iter()
        .filter_map(|x| x.parent())
        .collect();

    dirs.dedup();

    dirs.into_iter()
        .map(|x| render_package(config, traces, x))
        .collect()
}


fn render_package(config: &Config, traces: &TraceMap, pkg: &Path) -> Package {
    let root = config.manifest.parent().unwrap_or(&config.manifest);
    let name = pkg.strip_prefix(root)
        .map(|x| Path::to_str(x))
        .unwrap_or_default()
        .unwrap_or_default();

    let line_cover = traces.covered_in_path(pkg) as f64;
    let line_rate = line_cover / (traces.coverable_in_path(pkg) as f64);

    Package {
        name:           name.to_string(),
        line_rate:      line_rate,
        branch_rate:    0.0,
        complexity:     0.0,
        classes:        render_classes(config, traces, pkg)
    }
}


#[derive(Debug)]
struct Class {
    name:           String,
    file_name:      String,
    line_rate:      f64,
    branch_rate:    f64,
    complexity:     f64,
    lines:          Vec<Line>,
    methods:        Vec<Method>,
}

fn render_classes(config: &Config, traces: &TraceMap, pkg: &Path) -> Vec<Class> {
    traces.files().iter()
        .filter(|x| x.parent() == Some(pkg))
        .map(|x| render_class(config, traces, x))
        .collect()
}

// TODO: Cobertura distinguishes between lines outside methods, and methods
// (which also contain lines). As there is currently no way to get traces from
// a particular function only, all traces are put into lines, and the vector
// of methods is empty.
//
// Until this is fixed, the render_method function will panic if called, as it
// cannot be properly implemented.
//
fn render_class(config: &Config, traces: &TraceMap, file: &Path) -> Class {
    let root = config.manifest.parent().unwrap_or(&config.manifest);
    let name = file.file_stem()
        .map(|x| x.to_str().unwrap())
        .unwrap_or_default()
        .to_string();

    let file_name = file.strip_prefix(root)
        .unwrap_or(file)
        .to_str()
        .unwrap()
        .to_string();

    let covered = traces.covered_in_path(file) as f64;
    let line_rate = covered / traces.coverable_in_path(file) as f64;
    let lines = traces.get_child_traces(file).iter()
        .map(|x| render_line(x))
        .collect();

    Class {
        name:           name,
        file_name:      file_name,
        line_rate:      line_rate,
        branch_rate:    0.0,
        complexity:     0.0,
        lines:          lines,
        methods:        vec![]
    }
}


#[derive(Debug)]
struct Method {
    name:           String,
    signature:      String,
    line_rate:      f64,
    branch_rate:    f64,
    lines:          Vec<Line>,
}

fn render_methods() -> Vec<Method> {
    panic!("Not yet implemented")
}

fn render_method() -> Method {
    panic!("Not yet implemented")
}


#[derive(Debug)]
enum Line {
    Plain {
        number:     usize,
        hits:       usize,
    },

    Branch {
        number:     usize,
        hits:       usize,
        conditions: Vec<Condition>,
    }
}

fn render_line(trace: &Trace) -> Line {
    match &trace.stats {
        CoverageStat::Line(hits) => Line::Plain {
            number: trace.line as usize,
            hits:   *hits as usize
        },

        // TODO: Branches in cobertura are given a fresh number as a label,
        // which would require having some form of context when rendering.
        //
        _ => panic!("Not currently supported")
    }
}


#[derive(Debug)]
struct Condition {
    number:         usize,
    cond_type:      ConditionType,
    coverage:       f64,
}


// Condition types
#[derive(Debug)]
enum ConditionType {
    Jump,
}

