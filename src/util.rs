#[macro_export]
macro_rules! werr(
    ($($arg:tt)*) => (
        if let Err(err) = std::io::stderr().write_str(&*format!($($arg)*)) {
            panic!("{}", err);
        }
    )
)