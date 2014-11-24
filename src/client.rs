//! The module containing an implementation of a BAPS3 client.
#![experimental]

use std::io::{
    BufferedReader,
    IoError,
    IoErrorKind,
    IoResult,
    TcpStream
};
use std::io::net::ip::ToSocketAddr;

use baps3_protocol::{pack, Unpacker};

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
    SendMessage(String, Vec<String>),

    /// Close the client connection.
    Quit
}

/// A response from the BAPS3 client.
pub enum Response {
    /// The client has sent a message.
    Message(String, Vec<String>),

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

        let mut w_stream = stream.clone();
        let w_resp_tx = response_tx.clone();
        spawn(proc() { write_task(&mut w_stream, w_resp_tx, request_rx); });

        let r_stream = stream.clone();
        spawn(proc() { read_task(r_stream, response_tx); });

        Ok(Client {
            request_tx: request_tx,
            response_rx: response_rx
        })
    }
}

/// The body of the task responsible for reading responses from the client.
fn read_task(stream: TcpStream, tx: Sender<Response>) {
    let mut u = Unpacker::new();
    let mut reader = BufferedReader::new(stream);

    for line in reader.lines() {
        match line {
            Ok(s) => for pline in u.feed(s.as_slice()).iter() {
                if let [ref cmd, args..] = pline.as_slice() {
                    tx.send(Response::Message(cmd.clone(), args.to_vec()));
                }
            },
            Err(ref e) if e.kind == IoErrorKind::EndOfFile => {
                tx.send(Response::Gone);
                return;
            },
            Err(e) => {
                tx.send(Response::ClientError(e));
                return;
            },
        }
    }
}

/// The body of the task responsible for writing requests to the client.
fn write_task(stream: &mut TcpStream,
              tx: Sender<Response>,
              rx: Receiver<Request>) {
    for r in rx.iter() {
        match r {
            Request::SendMessage(cmd, args) => {
                let sargs = args.iter()
                                .map(|f| f.as_slice())
                                .collect::<Vec<&str>>();
                let packed = pack(cmd.as_slice(), sargs.as_slice());

                if let Err(e) = stream.write_line(packed.as_slice()) {
                    tx.send(Response::ClientError(e));

                    if let Err(ce) = stream.close_read() {
                        println!("Error closing stream: {}", ce);
                    }
                    return;
                }
            },
            Request::Quit => {
                if let Err(ce) = stream.close_read() {
                    println!("Error closing stream: {}", ce);
                }
                return;
            }
        }
    }
}