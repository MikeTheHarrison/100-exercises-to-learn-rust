// This is our last exercise. Let's go down a more unstructured path!
// Try writing an **asynchronous REST API** to expose the functionality
// of the ticket management system we built throughout the course.
// It should expose endpoints to:
//  - Create a ticket
//  - Retrieve ticket details
//  - Patch a ticket
//
// Use Rust's package registry, crates.io, to find the dependencies you need
// (if any) to build this system.

use serde_json::{self};
use std::sync::Arc;
use tokio::{io::AsyncReadExt, net::TcpListener};

use crate::ticket_store::{TicketError, TicketMessage, TicketStore};

pub mod ticket_store;

pub async fn launch_server(addr: std::net::IpAddr, port: u16) {
    let ticket_store = Arc::new(TicketStore::default());
    let listener = TcpListener::bind((addr, port)).await.unwrap();
    loop {
        let (mut stream, _addr) = listener.accept().await.unwrap();
        let store_clone = Arc::clone(&ticket_store);
        tokio::spawn(async move {
            let mut buffer = Vec::new();
            let n = stream.read_to_end(&mut buffer).await.unwrap();

            if n == 0 {
                // EOM or error.
                return;
            }

            let message = json_to_ticket_message(&buffer);
            match message {
                Ok(ticket_msg) => {
                    let _response = store_clone.handle_message(&ticket_msg).await;
                    //stream.write(response);
                }
                Err(_e) => todo!(), // stream.write(Err(e)),
            }
        });
    }
}

pub fn json_to_ticket_message(json: &Vec<u8>) -> Result<TicketMessage, TicketError> {
    match serde_json::from_slice::<TicketMessage>(&json) {
        Ok(message) => Ok(message),
        Err(error) => Err(TicketError(error.to_string())),
    }
}

pub fn ticket_to_json(ticket: &TicketMessage) -> String {
    serde_json::to_string_pretty(ticket).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::ticket_store::Ticket;

    use super::*;

    #[tokio::test]
    async fn ticket_tester() {}

    #[test]
    fn test_ticket_to_json() {
        let ticket = Ticket::new(1024, String::from("title"), String::from("description"));
        let mesg = TicketMessage::Create(ticket);
        let json_ticket = ticket_to_json(&mesg);
        let dup_ticket = json_to_ticket_message(&json_ticket.as_bytes().to_vec()).unwrap();

        println!("to json_ticket -> {json_ticket}");
        println!("from json ticket -> {:?}", &dup_ticket);

        assert_eq!(mesg, dup_ticket);
    }
}
