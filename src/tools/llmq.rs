use genai::chat::{ChatMessage, ChatRequest};
use genai::resolver::{AuthData, AuthResolver};
use genai::Client;
const GEMINI: &str = "AIzaSyCTjRoY450HtHZYEzXW2CRafFQEcHm6Mkk";
// const API_KEY: &str = "sk-022b0b782c2849f4a37ff736374825bd";
const MODEL: &str = "gemini-2.0-flash-lite-preview-02-05";

pub async fn prompt(s_prompt: &str, u_prompt: &str) -> String {
	let s_prompt = "First of all, You should give the outcome as fast as possible.\n".to_string() + s_prompt;
    let chat_req = ChatRequest::new(vec![
		// -- Messages (de/activate to see the differences)
		ChatMessage::system(s_prompt),
		ChatMessage::user(u_prompt),
	]);
    let auth_resolver = 
    AuthResolver::from_resolver_fn(
		|_| -> Result<Option<AuthData>, genai::resolver::Error> {
			let key = GEMINI.to_string();
			Ok(Some(AuthData::from_single(key)))
		},
	);
    
    let client = Client::builder().with_auth_resolver(auth_resolver).build();

	for _ in 0..3 {
		match client.exec_chat(MODEL, chat_req.clone(), None).await {
			Ok(r) => return r.content_text_as_str().unwrap_or("NO ANSWER").to_string(),
			Err(_) => (),
		}
	}
    "".to_string()
}
