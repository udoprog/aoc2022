use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use lib::cli::Report;
use serde::{de::IntoDeserializer, Deserialize};

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
    args: Vec<OsString>,
}

impl Opts {
    /// Parse CLI options.
    pub fn parse() -> Result<Self> {
        let mut opts = Self::default();
        let mut it = std::env::args_os().skip(1);

        for arg in it.by_ref() {
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
                "--" => {
                    break;
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

fn main() -> Result<()> {
    let opts = Opts::parse()?;

    let mut cmd = Command::new("cargo");
    cmd.stdout(Stdio::piped());
    cmd.arg("build");
    cmd.arg("--release");
    cmd.args(["-p", "y2022"]);
    cmd.args(["--message-format", "json"]);

    let mut child = cmd.spawn()?;

    let output = child.stdout.take().context("missing stdout")?;
    let output = serde_json::Deserializer::from_reader(output).into_iter();

    let mut executables = Vec::new();

    for value in output {
        let value: serde_json::Value = value?;

        if !matches!(
            value.get("reason").and_then(|d| d.as_str()),
            Some("compiler-artifact")
        ) {
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

        executables.push(Executable {
            name: artifact.target.name,
            path,
        });
    }

    let _ = child.wait()?;

    executables.sort_by(|a, b| a.name.cmp(&b.name));

    let mut total = Report::default();

    for e in executables {
        let mut cmd = Command::new(e.path);
        cmd.stdout(Stdio::piped());
        cmd.args(&opts.args[..]);
        cmd.arg("--json");

        let mut child = cmd.spawn()?;
        let output = child.stdout.take().context("missing stdout")?;
        let output = serde_json::Deserializer::from_reader(output).into_iter();

        for value in output {
            let value: serde_json::Value = value?;

            match value.get("type").and_then(|d| d.as_str()) {
                Some("report") => {
                    let report = Data::<Report>::deserialize(value.into_deserializer())?.data;

                    if !opts.quiet {
                        println!("{name}: {report}", name = e.name);
                    }

                    total += report;
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

        if opts.is_verbose() {
            println!("{name}: {status}", name = e.name);
        }
    }

    println!("total: {total}");
    Ok(())
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
        match self.kind.as_str() {
            "error" => true,
            _ => false,
        }
    }
}
