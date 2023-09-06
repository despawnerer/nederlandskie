use anyhow::Result;
use chat_gpt_lib_rs::{ChatGPTClient, ChatInput, Message, Model, Role};

pub type AI = ChatGPTClient;

pub fn make_ai_client() -> AI {
    // TODO: Take key from env vars
    let api_key = "fake-api-key";
    let base_url = "https://api.openai.com";
    return ChatGPTClient::new(api_key, base_url);
}

pub async fn infer_country_of_living(
    ai: &AI,
    display_name: &str,
    description: &str,
) -> Result<String> {
    let chat_input = ChatInput {
        model: Model::Gpt3_5Turbo,
        messages: vec![
            Message {
                role: Role::System,
                // TODO: Lol, prompt injection much?
                content: "You are a tool that attempts to guess where a person is likely to be from based on their name and short bio. Please respond with two-letter country code only. Use lowercase letters.".to_string(),
            },
            Message {
                role: Role::User,
                content: format!("Name: {display_name}\nBio:\n{description}"),
            },
        ],
        ..Default::default()
    };

    let response = ai.chat(chat_input).await?;

    // TODO: Error handling?
    return Ok(response.choices[0].message.content.clone());
}
