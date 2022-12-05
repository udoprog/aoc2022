use core::fmt;
use std::io::{self, Write};

use serde::Serialize;

use crate::cli::Report;

pub(crate) struct Output<O> {
    out: O,
    kind: OutputKind,
}

pub(crate) enum OutputKind {
    Json,
    Normal,
}

impl<O> Output<O>
where
    O: Write,
{
    pub(crate) fn new(out: O, kind: OutputKind) -> Self {
        Self { out, kind }
    }

    pub(crate) fn info(&mut self, m: impl fmt::Display) -> io::Result<()> {
        self.message(MessageKind::Info, m)
    }

    pub(crate) fn error(&mut self, m: impl fmt::Display) -> io::Result<()> {
        self.message(MessageKind::Error, m)
    }

    pub(crate) fn report(&mut self, report: &Report) -> io::Result<()> {
        match &self.kind {
            OutputKind::Json => {
                self.json(&Line {
                    ty: LineType::Report,
                    data: report,
                })?;
            }
            OutputKind::Normal => {
                writeln!(self.out, "{report}")?;
            }
        }

        Ok(())
    }

    fn message(&mut self, kind: MessageKind, m: impl fmt::Display) -> io::Result<()> {
        match &self.kind {
            OutputKind::Json => {
                self.json(&Line {
                    ty: LineType::Message,
                    data: Message { output: m, kind },
                })?;
            }
            OutputKind::Normal => {
                writeln!(self.out, "{kind}: {m}")?;
            }
        }

        Ok(())
    }

    fn json<T>(&mut self, m: &T) -> io::Result<()>
    where
        T: Serialize,
    {
        serde_json::to_writer(&mut self.out, m)?;
        writeln!(self.out)?;
        Ok(())
    }
}

#[derive(Serialize)]
struct Line<T> {
    #[serde(rename = "type")]
    ty: LineType,
    data: T,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
enum LineType {
    Message,
    Report,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
enum MessageKind {
    Info,
    Error,
}

impl fmt::Display for MessageKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageKind::Info => write!(f, "info"),
            MessageKind::Error => write!(f, "error"),
        }
    }
}

struct Message<T> {
    output: T,
    kind: MessageKind,
}

impl<T> Serialize for Message<T>
where
    T: fmt::Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("output", &DisplayString(&self.output))?;
        map.end()
    }
}

struct DisplayString<T>(T);

impl<T> Serialize for DisplayString<T>
where
    T: fmt::Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.0)
    }
}
