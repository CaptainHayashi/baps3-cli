#![macro_escape]

#[macro_export]
macro_rules! werr(
    ($($arg:tt)*) => (
        if let Err(err) = std::io::stderr().write_str(&*format!($($arg)*)) {
            panic!("{}", err);
        }
    )
)

/// Creates a vector of string-slices from a vector of strings.
///
/// The slice vector lives as long as the original vector.
pub fn slicify<'a>(vec: &'a Vec<String>) -> Vec<&'a str> {
    map_collect(vec.iter(), |a| a.as_slice())
}

/// Creates a vector of strings from a slice of string-slices.
pub fn unslicify(slices: &[&str]) -> Vec<String> {
    map_collect(slices.iter(), |a| a.to_string())
}

/// Performs a map and collects the results.
pub fn map_collect<A, B, I: Iterator<A>, F: FromIterator<B>>(
  iter: I, f: |A| -> B
) -> F {
    iter.map(f).collect::<F>()
}