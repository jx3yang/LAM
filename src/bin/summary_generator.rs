use std::fmt::Error;

use lam::{constants::DATABASE_URL, db_query::DbQuery};
use reqwest::Client;
// use reqwest::blocking::Client;
use serde_json::json;
use sqlx::{Connection, SqliteConnection};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let conn = SqliteConnection::connect(DATABASE_URL).await;
    if conn.is_err() {
        eprintln!("Could not form connection to DB: {:?}", conn);
        return Ok(());
    }
    let mut db_query = DbQuery::new(conn.unwrap());
    let anime = &db_query.query_year(2025)
        .await
        .unwrap()[0];

    // Define the API endpoint and API key
    let url = "https://api.groq.com/openai/v1/chat/completions";
    let api_key = std::env::var("GROQ_API_KEY_LAM")
        .expect("Environment variable GROQ_API_KEY must be set");

    // Create the JSON payload
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

    // Create the HTTP client and send the POST request
    let client = Client::new();
    let response = client.post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .body(payload.to_string())
        .send()
        .await
        .unwrap();

    // Print the response body
    let response_body = response.text().await;
    println!("{}", response_body.unwrap());

    Ok(())
}
