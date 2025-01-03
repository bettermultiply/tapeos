use async_openai::Client;
use async_openai::types::CreateCompletionRequestArgs;

pub async fn prompt(prompt: &str) -> String {
    return "".to_string();
    // Create client
    let client = Client::new();
    // Create request using builder pattern
    // Every request struct has companion builder struct with same name + Args suffix
    let request = CreateCompletionRequestArgs::default()
        .model("gpt-3.5-turbo-instruct")
        .prompt(prompt)
        .max_tokens(40_u32)
        .build()
        .unwrap();
   
    // Call API
    let response = client
        .completions()      // Get the API "group" (completions, images, etc.) from the client
        .create(request)    // Make the API call in that "group"
        .await
        .unwrap();
    
    let response_txt = response.choices.first().unwrap().text.clone();
    response_txt
}
