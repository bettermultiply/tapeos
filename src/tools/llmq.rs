use deepseek_api_client::{
    chat_completion, chat_completion_sync, get_response_text, Message
};
use log::info;

const API_KEY: &str = "sk-022b0b782c2849f4a37ff736374825bd";

pub async fn prompt(s_prompt: &str, u_prompt: &str) -> String {

    let mut llm_completion = chat_completion(API_KEY) ;

    let messages = vec![
        Message {
            role: "system".to_owned(),
            content: s_prompt.to_string(),
        },
        Message {
            role: "user".to_owned(),
            content: u_prompt.to_string(),
        },
    ]; 

    let res = llm_completion(messages).await;
    let res_text = get_response_text(&res.unwrap(), 0);
    info!("{}", res_text.as_ref().unwrap());
    res_text.unwrap()
}

// key sk-022b0b782c2849f4a37ff736374825bd