use std::{env, time::Duration};

use copypasta::{ClipboardContext, ClipboardProvider};
use ewebsock::{WsReceiver, WsSender};

fn main() {

    let mut url = env::args().nth(1).expect("failed to pass url param");
    
    url.insert_str(0, "ws://");

    url.push_str(":5321");

    let mut ctx = ClipboardContext::new().unwrap();

    let mut clipboard = ctx.get_contents().expect("failed to read initial clipboard");

    let (mut send, receive) = connect_to_server(&url);
    
    loop {
        //send.send(ewebsock::WsMessage::Text("hi".to_string()));

        match ctx.get_contents() {
            Ok(contents) => {
                // if clipboard updated
                if clipboard != contents {

                    
                    clipboard = contents;

                    println!("{}", clipboard);

                    send.send(ewebsock::WsMessage::Text(clipboard.clone()))
                }
            },
            Err(error) => {println!("{}", error.to_string())},
        }

        let clipboard_update = match receive.try_recv() {
            Some(event) => {
                match event {
                    ewebsock::WsEvent::Message(message) => {
                        match message {
                            ewebsock::WsMessage::Text(text) => {
                                Some(text)
                            },
                            _ => panic!("server error")
                        }
                    },
                    _ => panic!("server error")
                }
            },
            None => None,
        };

        if let Some(clipboard_update) = clipboard_update {

            println!("new clipboard: {}", clipboard_update);
            clipboard = clipboard_update;
            ctx.set_contents(clipboard.clone()).expect("failed to update clipboard");
        }
       

        std::thread::sleep(Duration::from_millis(500));

    }
}

fn connect_to_server(url: &str) -> (WsSender, WsReceiver) {
    let (server_send, server_receive) = match ewebsock::connect(url, ewebsock::Options::default()) {
        Ok(result) => result,
        Err(error) => {
            panic!("failed to connect to server: {}", error)
        },
    }; 

    // wait for Opened event from server
    loop {
        match server_receive.try_recv() {
            Some(event) => {
                match event {
                    ewebsock::WsEvent::Opened => {
                        println!("we got the opened message!");
                        break (server_send, server_receive);
                    },
                    ewebsock::WsEvent::Message(message) => {
                        match message {
                            _ => panic!("received a message from the server")
                        }
                    },
                    ewebsock::WsEvent::Error(error) => panic!("received error when trying to connect to server: {}", error),
                    ewebsock::WsEvent::Closed => panic!("server closed when trying to connect"),
                    
                }
            },
            None => continue,
        }
    }
}