// // in this file, we will implement the general interpreter for the intent.

// use crate::base::intent::Intent;
// use tch::{nn, Device, Tensor};
// use std::collections::HashMap;

// // Simple neural network for command selection
// struct CommandSelector {
//     embedding: nn::Embedding,
//     lstm: nn::LSTM,
//     linear: nn::Linear,
// }

use rust_bert::pipelines::zero_shot_classification::ZeroShotClassificationModel;

use crate::base::errort::BoxResult;

pub fn command_selector(command: &[&str], intent: &str) -> BoxResult<String> {
    let sequence_classification_model = ZeroShotClassificationModel::new(Default::default ())?;

    // let candidate_labels = & ["politics", "public health", "economics", "sports"];

    let output = sequence_classification_model.predict_multilabel(
        &[intent],
        command,   
        None,
        128,
    )?[0][0].text.clone();
    Ok(output)
}

