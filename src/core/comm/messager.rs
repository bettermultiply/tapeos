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

// a struct to transfer the message between bluetooth and the outer world.
struct BluetoothMessage {
    message_type: MessageType,
    content: String,
}

impl BluetoothMessage {
    pub fn new(message_type: MessageType, content: String) -> Self {
        Self { message_type, content }
    }
}

impl Message for BluetoothMessage {
    fn send_message(&self) -> bool {
        // TODO: implement the logic to send the message to the outer world
        true
    }

    fn receive_message(&self) -> bool {
        // TODO: implement the logic to receive the message from the outer world
        true
    }
}
