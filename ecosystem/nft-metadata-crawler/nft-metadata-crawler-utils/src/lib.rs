// Copyright © Aptos Foundation

use std::error::Error;

use reqwest::Client;
use serde_json::{json, Value};

pub async fn publish_to_queue(
    client: &Client,
    msg: String,
    auth: &String,
    topic_name: &String,
) -> Result<String, Box<dyn Error>> {
    let url = format!("https://pubsub.googleapis.com/v1/{}:publish", topic_name);

    let res = client
        .post(&url)
        .bearer_auth(auth)
        .json(&json!({
            "messages": [
                {
                    "data": base64::encode(msg.clone())
                }
            ]
        }))
        .send()
        .await?;

    match res.status().as_u16() {
        200..=299 => Ok(msg),
        _ => Err(format!("Error publishing to queue: {}", res.text().await?).into()),
    }
}

pub async fn consume_from_queue(
    client: &Client,
    auth: &String,
    subsctiption_name: &String,
) -> Result<Vec<String>, Box<dyn Error>> {
    let url = format!(
        "https://pubsub.googleapis.com/v1/{}:pull",
        subsctiption_name
    );

    let res = client
        .post(&url)
        .bearer_auth(auth)
        .json(&json!({
            "maxMessages": 5
        }))
        .send()
        .await?;

    let body: Value = res.json().await?;
    if let Some(messages) = body["receivedMessages"].as_array() {
        let mut links = Vec::new();
        for message in messages {
            let msg = message["message"]["data"].as_str();
            if let Some(msg) = msg {
                links.push(String::from_utf8(base64::decode(msg)?)?);
            }
        }
        Ok(links)
    } else {
        return Err("No message found".into());
    }
}
