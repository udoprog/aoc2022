use log::Log;

pub(crate) struct StdoutLogger;

impl Log for StdoutLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        println!(
            "{file}:{line}: {}: {}",
            record.level(),
            record.args(),
            file = record.file().unwrap_or_default(),
            line = record.line().unwrap_or_default()
        );
    }

    fn flush(&self) {}
}
