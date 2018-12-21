extern crate chrono;
extern crate env_logger;
#[macro_use]
extern crate log;
#[cfg_attr(test, macro_use)]
extern crate serde_json;

use env_logger::{Builder, Env};
use std::io::Write;

/// A convenience wrapper around the log! macro for writing log messages that ElasticSearch can
/// ingest.
/// You can use the default logging form:
/// `jlog!(log::level::Info, "Log message")`
/// which produces
/// `{"level": "INFO", "target": "my_module", "message":"Log message"}`
/// Or you can provide metadata for ES to use:
/// ```text
///   let val = -1;
///   jlog!(Error, "Amount must be positive", {"value": val})
/// ```
/// which will produce:
/// `{"level": "ERROR", "target": "my_module", "message": "Amount must be positive", "value": -1}`
#[macro_export]
macro_rules! jlog {
    ($t:path, $msg:expr) => {{
        use $crate::transform_message;
        transform_message($t, None, $msg, "")
    }};
    ($t:path, $msg:expr, $json:tt) => {{
        use $crate::transform_message;
        let meta = json!($json).to_string();
        transform_message($t, None, $msg, &meta)
    }};
    ($t:path, $target: expr, $msg:expr, $json:tt) => {{
        use $crate::transform_message;
        let meta = json!($json).to_string();
        transform_message($t, Some($target), $msg, &meta)
    }};
}

pub fn transform_message(level: log::Level, target: Option<&str>, msg: &str, meta: &str) {
    let inner = format_message(msg, meta);
    match target {
        Some(t) => log!(target: t, level, "{}", inner),
        None => log!(level, "{}", inner),
    }
}

fn format_message(msg: &str, meta: &str) -> String {
    match meta {
        "" => format!("\"message\": \"{}\"", msg.trim()),
        "{}" => format!("\"message\": \"{}\"", msg.trim()),
        _ => format!(
            "\"message\": \"{}\", {}",
            msg.trim(),
            &meta[1..meta.len() - 1]
        ),
    }
}

fn is_in_message_format(msg: &str) -> bool {
    msg.starts_with("\"message\":")
}

pub fn setup_logger() {
    Builder::from_env(Env::default().default_filter_or("debug"))
        .format(|buf, record| {
            let mut msg = format!("{}", record.args());
            if !is_in_message_format(&msg) {
                msg = format_message(&msg, "");
            }

            writeln!(
                buf,
                "{{ \"time\": \"{}\", \"level\": \"{}\", \"target\": \"{}\", {} }}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.level(),
                record.target(),
                msg,
            )
        })
        .init();
}

#[cfg(test)]
mod tests {
    use log::Level::*;

    #[test]
    fn test_jlog() {
        // super::setup_logger().unwrap();
        // Level, Message
        jlog!(Warn, "message");
        // Level, message, meta
        jlog!(Warn, "test", {"a": 1} );
        // Level, message, meta
        jlog!(Error, "test", {"a": 1, "b": "jake", "c": [3, 2, 1]});
        // Level, target, message, meta
        jlog!(
            Debug,
            "bigneon::domain_actions",
            "Found no actions to process",
            {}
        );
    }
}
