// in this file, we will implement the disassembler.

use crate::base::intent::{Intent, SubIntent};

pub fn disassembler<'a>(intent: &mut Intent<'a>) -> Option<()> {
    #[allow(unused_mut)]
    let mut sub_intents: Vec<SubIntent<'a>> = disassemble_intent(intent.get_description());
    // Logic to disassemble the intent based on resources

    if !sub_intents.is_empty() {
        intent.add_sub_intent(sub_intents);
    }

    Some(())
}

fn disassemble_intent<'a>(intent: &str) -> Vec<SubIntent<'a>> {
    // TODO: use ai here to disassemble the intent.
    // maybe we should design prompt here.
    vec![]
}