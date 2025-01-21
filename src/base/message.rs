use std::fmt::{self, Display};

// maybe we should not focus on puzzling message but simple structed information.
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Message {
    m_type: MessageType,

    m_body: String,    
    // in actual, this is id of intent.
    m_id: Option<i64>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageType {
    Intent,
    Response,
    Reject,
    Finish,
    Register,
    Heartbeat,
    Status,
    Unknown,
}

impl Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MessageType::Intent => write!(f, "Intent"),
            MessageType::Response => write!(f, "Response"),
            MessageType::Reject => write!(f, "Reject"),
            MessageType::Finish => write!(f, "Finish"),
            MessageType::Register => write!(f, "Register"),
            MessageType::Heartbeat => write!(f, "Heartbeat"),
            MessageType::Status => write!(f, "Status"),
            MessageType::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Message {
    pub fn new(m_type: MessageType, m_body: String, m_id: Option<i64>) -> Self {
        Self {
            m_type,
            m_body,
            m_id,
        }
    }

    pub fn get_type(&self) -> &MessageType {
        &self.m_type
    }

    pub fn get_body(&self) -> String {
        self.m_body.clone()
    }

    pub fn get_id(&self) -> Option<i64> {
        self.m_id.clone()
    }
}