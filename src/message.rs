//! The module containing a structure for BAPS3 protocol messages.
#![experimental]

use util::{slicify, unslicify};
use baps3_protocol;

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
    ///
    /// # Example
    ///
    /// A `from_word` message has a word:
    ///
    /// ```rust
    /// use baps3_cli::message::Message;
    /// let m = Message::from_word("bird");
    /// assert_eq!(m.word(), "bird");
    /// ```
    ///
    /// ...but no arguments:
    ///
    /// ```rust
    /// use baps3_cli::message::Message;
    /// let m = Message::from_word("bird");
    /// assert!(m.args().is_empty());
    /// ```
    pub fn from_word<Sized? W: Str>(word: &W) -> Message {
        Message { _word: word.as_slice().to_string(), _args: vec![] }
    }

    /// Packs a Message into a BAPS3 protocol line.
    ///
    /// A `pack`ed message is ready for sending down the wire to a BAPS3
    /// client or server via `write_line`.
    ///
    /// # Example
    ///
    /// If none of the arguments have special characters, this is trivial:
    ///
    /// ```rust
    /// use baps3_cli::message::Message;
    /// let m = Message::new("foo", &["bar", "baz"]);
    /// assert_eq!(m.pack().as_slice(), "foo bar baz");
    /// ```
    ///
    /// With special characters, we get escaping:
    ///
    /// ```rust
    /// use baps3_cli::message::Message;
    /// let m = Message::new("foo", &["with space", "'single'", "\"double\""]);
    /// assert_eq!(m.pack().as_slice(),
    ///            "foo 'with space' ''\\''single'\\''' '\"double\"'");
    /// ```
    pub fn pack(&self) -> String {
        let vargs = self.args();
        baps3_protocol::pack(self.word(), vargs.as_slice())
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
