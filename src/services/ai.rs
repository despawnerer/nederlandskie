use anyhow::Result;
use chat_gpt_lib_rs::{ChatGPTClient, ChatInput, Message, Model, Role};

pub struct AI {
    chat_gpt_client: ChatGPTClient,
}

impl AI {
    pub fn new(api_key: &str, base_url: &str) -> Self {
        Self {
            chat_gpt_client: ChatGPTClient::new(api_key, base_url),
        }
    }

    pub async fn infer_country_of_living(
        &self,
        display_name: &str,
        description: &str,
    ) -> Result<String> {
        let chat_input = ChatInput {
            model: Model::Gpt3_5Turbo,
            messages: vec![
                Message {
                    role: Role::System,
                    // TODO: Lol, prompt injection much?
                    content: "You are a tool that attempts to guess where a person is likely to be from based on their name and short bio. Please respond with two-letter country code only. If unable to determine, say xx.".to_string(),
                },
                Message {
                    role: Role::User,
                    content: format!("Name: {display_name}\nBio:\n{description}"),
                },
            ],
            ..Default::default()
        };

        let response = self.chat_gpt_client.chat(chat_input).await?;

        // TODO: Error handling?
        return Ok(response.choices[0].message.content.to_lowercase());
    }
}
