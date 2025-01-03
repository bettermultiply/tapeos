// in this file, we will implement the general interpreter for the intent.

use crate::base::intent::Intent;
use tch::{nn, Device, Tensor};
use std::collections::HashMap;

// Simple neural network for command selection
struct CommandSelector {
    embedding: nn::Embedding,
    lstm: nn::LSTM,
    linear: nn::Linear,
}

impl CommandSelector {
    fn new(vs: &nn::Path, vocab_size: i64, embedding_dim: i64, hidden_size: i64, num_commands: i64) -> Self {
        let embedding = nn::embedding(vs, vocab_size, embedding_dim, Default::default());
        let lstm = nn::lstm(vs, embedding_dim, hidden_size, Default::default());
        let linear = nn::linear(vs, hidden_size, num_commands, Default::default());
        
        CommandSelector {
            embedding,
            lstm,
            linear
        }
    }
    
    fn forward(&self, input: &Tensor) -> Tensor {
        let embedded = self.embedding.forward(input);
        let lstm_out = self.lstm.forward(&embedded);
        let final_hidden = lstm_out.0.select(0, -1);
        self.linear.forward(&final_hidden)
    }
}


fn interprete(intent: Intent) {}

fn select_best_command(intent_desc: &str, commands: &[&str]) -> String {
    // Initialize model
    let vs = nn::VarStore::new(Device::Cpu);
    let vocab_size = 10000; // Vocabulary size
    let embedding_dim = 128;
    let hidden_size = 256;
    let model = CommandSelector::new(&vs.root(), vocab_size, embedding_dim, hidden_size, commands.len() as i64);

    // Convert intent description to tensor
    // This is simplified - would need proper tokenization in practice
    let input = Tensor::of_slice(&[1, 2, 3]); // Placeholder tokenized input
    
    // Get model predictions
    let outputs = model.forward(&input);
    let best_idx = outputs.argmax(None, false);
    
    // Return best matching command
    commands[best_idx.int64_value(&[]) as usize].to_string()
}
