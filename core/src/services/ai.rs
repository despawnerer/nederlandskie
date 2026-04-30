use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct AI {
    client: Client,
    api_key: String,
}

#[derive(Serialize)]
struct RequestMessage {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: &'static str,
    max_tokens: u32,
    system: &'static str,
    messages: Vec<RequestMessage>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    kind: String,
    text: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

impl AI {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
        }
    }

    pub async fn infer_country_of_living(
        &self,
        display_name: &str,
        description: &str,
    ) -> Result<String> {
        let request = AnthropicRequest {
            model: "claude-haiku-4-5-20251001",
            max_tokens: 10,
            // TODO: Lol, prompt injection much?
            system: "You are a tool that attempts to guess where a person is likely to be from based on their name and short bio. Please respond with two-letter country code only. If unable to determine, say xx.",
            messages: vec![RequestMessage {
                role: "user",
                content: format!("Name: {display_name}\nBio:\n{description}"),
            }],
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Anthropic API error {status}: {body}"));
        }

        let response = response.json::<AnthropicResponse>().await?;

        response
            .content
            .into_iter()
            .find(|b| b.kind == "text")
            .map(|b| b.text.trim().to_lowercase())
            .ok_or_else(|| anyhow!("No text content received from Claude"))
    }
}
