use genai::chat::{ChatMessage, ChatRequest};
use genai::resolver::{AuthData, AuthResolver};
use genai::Client;

const API_KEY: &str = "sk-022b0b782c2849f4a37ff736374825bd";
const MODEL: &str = "deepseek-chat";

pub async fn prompt(s_prompt: &str, u_prompt: &str) -> String {
    let chat_req = ChatRequest::new(vec![
		// -- Messages (de/activate to see the differences)
		ChatMessage::system(s_prompt),
		ChatMessage::user(u_prompt),
	]);
    let auth_resolver = 
    AuthResolver::from_resolver_fn(
		|_| -> Result<Option<AuthData>, genai::resolver::Error> {
			let key = API_KEY.to_string();
			Ok(Some(AuthData::from_single(key)))
		},
	);
    
    let client = Client::builder().with_auth_resolver(auth_resolver).build();

	let chat_res = client.exec_chat(MODEL, chat_req.clone(), None).await.unwrap();
	
    chat_res.content_text_as_str().unwrap_or("NO ANSWER").to_string()
}
