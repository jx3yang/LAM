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

#[derive(sqlx::FromRow, Debug)]
pub struct AnimeMetadataRow {
    pub id: i32,
    pub english_title: Option<String>,
    pub romaji_title: Option<String>,
    pub season: Option<String>,

    pub season_year: i32,
    pub description: Option<String>,
    pub popularity: Option<i32>,

    pub mean_score: Option<i32>,

    pub genres: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AnimeGeneratedSummary {
    pub summary: String,
    #[serde(rename = "genres")]
    pub generated_genres: Vec<String>,
    #[serde(rename = "themes")]
    pub generated_themes: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AnimeSummary {
    pub id: i32,
    pub generated_summary: AnimeGeneratedSummary
}
