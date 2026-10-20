use log::{Level, LevelFilter, Log, Metadata, Record};
use simplelog::{Config as SLConfig, SharedLogger};
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const CAPACITY: usize = 500;

#[derive(Clone, Debug)]
pub struct LogLine {
    pub level: String,
    pub message: String,
    pub time_ms: u128,
}

static BUFFER: Mutex<VecDeque<LogLine>> = Mutex::new(VecDeque::new());

pub fn push(level: &str, message: impl Into<String>) {
    let time_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    if let Ok(mut buf) = BUFFER.lock() {
        if buf.len() >= CAPACITY {
            buf.pop_front();
        }
        buf.push_back(LogLine {
            level: level.to_string(),
            message: message.into(),
            time_ms,
        });
    }
}

pub fn snapshot() -> Vec<LogLine> {
    BUFFER
        .lock()
        .map(|b| b.iter().cloned().collect())
        .unwrap_or_default()
}

pub fn clear() {
    if let Ok(mut b) = BUFFER.lock() {
        b.clear();
    }
}

pub struct BufferLogger;

impl Log for BufferLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            push(record.level().as_str(), format!("{}", record.args()));
        }
    }

    fn flush(&self) {}
}

impl SharedLogger for BufferLogger {
    fn level(&self) -> LevelFilter {
        LevelFilter::Debug
    }

    fn config(&self) -> Option<&SLConfig> {
        None
    }

    fn as_log(self: Box<Self>) -> Box<dyn Log> {
        self
    }
}
