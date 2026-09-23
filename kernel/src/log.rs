use core::fmt::Write;
use crate::uart::Uart;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]

pub enum Level {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

impl Level {
    fn as_str(self) -> &'static str {
        match self {
            Level::Error => "ERROR",
            Level::Warn =>  "WARN",
            Level::Info =>  "INFO",
            Level::Debug => "DEBUG",
            Level::Trace => "TRACE",
        }
    }
}

static mut MAX_LEVEL: Level = Level::Info;

pub unsafe fn set_max_level(level: Level) {
    MAX_LEVEL = level;
}

fn max_level() -> Level {
    unsafe {core::ptr::read(core::ptr::addr_of!(MAX_LEVEL))}
}

#[doc(hidden)]
pub fn __log(level: Level, subsystem: &str, args: core::fmt::Arguments) {
    if level > max_level() {
        return;
    }
    let mut uart = Uart::new();
    let _ = writeln!(uart, "[{}][{:>6}] {}", level.as_str(), subsystem, args);
}

#[macro_export]
macro_rules! log_error {
    ($sub:expr, $($arg:tt)*) => {
        $crate::log::__log($crate::log::Level::Error, $sub, format_args!($($arg)*))
    };
}
#[macro_export]
macro_rules! log_warn {
    ($sub:expr, $($arg:tt)*) => {
        $crate::log::__log($crate::log::Level::Warn, $sub, format_args!($($arg)*))
    };
}
#[macro_export]
macro_rules! log_info {
    ($sub:expr, $($arg:tt)*) => {
        $crate::log::__log($crate::log::Level::Info, $sub, format_args!($($arg)*))
    };
}
#[macro_export]
macro_rules! log_debug {
    ($sub:expr, $($arg:tt)*) => {
        $crate::log::__log($crate::log::Level::Debug, $sub, format_args!($($arg)*))
    };
}
#[macro_export]
macro_rules! log_trace {
    ($sub:expr, $($arg:tt)*) => {
        $crate::log::__log($crate::log::Level::Trace, $sub, format_args!($($arg)*))
    };
}