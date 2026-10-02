use std::io::{Read, Write, ErrorKind};
use std::net::{TcpListener, TcpStream};
use chess;

pub const PORT: u16 = 6767;

// Some network read/write code taken fromthepacketgeek.com

// Used for handling the game over network from a given stream 
pub struct Network {
    stream: TcpStream,
    self_color: chess::Color,
    is_host: bool
}

#[derive(Debug)]
pub enum NetworkError {
    ConnectionFailure,
    NonBlockingFailure,
    NoClient,
    HandshakeFailure,
    Disconnection
}

impl Network {
    // Return local color
    pub fn self_color(&self) -> chess::Color {
        return self.self_color;
    }

    // Return if host or not
    pub fn is_host(&self) -> bool {
        return self.is_host;
    }
    
    // Creates a network connection as a client
    pub fn establish_as_client(addr: &str, self_color: chess::Color) -> Result<Self, NetworkError> {
        // Tries to connect to addr
        let mut stream = match TcpStream::connect(addr) {
            Ok(stream) => stream,
            _ => return Err(NetworkError::ConnectionFailure)
        };
        // Send color to host
        match send_color(&mut stream, self_color) {
            Ok(()) => (),
            Err(error) => return Err(error)
        };
        // Sets nonblocking
        match stream.set_nonblocking(true) {
            Ok(()) => (),
            _ => return Err(NetworkError::NonBlockingFailure)
        };
        // Creates new network structure
        return Ok(Self { stream, self_color , is_host: false});
    }

    // Creates a network connection as a host
    pub fn establish_as_host() -> Result<Self, NetworkError> {
        // Sets up host
        let listener = match TcpListener::bind(("0.0.0.0", PORT)) {
            Ok(listener) => listener,
            _ => return Err(NetworkError::ConnectionFailure)
        };

        // Look for client
        let (mut stream, _addr) = match listener.accept() {
            Ok(connection) => connection,
            _ => return Err(NetworkError::NoClient)
        };

        // Checks client color
        let client_color = match receive_color(&mut stream) {
            Ok(color) => color,
            Err(error) => return Err(error)
        };

        // Sets nonblocking
        match stream.set_nonblocking(true) {
            Ok(()) => (),
            _ => return Err(NetworkError::NonBlockingFailure)
        };

        // Host is the opposite color of the client
        let self_color = match client_color {
            chess::Color::White => chess::Color::Black,
            _ => chess::Color::White
        };
        // Creates new network structure
        return Ok(Self { stream, self_color , is_host: true});
    }
    // FIX
    pub fn send_message(&mut self, message: &str) -> Result<(), NetworkError> {

        let mut final_message = String::new();
        final_message.push_str(message);
        final_message.push_str("\n");

        // Writes message to the stream
        match self.stream.write_all(final_message.as_bytes()) {
            Ok(()) => return Ok(()),
            _ => return Err(NetworkError::Disconnection)
        };
    }

    pub fn receive_message(&mut self) -> Result<Option<String>, NetworkError> {

        // Store all the bytes for our received String
        let mut received: Vec<u8> = vec![];

        // Read one byte at a time until we hit a \n newline
        let mut rx_bytes = [0u8; 1];
        loop {
            // Read from the current data in the TcpStream
            let bytes_read = match self.stream.read(&mut rx_bytes) {
                Ok(bytes_read) => bytes_read,
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    if received.is_empty() {
                        // Nothing to read, go back to main
                        return Ok(None);
                    }
                    // Keep looping if we're not done receiving data
                    continue;
                }
                _ => return Err(NetworkError::Disconnection)
            };

            // If we didn't fill the array
            // stop reading because there's no more data (we hope!)
            if bytes_read == 0 {
                return Err(NetworkError::Disconnection);
            }

            // However many bytes we read, extend the `received` string bytes
            received.extend_from_slice(&rx_bytes[..bytes_read]);

            // Stop when the whole message (ending in \n) has arrived
            if received.contains(&b'\n') {
                break;
            }
        }

        // Convert to string
        let mut message = match String::from_utf8(received) {
            Ok(message) => message,
            _ => return Err(NetworkError::Disconnection)
        };

        // Return the string without \n
        message.pop();
        return Ok(Some(message));
    }
}

fn send_color(stream: &mut TcpStream, color: chess::Color) -> Result<(), NetworkError> {

    // Client decides starting color
    let message = match color {
        chess::Color::White => "W\n",
        _ => "B\n"
    };

    // Writes message to the stream
    match stream.write_all(message.as_bytes()) {
        Ok(()) => return Ok(()),
        _ => return Err(NetworkError::HandshakeFailure)
    };
}

fn receive_color(stream: &mut TcpStream) -> Result<chess::Color, NetworkError> {

    // Store all the bytes for our received String
    let mut received: Vec<u8> = vec![];

    // Array with a fixed size
    let mut rx_bytes = [0u8; 2];
    loop {
        // Read from the current data in the TcpStream
        let bytes_read = match stream.read(&mut rx_bytes) {
            Ok(bytes_read) => bytes_read,
            _ => return Err(NetworkError::HandshakeFailure)
        };

        // If we didn't fill the array
        // stop reading because there's no more data (we hope!)
        if bytes_read == 0 {
            return Err(NetworkError::HandshakeFailure);
        }

        // However many bytes we read, extend the `received` string bytes
        received.extend_from_slice(&rx_bytes[..bytes_read]);

        // Stop when the whole message (ending in \n) has arrived
        if received.contains(&b'\n') {
            break;
        }
    }

    // Convert to string
    let message = match String::from_utf8(received) {
        Ok(message) => message,
        _ => return Err(NetworkError::HandshakeFailure)
    };

    // Check the string
    match message.as_str() {
        "W\n" => return Ok(chess::Color::White),
        "B\n" => return Ok(chess::Color::Black),
        _ => return Err(NetworkError::HandshakeFailure)
    };
}