//! `tracing_subscriber` writer that emits to the browser console.
//!
//! Replaces the `tracing-subscriber-wasm` crate, whose only remaining
//! dependency (`gloo 0.8` → `gloo-worker 0.2` → `anymap2`) is unmaintained and
//! trips `cargo-deny` (`RUSTSEC-2026-0319`, no fixed version available). Both
//! microfrontends already depend on `gloo 0.12`, so the old crate was also
//! pulling a second, stale copy of `gloo` into every WASM bundle.

use std::io::{self, Write};

use tracing::Level;
use tracing_subscriber::fmt::MakeWriter;

/// [`MakeWriter`] that forwards each event to the matching `console` method.
///
/// Level mapping: `TRACE`/`DEBUG` → `console.debug`, `INFO` → `console.log`,
/// `WARN` → `console.warn`, `ERROR` → `console.error`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ConsoleMakeWriter;

impl<'a> MakeWriter<'a> for ConsoleMakeWriter {
    type Writer = ConsoleWriter;

    fn make_writer(&'a self) -> Self::Writer {
        ConsoleWriter {
            level: Level::INFO,
            buf: Vec::with_capacity(256),
        }
    }

    fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        ConsoleWriter {
            level: *meta.level(),
            buf: Vec::with_capacity(256),
        }
    }
}

/// Buffers one event and flushes it to the console on drop.
pub struct ConsoleWriter {
    level: Level,
    buf: Vec<u8>,
}

impl Write for ConsoleWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        // Drain, so the explicit flush `fmt` may issue and the one in `Drop`
        // cannot emit the same event twice. A non-UTF-8 buffer is dropped
        // rather than retried forever.
        if let Some((level, data)) = self.take_event() {
            emit(level, &data);
        }
        Ok(())
    }
}

/// Hand the event to the browser console.
///
/// `web_sys` is a panic-on-call stub off wasm, and `lib/ui` is built for the
/// host too (`--features ssr`, clippy, doctests) where there is no console to
/// write to — so the sink, not the buffering, is what gets gated.
#[cfg(target_arch = "wasm32")]
fn emit(level: Level, data: &str) {
    let value = wasm_bindgen::JsValue::from_str(data);
    match level {
        Level::TRACE | Level::DEBUG => web_sys::console::debug_1(&value),
        Level::INFO => web_sys::console::log_1(&value),
        Level::WARN => web_sys::console::warn_1(&value),
        Level::ERROR => web_sys::console::error_1(&value),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn emit(_level: Level, _data: &str) {}

impl ConsoleWriter {
    /// Take the buffered event, leaving the buffer empty.
    ///
    /// Split out from [`Write::flush`] so the buffering rule — an event is
    /// emitted at most once — is testable on the host, where the `web_sys`
    /// call itself cannot run.
    fn take_event(&mut self) -> Option<(Level, String)> {
        if self.buf.is_empty() {
            return None;
        }
        let buf = std::mem::take(&mut self.buf);
        std::str::from_utf8(&buf)
            .ok()
            .map(|data| (self.level, data.to_owned()))
    }
}

impl Drop for ConsoleWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn writer(level: Level) -> ConsoleWriter {
        ConsoleWriter {
            level,
            buf: Vec::new(),
        }
    }

    #[test]
    fn an_event_is_emitted_at_most_once() {
        let mut w = writer(Level::WARN);
        w.write_all(b"halo").unwrap();
        assert_eq!(w.take_event(), Some((Level::WARN, "halo".to_string())));
        assert_eq!(w.take_event(), None, "flush must not repeat the event");
    }

    #[test]
    fn writes_accumulate_into_one_event() {
        let mut w = writer(Level::INFO);
        w.write_all(b"a").unwrap();
        w.write_all(b"b").unwrap();
        assert_eq!(w.take_event(), Some((Level::INFO, "ab".to_string())));
    }

    #[test]
    fn invalid_utf8_is_dropped_not_retried() {
        let mut w = writer(Level::ERROR);
        w.write_all(&[0xff, 0xfe]).unwrap();
        assert_eq!(w.take_event(), None);
        assert_eq!(w.take_event(), None, "a bad buffer must not linger");
    }

    #[test]
    fn default_writer_reports_info() {
        assert_eq!(ConsoleMakeWriter.make_writer().level, Level::INFO);
    }
}
