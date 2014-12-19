//! The module containing a structure for BAPS3 protocol messages.
#![experimental]

use util::{slicify, unslicify};

/// A structure for BAPS3 protocol messages.
#[deriving(Clone)]
pub struct Message {
    /// The command word of the message.
    _word: String,

    /// The arguments of the message.
    _args: Vec<String>
}

impl Message {
    /// Creates a new Message.
    pub fn new<Sized? W: Str, Sized? A: Str+Sized>(word: &W, args: &[A])
      -> Message {
        Message { _word: word.as_slice().to_string(), _args: unslicify(args) }
    }

    /// Creates a new Message with no arguments.
    pub fn from_word<Sized? W: Str>(word: &W) -> Message {
        Message { _word: word.as_slice().to_string(), _args: vec![] }
    }

    /// Retrieves the command word of this Message.
    ///
    /// # Example
    /// ```rust
    /// use baps3_cli::message::Message;
    /// let m = Message::new("foo", &["bar", "baz"]);
    /// assert_eq!(m.word(), "foo");
    /// ```
    pub fn word<'a>(&'a self) -> &'a str {
        self._word.as_slice()
    }

    /// Retrieves the command arguments of this Message.
    ///
    /// # Example
    /// ```rust
    /// use baps3_cli::message::Message;
    /// let m = Message::new("foo", &["bar", "baz"]);
    /// assert_eq!(m.args(), vec!["bar", "baz"]);
    /// ```
    pub fn args<'a>(&'a self) -> Vec<&'a str> {
        slicify(&self._args)
    }

    /// Constructs a vector of string slices referencing the whole Message.
    ///
    /// This is probably best used for pattern-matching on the entire Message.
    ///
    /// # Example
    /// ```rust
    /// use baps3_cli::message::Message;
    /// let m = Message::new("foo", &["bar", "baz"]);
    /// assert_eq!(m.as_str_vec(), vec!["foo", "bar", "baz"]);
    /// ```
    pub fn as_str_vec<'a>(&'a self) -> Vec<&'a str> {
        let mut v = self.args();
        v.insert(0, self.word());
        v
    }
}
