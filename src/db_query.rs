use sqlx::{Result, SqliteConnection};

use crate::types::{AnimeMetadata, AnimeMetadataRow, Title};

pub struct DbQuery {
    conn: SqliteConnection,
}

impl DbQuery {
    pub fn new(conn: SqliteConnection) -> Self {
        Self { conn }
    }

    pub async fn query_year(&mut self, season_year: i32) -> Result<Vec<AnimeMetadata>> {
        let rows: Vec<AnimeMetadataRow> = sqlx::query_as("SELECT * FROM anime_metadata WHERE season_year = ?;").bind(season_year).fetch_all(&mut self.conn).await?;
        let media = rows.into_iter().map(|row| {
            AnimeMetadata {
                id: row.id,
                title: Title {
                    romaji: row.romaji_title,
                    english: row.english_title,
                },
                season: row.season,
                season_year: row.season_year,
                description: row.description,
                popularity: row.popularity,
                mean_score: row.mean_score,
                genres: row.genres.map(|g| g.split(",").into_iter().map(|x| x.to_string()).collect()),
            }
        }).collect();
        Ok(media)
    }
}
