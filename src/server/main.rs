use std::{net::{TcpListener, TcpStream}, time::Duration};

use tungstenite::{Message, WebSocket};

fn main() {

    let listener = match TcpListener::bind("0.0.0.0:5321") {
        Ok(listener) => listener,
        Err(error) => panic!("failed to bind listener: {}", error),
    };

    match listener.set_nonblocking(true) {
        Ok(_) => {},
        Err(error) => panic!("failed to set server as non blocking: {}", error),
    };

    let mut clients: Vec<WebSocket<TcpStream>> = vec![];

    loop {
        accept_new_clients(&listener, &mut clients);

        relay_updates(&mut clients);

        std::thread::sleep(Duration::from_secs(1));
    }
    

}

fn relay_updates(clients: &mut Vec<WebSocket<TcpStream>>) {
    'client_loop: for client_index in 0..clients.len() {

        let mut client = clients.remove(client_index);

        let update = match client.read() {
            Ok(message) => {
                match message {
                    Message::Text(text) => {
                        println!("{}", text);
                        text
                    },
                    _ => panic!("unmateched message type")
                    
                }
            },
            Err(error) => {
                match error {

                    tungstenite::Error::Io(io_error) => {
                        match io_error.kind() {
                            std::io::ErrorKind::WouldBlock => {
                                // this means that there was no update to read
                                clients.insert(client_index, client);
                                
                                continue 'client_loop // move to the next client
                            },
                            _ => todo!("unhandled io error: {}", io_error),
                        }
                    },
                    _ => todo!("unhandled websocket message read error: {}", error)
                }
            },
        };

        for other_client in &mut *clients {
            other_client.send(Message::Text(update.clone())).expect("failed to relay message to one of the clients");
        }

        clients.insert(client_index, client);
    }
}

fn accept_new_clients(listener: &TcpListener, clients: &mut Vec<WebSocket<TcpStream>>) {
    let mut client: Option<WebSocket<TcpStream>> = match listener.accept() {
        Ok((stream, address)) => {

            println!("received new connection from address: {}", address);

            stream.set_nonblocking(true).expect("Failed to set new client as non blocking");

            // loop while trying to establish websocket connection
            let websocket_stream = loop {

                match tungstenite::accept(stream.try_clone().expect("failed to clone stream")) {
                    Ok(websocket_stream) => break websocket_stream, // success

                    Err(error) => {
                        match error {
                            tungstenite::HandshakeError::Interrupted(_) => continue, // try again if the handshake isnt done yet
                            tungstenite::HandshakeError::Failure(error) => panic!("handshake failed with new client: {}", error),
                        }
                    },

                };
            };

            Some(websocket_stream)
        },
        Err(error) => {
            match error.kind() {
                std::io::ErrorKind::WouldBlock => None, // no new clients

                _ => {
                    println!("Something went wrong trying to accept a new client");
                    None
                }
            }
        },
    };

    if let Some(client) = client {
        clients.push(client)
    }
    
}