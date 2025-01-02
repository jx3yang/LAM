use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Title {
    pub romaji: Option<String>,
    pub english: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AnimeMetadata {
    pub id: i32,
    pub title: Title,
    pub season: Option<String>,

    #[serde(rename = "seasonYear")]
    pub season_year: i32,
    pub description: Option<String>,
    pub popularity: Option<i32>,

    #[serde(rename = "meanScore")]
    pub mean_score: Option<i32>,

    pub genres: Option<Vec<String>>,
}
