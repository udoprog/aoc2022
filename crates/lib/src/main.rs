use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
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

fn main() -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.stdout(Stdio::piped());
    cmd.arg("build");
    cmd.arg("--release");
    cmd.args(&["-p", "y2022"]);
    cmd.args(&["--message-format", "json"]);

    let output = cmd.spawn()?;

    let output = output.stdout.context("missing stdout")?;
    let mut output = serde_json::Deserializer::from_reader(output).into_iter();

    let mut executables = Vec::new();

    while let Some(value) = output.next() {
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

    executables.sort_by(|a, b| a.name.cmp(&b.name));

    let args = std::env::args_os().skip(1).collect::<Vec<_>>();

    for e in executables {
        println!("running: {}", e.name);
        let mut cmd = Command::new(e.path);
        cmd.args(&args[..]);
        let status = cmd.status()?;
        println!("{status}");
    }

    Ok(())
}
