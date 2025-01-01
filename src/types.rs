use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Title {
    pub romaji: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AnimeMetadata {
    pub id: i32,
    pub title: Title,
    pub season: String,

    #[serde(rename = "seasonYear")]
    pub season_year: i32,
    pub description: String,
    pub popularity: i32,

    #[serde(rename = "meanScore")]
    pub mean_score: i32,
}
