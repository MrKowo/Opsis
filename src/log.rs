use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

static APP_START_TIME: OnceLock<Instant> = OnceLock::new();

/// Log level hierarchy for console output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Off = 0,
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

impl LogLevel {
    pub fn from_str_loose(s: &str) -> Option<LogLevel> {
        match s.trim().to_lowercase().as_str() {
            "off" | "none" | "0" | "false" => Some(LogLevel::Off),
            "error" | "err" | "1" => Some(LogLevel::Error),
            "warn" | "warning" | "2" => Some(LogLevel::Warn),
            "info" | "default" | "3" => Some(LogLevel::Info),
            "debug" | "verbose" | "4" => Some(LogLevel::Debug),
            "trace" | "all" | "5" => Some(LogLevel::Trace),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Off => "OFF",
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}

static CURRENT_LOG_LEVEL: AtomicU8 = AtomicU8::new(
    if cfg!(debug_assertions) {
        LogLevel::Debug as u8
    } else {
        LogLevel::Info as u8
    },
);

/// Set global logging level at runtime.
pub fn set_log_level(level: LogLevel) {
    CURRENT_LOG_LEVEL.store(level as u8, Ordering::SeqCst);
}

/// Retrieve the active global logging level.
pub fn get_log_level() -> LogLevel {
    match CURRENT_LOG_LEVEL.load(Ordering::Relaxed) {
        0 => LogLevel::Off,
        1 => LogLevel::Error,
        2 => LogLevel::Warn,
        3 => LogLevel::Info,
        4 => LogLevel::Debug,
        _ => LogLevel::Trace,
    }
}

/// Returns true if messages at the given level should be displayed.
pub fn is_level_enabled(level: LogLevel) -> bool {
    get_log_level() >= level
}

/// Returns the elapsed time since application start formatted as seconds (e.g. `1.234s`).
pub fn elapsed_since_start() -> String {
    let start = APP_START_TIME.get_or_init(Instant::now);
    let elapsed = start.elapsed();
    let secs = elapsed.as_secs_f64();
    format!("{:.3}s", secs)
}

/// Parsed command-line arguments and configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliArgs {
    pub image_path: Option<PathBuf>,
    pub log_level: Option<LogLevel>,
    pub print_help: bool,
    pub print_version: bool,
}

/// Parse command-line arguments from an iterator of string arguments.
pub fn parse_cli_args<I, T>(args: I) -> CliArgs
where
    I: IntoIterator<Item = T>,
    T: Into<String>,
{
    let mut image_path = None;
    let mut log_level = None;
    let mut print_help = false;
    let mut print_version = false;

    let mut iter = args.into_iter().map(|s| s.into()).peekable();
    // Skip program name if present
    let _ = iter.next();

    while let Some(arg) = iter.next() {
        if arg == "-h" || arg == "--help" {
            print_help = true;
        } else if arg == "-V" || arg == "--version" {
            print_version = true;
        } else if arg == "-v" || arg == "--verbose" {
            log_level = Some(LogLevel::Debug);
        } else if arg == "-vv" || arg == "--trace" {
            log_level = Some(LogLevel::Trace);
        } else if arg == "-q" || arg == "--quiet" {
            log_level = Some(LogLevel::Error);
        } else if arg == "--silent" {
            log_level = Some(LogLevel::Off);
        } else if arg == "--log-level" {
            if let Some(val) = iter.next() {
                if let Some(parsed) = LogLevel::from_str_loose(&val) {
                    log_level = Some(parsed);
                }
            }
        } else if let Some(stripped) = arg.strip_prefix("--log-level=") {
            if let Some(parsed) = LogLevel::from_str_loose(stripped) {
                log_level = Some(parsed);
            }
        } else if !arg.starts_with('-') && image_path.is_none() {
            let path = PathBuf::from(&arg);
            image_path = Some(path);
        }
    }

    CliArgs {
        image_path,
        log_level,
        print_help,
        print_version,
    }
}

/// Initialize logging configuration from CLI flags and environment variables.
pub fn init_logging_from_args_and_env(cli_level: Option<LogLevel>) {
    if let Some(level) = cli_level {
        set_log_level(level);
        return;
    }

    if let Ok(env_val) = std::env::var("OPSIS_LOG").or_else(|_| std::env::var("RUST_LOG")) {
        if let Some(level) = LogLevel::from_str_loose(&env_val) {
            set_log_level(level);
        }
    }
}

/// Print CLI help screen to stdout.
pub fn print_help_screen() {
    println!(
        r#"Opsis v{} - Minimalist, hyper-fast image viewer

USAGE:
    opsis [OPTIONS] [IMAGE_PATH]

ARGS:
    <IMAGE_PATH>           Path to image file to view upon launch

OPTIONS:
    --log-level <LEVEL>    Set console logging level (off, error, warn, info, debug, trace)
    -v, --verbose          Enable verbose debug logging (--log-level=debug)
    -vv, --trace           Enable full trace logging (--log-level=trace)
    -q, --quiet            Only log errors (--log-level=error)
    --silent               Disable all console logging (--log-level=off)
    -h, --help             Print help information
    -V, --version          Print version information

ENVIRONMENT:
    OPSIS_LOG, RUST_LOG    Set fallback log level (e.g. OPSIS_LOG=debug)
"#,
        env!("CARGO_PKG_VERSION")
    );
}

/// Formatted dev logger for console output filtered by the active LogLevel.
#[macro_export]
macro_rules! dev_log_level {
    ($level:expr, $tag:expr, $($arg:tt)*) => {
        if $crate::log::is_level_enabled($level) {
            println!(
                "[{}] [{}] [{}] {}",
                $crate::log::elapsed_since_start(),
                $level.as_str(),
                $tag,
                format_args!($($arg)*)
            );
        }
    };
}

/// Convenience macro for logging user input events (keys, pointer, gestures).
#[macro_export]
macro_rules! log_input {
    ($($arg:tt)*) => {
        $crate::dev_log_level!($crate::log::LogLevel::Debug, "Opsis Input", $($arg)*)
    };
}

/// Convenience macro for logging file reading, decoding, and folder scanning events.
#[macro_export]
macro_rules! log_io {
    ($($arg:tt)*) => {
        $crate::dev_log_level!($crate::log::LogLevel::Info, "Opsis File I/O", $($arg)*)
    };
}

/// Convenience macro for logging extension discovery, loading, and lifecycle events.
#[macro_export]
macro_rules! log_ext {
    ($($arg:tt)*) => {
        $crate::dev_log_level!($crate::log::LogLevel::Info, "Opsis Extensions", $($arg)*)
    };
}

/// Convenience macro for logging hotkey evaluation, rebinding, and dispatching.
#[macro_export]
macro_rules! log_hotkey {
    ($($arg:tt)*) => {
        $crate::dev_log_level!($crate::log::LogLevel::Info, "Opsis Hotkeys", $($arg)*)
    };
}

/// Convenience macro for logging canvas zooming, panning, and viewport rendering.
#[macro_export]
macro_rules! log_canvas {
    ($($arg:tt)*) => {
        $crate::dev_log_level!($crate::log::LogLevel::Debug, "Opsis Canvas", $($arg)*)
    };
}

/// Convenience macro for logging window lifecycle and geometry events.
#[macro_export]
macro_rules! log_window {
    ($($arg:tt)*) => {
        $crate::dev_log_level!($crate::log::LogLevel::Debug, "Opsis Window", $($arg)*)
    };
}

/// Convenience macro for logging error events.
#[macro_export]
macro_rules! log_error {
    ($tag:expr, $($arg:tt)*) => {
        $crate::dev_log_level!($crate::log::LogLevel::Error, $tag, $($arg)*)
    };
}

/// Convenience macro for logging warning events.
#[macro_export]
macro_rules! log_warn {
    ($tag:expr, $($arg:tt)*) => {
        $crate::dev_log_level!($crate::log::LogLevel::Warn, $tag, $($arg)*)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering_and_parsing() {
        assert!(LogLevel::Off < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Trace);

        assert_eq!(LogLevel::from_str_loose("debug"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_str_loose("TRACE"), Some(LogLevel::Trace));
        assert_eq!(LogLevel::from_str_loose("0"), Some(LogLevel::Off));
        assert_eq!(LogLevel::from_str_loose("invalid"), None);
    }

    #[test]
    fn test_cli_argument_parsing() {
        let args = vec!["opsis", "--verbose", "assets/logo.png"];
        let parsed = parse_cli_args(args);
        assert_eq!(parsed.log_level, Some(LogLevel::Debug));
        assert_eq!(parsed.image_path, Some(PathBuf::from("assets/logo.png")));
        assert!(!parsed.print_help);

        let args_level = vec!["opsis", "--log-level=trace", "-h"];
        let parsed_level = parse_cli_args(args_level);
        assert_eq!(parsed_level.log_level, Some(LogLevel::Trace));
        assert!(parsed_level.print_help);

        let args_quiet = vec!["opsis", "-q"];
        let parsed_quiet = parse_cli_args(args_quiet);
        assert_eq!(parsed_quiet.log_level, Some(LogLevel::Error));

        let args_silent = vec!["opsis", "--silent"];
        let parsed_silent = parse_cli_args(args_silent);
        assert_eq!(parsed_silent.log_level, Some(LogLevel::Off));
    }

    #[test]
    fn test_set_and_get_log_level() {
        set_log_level(LogLevel::Warn);
        assert_eq!(get_log_level(), LogLevel::Warn);
        assert!(is_level_enabled(LogLevel::Error));
        assert!(is_level_enabled(LogLevel::Warn));
        assert!(!is_level_enabled(LogLevel::Debug));

        set_log_level(LogLevel::Debug);
        assert!(is_level_enabled(LogLevel::Info));
        assert!(is_level_enabled(LogLevel::Debug));
    }
}
