// TODO: design the message structure used to communicate with the outer world.

pub enum MessageType {
    IntentRequest,
    IntentResponse,
    StartRequest,
    StopRequest,
    FinishResponse,
}

pub trait Message {
    fn send_message(&self) -> bool;
    fn receive_message(&self) -> bool;
}