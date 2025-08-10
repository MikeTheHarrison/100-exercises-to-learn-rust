// TODO: Convert the implementation to use bounded channels.
use crate::data::{Ticket, TicketDraft};
use crate::store::{TicketId, TicketStore};
use std::sync::mpsc::{Receiver, SyncSender};
use thiserror::Error;

pub mod data;
pub mod store;

#[derive(Clone)]
pub struct TicketStoreClient {
    sender: SyncSender<Command>,
}

#[derive(Error, Debug, Clone)]
pub enum TicketStoreError {
    #[error("Failed to insert new ticket {0}")]
    InsertError(String),
    #[error("Failed to get ticket: {0}")]
    GetError(String),
}

impl TicketStoreClient {
    pub fn insert(&self, draft: TicketDraft) -> Result<TicketId, TicketStoreError> {
        let (tx, rx) = std::sync::mpsc::sync_channel(10);
        self.sender
            .send(Command::Insert {
                draft,
                response_channel: tx,
            })
            .map_err(|e| TicketStoreError::InsertError(format!("{e}")))?;

        match rx.recv() {
            Ok(id) => Ok(id),
            Err(e) => Err(TicketStoreError::InsertError(e.to_string())),
        }
    }

    pub fn get(&self, id: TicketId) -> Result<Option<Ticket>, TicketStoreError> {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        self.sender
            .send(Command::Get {
                id,
                response_channel: tx,
            })
            .map_err(|e| TicketStoreError::GetError(format!("{e}")))?;

        match rx.recv() {
            Ok(ticket) => Ok(ticket),
            Err(e) => Err(TicketStoreError::InsertError(e.to_string())),
        }
    }
}

pub fn launch(capacity: usize) -> TicketStoreClient {
    let (sender, receiver) = std::sync::mpsc::sync_channel(capacity);
    std::thread::spawn(move || server(receiver));

    TicketStoreClient { sender }
}

enum Command {
    Insert {
        draft: TicketDraft,
        response_channel: SyncSender<TicketId>,
    },
    Get {
        id: TicketId,
        response_channel: SyncSender<Option<Ticket>>,
    },
}

fn server(receiver: Receiver<Command>) {
    let mut store = TicketStore::new();
    loop {
        match receiver.recv() {
            Ok(Command::Insert {
                draft,
                response_channel,
            }) => {
                let id = store.add_ticket(draft);
                let _ = response_channel.send(id);
            }
            Ok(Command::Get {
                id,
                response_channel,
            }) => {
                let ticket = store.get(id);
                let _ = response_channel.send(ticket.cloned());
            }
            Err(_) => {
                // There are no more senders, so we can safely break
                // and shut down the server.
                break;
            }
        }
    }
}
