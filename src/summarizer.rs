use std::time::Duration;

use reqwest::{Client, Error};
use serde_json::json;
use tokio::time::sleep;
use tokio::sync::mpsc;
use futures::stream::{self, StreamExt};

use crate::types::{AnimeMetadata, AnimeSummary};

pub struct Summarizer {
    receiver: mpsc::Receiver<Option<Vec<AnimeMetadata>>>,
    sender: mpsc::Sender<Option<AnimeSummary>>,
    url: String,
    api_key: String,
}

impl Summarizer {
    pub fn new(
        receiver: mpsc::Receiver<Option<Vec<AnimeMetadata>>>,
        sender: mpsc::Sender<Option<AnimeSummary>>,
    ) -> Self {
        let url = "https://api.groq.com/openai/v1/chat/completions";
        let api_key = std::env::var("GROQ_API_KEY_LAM")
            .expect("Environment variable GROQ_API_KEY must be set");
        Self {
            receiver,
            sender,
            url: url.to_string(),
            api_key: api_key,
        }
    }

    pub async fn start_summarize_job(
        &mut self,
    ) -> Result<bool, Error> {
        loop {
            match self.receiver.recv().await {
                Some(maybe_data) => {
                    match maybe_data {
                        Some(data) => {
                            println!("Summarizer has received data");
                            if data.is_empty() {
                                continue;
                            }
                            stream::iter(data)
                                .map(|anime| async {
                                    let anime_id = anime.id;
                                    let response = Self::summarize_anime(&self.url, &self.api_key, anime).await.map(|response| Self::parse_response(response, anime_id));
                                    if response.is_err() {
                                        println!("Summarize error: {:?}", response.unwrap_err());
                                        return;
                                    }
                                    let maybe_data = response.unwrap();
                                    if maybe_data.is_none() {
                                        println!("Got none after summarizing anime");
                                        return;
                                    }
                                    let send_result = self.sender.send(maybe_data).await;
                                    if send_result.is_err() {
                                        println!("Summary send error: {:?}", send_result.unwrap_err());
                                    }
                                })
                                .buffer_unordered(1)
                                .collect::<Vec<_>>()
                                .await;
                        },
                        None => {
                            println!("Finished receiving metadata, ending summarizer job");
                            let _ = self.sender.send(None).await;
                            println!("Finished sending anime summaries");
                            return Ok(true);
                        },
                    }
                },
                None => return Ok(false),
            };
        }
    }

    async fn summarize_anime(
        url: &String,
        api_key: &String,
        anime: AnimeMetadata
    ) -> Result<serde_json::Value, Error> {
        let mut retry = -1;
        loop {
            retry += 1;
            if retry == 3 {
                println!("Tried 3 times, skip to next request...");
                return Ok(serde_json::Value::Null);
            }
            let payload = json!({
                "messages": [
                    {
                        "role": "system",
                        "content": "You are an expert in animes. Given the title of an anime and a description, generate a 2 sentence summary as well as some related keywords such as themes and genres.\n\nUse the following output format in json:\n\n{\n  \"summary\": \"summary of the anime\",\n  \"themes\": [\"theme1\", \"theme2\"],\n  \"genres\": [\"genre1\", \"genre2\"]\n}"
                    },
                    {
                        "role": "user",
                        "content": format!("Title: {}\nDescription: {}", anime.title.english.clone().or(anime.title.romaji.clone()).unwrap_or_default(), anime.description.clone().unwrap_or_default()),
                    }
                ],
                "model": "llama-3.3-70b-versatile",
                "temperature": 1,
                "max_tokens": 1024,
                "top_p": 1,
                "stream": false,
                "response_format": {
                    "type": "json_object"
                },
                "stop": null
            });

            let client = Client::new();
            let response = client.post(url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", api_key))
                .body(payload.to_string())
                .send()
                .await
                .unwrap();

            if response.status() == 429 {
                let retry_after = response.headers()
                    .get("retry-after")
                    .map(|x| x.to_str())
                    .unwrap_or(Ok("10"))
                    .unwrap()
                    .parse::<i32>()
                    .unwrap();
                println!("Hit limit, sleeping for {} seconds", retry_after);
                sleep(Duration::from_secs(retry_after.try_into().unwrap())).await;
                continue;
            }

            if response.status() != 200 {
                println!("Got the following: {}", response.status());
                println!("Sleeping for 10 seconds");
                sleep(Duration::from_secs(10)).await;
                continue;
            }
            return response.text()
                .await
                .map(|response| {
                    serde_json::from_str(&response).unwrap()
                });
        }
    }

    fn parse_response(
        data: serde_json::Value,
        anime_id: i32,
    ) -> Option<AnimeSummary> {
        if data.is_null() {
            return None;
        }
        // println!("{:?}", data["choices"][0]["message"]["content"].as_str().clone());
        let anime_summary = match serde_json::from_str(data["choices"][0]["message"]["content"].as_str().clone().unwrap()) {
            Ok(summary) => Some(summary),
            Err(e) => {
                println!("Error: {}", e);
                None
            }
        };
        anime_summary.map(|generated_summary| {
            AnimeSummary {
                id: anime_id,
                generated_summary: generated_summary
            }
        })
    }
}
