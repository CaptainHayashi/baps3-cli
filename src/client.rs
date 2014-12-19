//! The module containing an implementation of a BAPS3 client.
#![experimental]

use std::io::{
    IoError,
    IoErrorKind,
    IoResult,
    TcpStream
};
use std::io::net::ip::ToSocketAddr;

use baps3_protocol::{pack, Unpacker};
use message::Message;

/// A BAPS3 protocol client.
pub struct Client {
    /// The channel used to send requests to the client.
    pub request_tx: Sender<Request>,

    /// The channel on which the client listens for responses.
    pub response_rx: Receiver<Response>,
}

/// A request to the BAPS3 client.
pub enum Request {
    /// Send a message to a client.
    SendMessage(Message),

    /// Close the client connection.
    Quit
}

/// A response from the BAPS3 client.
pub enum Response {
    /// The client has sent a message.
    Message(Message),

    /// The client connection has closed.
    Gone,

    /// The client connection encountered an error.
    ClientError(IoError)
}

impl Client {
    pub fn new<T: ToSocketAddr>(addr: T) -> IoResult<Client> {
        TcpStream::connect(addr).and_then(Client::from_stream)
    }

    pub fn from_stream(stream: TcpStream) -> IoResult<Client> {
        let (request_tx, request_rx) = channel();
        let (response_tx, response_rx) = channel();

        let w_stream = stream.clone();
        let w_resp_tx = response_tx.clone();
        spawn(move || { write_task(w_stream, w_resp_tx, request_rx); });

        let r_stream = stream.clone();
        spawn(move || { read_task(r_stream, response_tx); });

        Ok(Client {
            request_tx: request_tx,
            response_rx: response_rx
        })
    }
}

/// The body of the task responsible for reading responses from the client.
fn read_task(stream: TcpStream, tx: Sender<Response>) {
    let mut u = Unpacker::new();
    let mut strm = stream;

    'l: loop {
        // We ignore any send errors, because the response channel is liable to
        // be closed by the client when it gets sick of hearing us.

        match strm.read_byte() {
            Ok(b) => for pline in u.feed_bytes(&mut(Some(b).into_iter())).iter() {
                if let [ref word, args..] = pline.as_slice() {
                    if let Err(_) = tx.send_opt(
                        Response::Message(Message::new(word.as_slice(), args))
                    ) {
                        break 'l;
                    }
                }
            },
            Err(ref e) if e.kind == IoErrorKind::EndOfFile => {
                let _ = tx.send_opt(Response::Gone);
                break 'l;
            },
            Err(e) => {
                let _ = tx.send_opt(Response::ClientError(e));
                break 'l;
            },
        }
    }
}

/// The body of the task responsible for writing requests to the client.
fn write_task(stream: TcpStream,
              tx: Sender<Response>,
              rx: Receiver<Request>) {
    let mut strm = stream;

    'l: for r in rx.iter() {
        match r {
            Request::SendMessage(msg) => {
                let sargs = msg.args();
                let packed = pack(msg.word(), sargs.as_slice());

                if let Err(e) = strm.write_line(packed.as_slice()) {
                    tx.send(Response::ClientError(e));
                    break 'l;
                }
            },
            Request::Quit => break 'l
        }
    }

    if let Err(ce) = strm.close_read() {
        println!("Error closing stream: {}", ce);
    }
    if let Err(ce) = strm.close_write() {
        println!("Error closing stream: {}", ce);
    }
}