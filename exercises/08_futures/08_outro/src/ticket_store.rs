use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Ticket {
    pub(crate) id: u64,
    pub(crate) title: String,
    description: String,
}

impl Ticket {
    pub fn new(id: u64, title: String, description: String) -> Ticket {
        Ticket {
            id,
            title,
            description,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum TicketMessage {
    Create(Ticket),
    Get(u64),
    Patch(TicketPatch),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TicketPatch {
    title: Option<String>,
    description: Option<String>,
    id: u64,
}

#[derive(Debug)]
pub struct TicketError(pub String);

#[derive(Default, Debug)]
pub struct TicketStore {
    data: RwLock<TicketData>,
}

#[derive(Default, Debug)]
struct TicketData {
    last_id: u64,
    tickets: HashMap<u64, Ticket>,
}

impl TicketStore {
    pub async fn handle_message(&self, message: &TicketMessage) -> Result<Ticket, TicketError> {
        match message {
            TicketMessage::Create(ticket) if ticket.description.len() > 256 => {
                Err(TicketError(String::from("Description too long, max 256.")))
            }
            TicketMessage::Create(ticket) if ticket.title.len() > 128 => {
                Err(TicketError(String::from("Title too long, max 128.")))
            }
            TicketMessage::Create(ticket) => {
                let mut data = self.data.write().await;
                data.last_id += 1;
                let new_ticket: Ticket = Ticket {
                    id: data.last_id,
                    title: ticket.title.clone(),
                    description: ticket.description.clone(),
                };
                data.tickets.insert(new_ticket.id, new_ticket.clone());
                Ok(new_ticket)
            }
            TicketMessage::Get(id) => {
                let data = self.data.read().await;
                if let Some(ticket) = data.tickets.get(id) {
                    Ok(ticket.clone())
                } else {
                    Err(TicketError(String::from("Ticket not found.")))
                }
            }
            TicketMessage::Patch(ticket_patch) => {
                let mut data = self.data.write().await;
                if let Some(ticket) = data.tickets.get_mut(&ticket_patch.id) {
                    if let Some(description) = &ticket_patch.description {
                        ticket.description = description.clone();
                    }
                    if let Some(title) = &ticket_patch.title {
                        ticket.title = title.clone();
                    }
                    Ok(ticket.clone())
                } else {
                    Err(TicketError(String::from("Ticket not found.")))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn ticket_tester() {}

    #[tokio::test]
    async fn ticket_store_create_tests() {
        let ts: TicketStore = TicketStore::default();
        for _ in [0..50] {
            let ticket = Ticket {
                id: 99,
                title: "Title".to_string(),
                description: "Description".to_string(),
            };
            let create = TicketMessage::Create(ticket.clone());
            let response = ts.handle_message(&create).await;
            assert!(response.is_ok());

            let created_ticket = response.unwrap();

            assert_ne!(&created_ticket.id, &99);
            assert_eq!(&created_ticket.description, &ticket.description);
            assert_eq!(&created_ticket.title, &ticket.title);
        }
    }

    #[tokio::test]
    async fn ticket_store_get_tests() {
        let ts: TicketStore = TicketStore::default();

        let ticket = Ticket {
            id: 99,
            title: "Title".to_string(),
            description: "Description".to_string(),
        };
        let create = TicketMessage::Create(ticket.clone());
        let response = ts.handle_message(&create).await;
        assert!(response.is_ok());

        let get_ticket = ts.handle_message(&TicketMessage::Get(1)).await.unwrap();
        assert_eq!(&get_ticket.id, &1);
        assert_eq!(&get_ticket.description, &ticket.description);
        assert_eq!(&get_ticket.title, &ticket.title);
    }

    #[tokio::test]
    async fn ticket_store_patch_tests() {
        let ts: TicketStore = TicketStore::default();

        let ticket = Ticket {
            id: 99,
            title: "Title".to_string(),
            description: "Description".to_string(),
        };
        let create = TicketMessage::Create(ticket.clone());
        let response = ts.handle_message(&create).await;
        assert!(response.is_ok());

        // Bad patch, id doesn't match.
        let patch = TicketPatch {
            title: None,
            description: None,
            id: 1111,
        };
        let response = ts.handle_message(&TicketMessage::Patch(patch)).await;
        assert!(response.is_err());
        if let Err(TicketError(error)) = response {
            println!("{error}");
        } else {
            assert!(false, "Didn't catch patch error!");
        }
    }
}
