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
///
/// # Examples
/// ```rust
/// use baps3_cli::util::slicify;
///
/// let v = vec!["a".to_string(), "b".to_string(), "c".to_string()];
/// assert_eq!(slicify(&v), vec!["a", "b", "c"]);
/// ```
pub fn slicify<'a, Sized? S: Sized+Str>(vec: &'a Vec<S>) -> Vec<&'a str> {
    map_collect(vec.iter(), |a| a.as_slice())
}

/// Creates a vector of strings from a slice of string-slices.
///
/// # Examples
/// ```rust
/// use baps3_cli::util::unslicify;
///
/// let v = ["a", "b", "c"];
/// assert_eq!(unslicify(&v),
///            vec!["a".to_string(), "b".to_string(), "c".to_string()]);
/// ```
pub fn unslicify<'a, Sized? S: Sized+Str>(slices: &'a[S]) -> Vec<String> {
    map_collect(slices.iter(), |a: &S| a.as_slice().to_string())
}

/// Performs a map and collects the results.
pub fn map_collect<A, B, I, F, C>(iter: I, f: C) -> F
  where I: Iterator<A>,
        F: FromIterator<B>,
        C: FnMut(A) -> B {
    iter.map(f).collect::<F>()
}