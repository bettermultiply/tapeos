// maybe we should not focus on puzzling message but simple structed information.
use serde::{Serialize, Deserialize};

// TODO

#[derive(Serialize, Deserialize)]
pub struct Message {
    m_type: MessageType,

    m_body: String,    
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageType {
    Intent,
    Response,
    Reject,
    Register,
    Heartbeat,
    Unknow,
}

impl Message {
    pub fn new(m_type: MessageType, m_body: String) -> Self {
        Self {
            m_type,
            m_body,
        }
    }

    pub fn get_type(&self) -> &MessageType {
        &self.m_type
    }

    pub fn get_body(&self) -> String {
        self.m_body.clone()
    }
}