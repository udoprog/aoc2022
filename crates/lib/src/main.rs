use std::collections::HashSet;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, ExitCode, ExitStatus, Stdio};

use anyhow::{bail, Context, Result};
use lib::cli::Report;
use serde::{de::IntoDeserializer, Deserialize};

const LIB_NAME: &str = env!("CARGO_CRATE_NAME");

#[derive(Debug, Deserialize)]
struct Target {
    name: String,
    kind: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Artifact {
    target: Target,
    executable: Option<PathBuf>,
}

struct Executable {
    name: String,
    path: PathBuf,
}

#[derive(Default)]
struct Opts {
    quiet: bool,
    verbose: bool,
    release: bool,
    no_prod: bool,
    project: Option<String>,
    args: Vec<OsString>,
    names: HashSet<String>,
}

impl Opts {
    /// Parse CLI options.
    pub fn parse() -> Result<Self> {
        let mut opts = Self::default();
        let mut it = std::env::args_os().skip(1);

        while let Some(arg) = it.next() {
            let Some(arg) = arg.to_str() else {
                bail!("non-utf8 argument");
            };

            match arg {
                "-q" | "--quiet" => {
                    opts.quiet = true;
                }
                "-V" | "--verbose" => {
                    opts.verbose = true;
                }
                "-p" => {
                    let project = it.next().context("missing argument to `-p`")?;
                    opts.project = Some(project.to_string_lossy().into_owned());
                }
                "--release" => {
                    opts.release = true;
                }
                "--no-prod" => {
                    opts.no_prod = true;
                }
                "--" => {
                    break;
                }
                name if !name.starts_with('-') => {
                    opts.names.insert(name.to_owned());
                }
                other => {
                    bail!("unsupported argument: {other}");
                }
            }
        }

        opts.args.extend(it);
        Ok(opts)
    }

    /// Test if options are verbose.
    fn is_verbose(&self) -> bool {
        self.verbose && !self.quiet
    }
}

fn main() -> Result<ExitCode> {
    let opts = Opts::parse()?;

    let (mut executables, status) = build_project(&opts)?;

    if !opts.names.is_empty() {
        executables.retain(|e| opts.names.contains(&e.name));
    }

    if executables.is_empty() {
        bail!("no executables!");
    }

    let mut reports = Vec::new();
    let mut all = ExitCode::SUCCESS;

    for e in executables {
        if opts.verbose {
            println!("Running: {}", e.name);
        }

        let mut cmd = Command::new(e.path);
        cmd.stdout(Stdio::piped());
        cmd.args(&opts.args[..]);
        cmd.arg("--json");

        let mut child = cmd.spawn()?;
        let output = child.stdout.take().context("missing stdout")?;
        let output = serde_json::Deserializer::from_reader(output).into_iter();

        for value in output {
            let Ok(value): Result<serde_json::Value, _> = value else {
                continue;
            };

            match value.get("type").and_then(|d| d.as_str()) {
                Some("report") => {
                    let report = Data::<Report>::deserialize(value.into_deserializer())?.data;
                    println!("# {name}", name = e.name);
                    println!("{report}");
                    reports.push(report);
                }
                Some("message") => {
                    let message = Data::<Message>::deserialize(value.into_deserializer())?.data;

                    if opts.is_verbose() || message.is_important() {
                        println!(
                            "{name}: {kind}: {output}",
                            name = e.name,
                            kind = message.kind,
                            output = message.output
                        );
                    }
                }
                _ => {}
            }
        }

        let status = child.wait()?;

        if !status.success() {
            all = ExitCode::FAILURE;
        }

        if opts.is_verbose() {
            println!("{name}: {status}", name = e.name);
        }
    }

    if !reports.is_empty() {
        let mut total = Report::default();

        for t in &reports {
            total += t;
        }

        println!("# totals (each sample added together)");
        println!("{total}");
    }

    if !status.success() {
        return Ok(ExitCode::FAILURE);
    }

    Ok(all)
}

/// Build the project and return status.
fn build_project(opts: &Opts) -> Result<(Vec<Executable>, ExitStatus)> {
    let mut cmd = Command::new("cargo");
    cmd.stdout(Stdio::piped());
    cmd.arg("build");

    if opts.release {
        cmd.arg("--release");
    }

    if opts.release && !opts.no_prod {
        cmd.env("RUSTFLAGS", "--cfg prod");
    }

    if let Some(project) = &opts.project {
        cmd.args(&["-p", project.as_str()]);
    } else {
        cmd.arg("--all");
    }

    cmd.args(["--message-format", "json"]);

    let mut child = cmd.spawn()?;
    let output = child.stdout.take().context("missing stdout")?;
    let output = serde_json::Deserializer::from_reader(output).into_iter();

    let mut executables = Vec::new();

    for value in output {
        let value: serde_json::Value = value?;

        let reason = value.get("reason").and_then(|d| d.as_str());

        if !matches!(reason, Some("compiler-artifact")) {
            continue;
        }

        let artifact: Artifact = Artifact::deserialize(value.into_deserializer())?;

        let [kind] = &artifact.target.kind[..] else {
            continue;
        };

        if kind != "bin" {
            continue;
        }

        let path = artifact.executable.context("missing executable")?;

        // Skip over self.
        if artifact.target.name == LIB_NAME {
            continue;
        }

        executables.push(Executable {
            name: artifact.target.name,
            path,
        });
    }

    let status = child.wait()?;
    executables.sort_by(|a, b| a.name.cmp(&b.name));
    Ok((executables, status))
}

#[derive(Deserialize)]
struct Data<T> {
    data: T,
}

#[derive(Deserialize)]
struct Message {
    kind: String,
    output: String,
}

impl Message {
    fn is_important(&self) -> bool {
        matches!(self.kind.as_str(), "error")
    }
}
