use sqlx::{Result, SqliteConnection};
use tokio::sync::mpsc;

use crate::types::{AnimeMetadata, AnimeMetadataRow, Title};

#[derive(Debug, sqlx::FromRow)]
struct Years {
    max_year: i32,
    min_year: i32,
}

pub struct DbQuery {
    sender: mpsc::Sender<Option<Vec<AnimeMetadata>>>,
    conn: SqliteConnection,
}

impl DbQuery {
    pub fn new(sender: mpsc::Sender<Option<Vec<AnimeMetadata>>>,conn: SqliteConnection) -> Self {
        Self { sender, conn }
    }

    pub async fn query_all_years(&mut self) -> Result<bool> {
        let years: Years = sqlx::query_as("SELECT MAX(season_year) AS max_year, MIN(season_year) AS min_year FROM anime_metadata;").fetch_one(&mut self.conn).await?;
        for year in years.min_year..years.max_year+1 {
            let rows = self.query_year(year).await.unwrap();
            println!("Year: {}, num rows: {}", year, rows.len());
            if rows.is_empty() {
                continue;
            }
            let _ = self.sender.send(Some(rows)).await;
        }
        let _ = self.sender.send(None).await;
        println!("Finished sending metadata");
        Ok(true)
    }

    pub async fn query_year(&mut self, season_year: i32) -> Result<Vec<AnimeMetadata>> {
        let rows: Vec<AnimeMetadataRow> = sqlx::query_as("
            SELECT * FROM anime_metadata
            WHERE LOWER(genres) NOT LIKE '%hen%'
                AND description <> ''
                AND description IS NOT NULL
                AND season_year = ?
                AND id NOT IN (
                    SELECT id FROM anime_summary
                );
            ").bind(season_year).fetch_all(&mut self.conn).await?;
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
