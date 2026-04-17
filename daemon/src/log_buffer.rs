use chrono::Utc;
use std::collections::VecDeque;
use std::sync::Mutex;
use tracing_subscriber::field::Visit;

/// Ring buffer for capturing recent log entries
pub struct LogBuffer {
    entries: Mutex<VecDeque<LogEntry>>,
    capacity: usize,
}

#[derive(Clone, serde::Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
}

impl LogBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    pub fn push(&self, level: &str, target: &str, message: &str) {
        let entry = LogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: level.to_string(),
            target: target.to_string(),
            message: message.to_string(),
        };
        let mut entries = self.entries.lock().unwrap();
        if entries.len() >= self.capacity {
            entries.pop_front();
        }
        entries.push_back(entry);
    }

    pub fn get_recent(&self, limit: usize) -> Vec<LogEntry> {
        let entries = self.entries.lock().unwrap();
        let start = entries.len().saturating_sub(limit);
        entries.range(start..).cloned().collect()
    }

    pub fn count(&self) -> usize {
        self.entries.lock().unwrap().len()
    }
}

/// Tracing layer that writes to LogBuffer
pub struct LogBufferLayer {
    buffer: std::sync::Arc<LogBuffer>,
}

impl LogBufferLayer {
    pub fn new(buffer: std::sync::Arc<LogBuffer>) -> Self {
        Self { buffer }
    }
}

impl<S> tracing_subscriber::Layer<S> for LogBufferLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = MessageVisitor(String::new());
        event.record(&mut visitor);

        let level = match *event.metadata().level() {
            tracing::Level::ERROR => "ERROR",
            tracing::Level::WARN => "WARN",
            tracing::Level::INFO => "INFO",
            tracing::Level::DEBUG => "DEBUG",
            tracing::Level::TRACE => "TRACE",
        };

        let target = event.metadata().target();
        let msg = visitor.0;

        // Skip internal noisy logs
        if !msg.is_empty() {
            self.buffer.push(level, target, &msg);
        }
    }
}

struct MessageVisitor(String);

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{:?}", value);
        }
    }
}
