pub use std::io::{self, Write};

macro_rules! warn {
    ($fmt:expr $(, $arg:expr )* $(,)*) => {
        let _ = write!(io::stderr(), "WARN: {}", format!($fmt, $( $arg, )*));
    };
}
