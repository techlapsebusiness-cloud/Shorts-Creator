use serde_json::Value;

use crate::error::AiError;
use crate::types::{Completion, TokenUsage};

pub fn parse_model_ids(body: &str) -> Result<Vec<String>, AiError> {
    let value: Value =
        serde_json::from_str(body).map_err(|error| AiError::Parse(error.to_string()))?;
    let data = value["data"]
        .as_array()
        .ok_or_else(|| AiError::Parse("models response has no data array".into()))?;
    let mut ids = Vec::new();
    for item in data {
        if let Some(id) = item["id"].as_str()
            && !id.is_empty()
        {
            ids.push(id.to_string());
        }
    }
    Ok(ids)
}

pub fn parse_anthropic_sse(body: &str) -> Result<Completion, AiError> {
    let mut deltas = Vec::new();
    let mut input = 0;
    let mut output = 0;
    for event in events(body)? {
        match event["type"].as_str() {
            Some("content_block_delta") if event["delta"]["type"] == "text_delta" => {
                if let Some(text) = event["delta"]["text"].as_str() {
                    deltas.push(text.to_string());
                }
            }
            Some("message_start") => {
                if let Some(tokens) = event["message"]["usage"]["input_tokens"].as_i64() {
                    input = tokens;
                }
            }
            Some("message_delta") => {
                if let Some(tokens) = event["usage"]["output_tokens"].as_i64() {
                    output = tokens;
                }
            }
            Some("error") => {
                let message = event["error"]["message"]
                    .as_str()
                    .unwrap_or("provider error");
                return Err(AiError::Provider(message.to_string()));
            }
            _ => {}
        }
    }
    completion(deltas, input, output)
}

pub fn parse_openai_sse(body: &str) -> Result<Completion, AiError> {
    let mut deltas = Vec::new();
    let mut input = 0;
    let mut output = 0;
    for event in events(body)? {
        match event["type"].as_str() {
            Some("response.output_text.delta") => {
                if let Some(text) = event["delta"].as_str() {
                    deltas.push(text.to_string());
                }
            }
            Some("response.completed") => {
                let usage = &event["response"]["usage"];
                if let Some(tokens) = usage["input_tokens"].as_i64() {
                    input = tokens;
                }
                if let Some(tokens) = usage["output_tokens"].as_i64() {
                    output = tokens;
                }
            }
            Some("response.failed" | "error") => {
                let message = event["response"]["error"]["message"]
                    .as_str()
                    .or_else(|| event["error"]["message"].as_str())
                    .unwrap_or("provider error");
                return Err(AiError::Provider(message.to_string()));
            }
            _ => {}
        }
    }
    completion(deltas, input, output)
}

fn completion(deltas: Vec<String>, input: i64, output: i64) -> Result<Completion, AiError> {
    if deltas.is_empty() {
        return Err(AiError::Empty);
    }
    let text = deltas.concat();
    Ok(Completion {
        text,
        deltas,
        usage: TokenUsage { input, output },
    })
}

fn events(body: &str) -> Result<Vec<Value>, AiError> {
    let mut parsed = Vec::new();
    for block in body.split("\n\n") {
        for line in block.lines() {
            let Some(data) = line.trim().strip_prefix("data:") else {
                continue;
            };
            let data = data.trim();
            if data.is_empty() || data == "[DONE]" {
                continue;
            }
            let value =
                serde_json::from_str(data).map_err(|error| AiError::Parse(error.to_string()))?;
            parsed.push(value);
        }
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::{parse_anthropic_sse, parse_model_ids, parse_openai_sse};

    #[test]
    fn anthropic_fixture_yields_ordered_deltas() {
        let completion = parse_anthropic_sse(include_str!("../fixtures/anthropic.sse")).unwrap();
        assert_eq!(completion.deltas, vec!["{\"ok\":", "true}"]);
        assert_eq!(completion.text, "{\"ok\":true}");
        assert_eq!(completion.usage.input, 11);
        assert_eq!(completion.usage.output, 4);
    }

    #[test]
    fn openai_fixture_yields_ordered_deltas() {
        let completion = parse_openai_sse(include_str!("../fixtures/openai.sse")).unwrap();
        assert_eq!(completion.deltas, vec!["{\"ok\":", "true}"]);
        assert_eq!(completion.text, "{\"ok\":true}");
        assert_eq!(completion.usage.input, 9);
        assert_eq!(completion.usage.output, 4);
    }

    #[test]
    fn model_ids_come_from_the_payload() {
        let ids = parse_model_ids(
            r#"{"data":[{"id":"model-a","object":"model"},{"id":""},{"id":"model-b"}]}"#,
        )
        .unwrap();
        assert_eq!(ids, vec!["model-a", "model-b"]);
    }
}
