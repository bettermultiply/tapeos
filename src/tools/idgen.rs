// in this file, we will wrap the functions to generate the id for rules, intents, etc.
// https://github.com/BobAnkh/idgenerator?tab=readme-ov-file

use idgenerator::*;

pub fn init_id_generator() -> Result<(), OptionError> {

    let mut options: Vec<IdGeneratorOptions> = vec![];
    for i in 0..(IdType::Len as usize) {
        options.push(IdGeneratorOptions::new().worker_id(i as u32).worker_id_bit_len(6).seq_bit_len(12));
    }
    let _ = IdVecInstance::init(options);
    Ok(())
}

// generate the id for the given id_type
pub fn generate_id(id_type: IdType) -> i64 {
    IdVecInstance::next_id(id_type as usize)
}

pub enum IdType {
    Rule = 0,
    Intent,
    Resource,
    //Len, used to check the length of the id, but we have better ways to do this.
    Len,
}
