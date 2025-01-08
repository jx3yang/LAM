use sqlx::{Result, SqliteConnection};
use tokio::sync::mpsc;

use crate::types::{AnimeMetadata, AnimeMetadataRow, Title};

#[derive(Debug, sqlx::FromRow)]
struct Years {
    max_year: i32,
    min_year: i32,
}

pub struct DbQuery {
    senders: Vec<mpsc::Sender<Option<AnimeMetadata>>>,
    ready_receiver: mpsc::Receiver<usize>,
    conn: SqliteConnection,
}

impl DbQuery {
    pub fn new(senders: Vec<mpsc::Sender<Option<AnimeMetadata>>>, ready_receiver: mpsc::Receiver<usize>, conn: SqliteConnection) -> Self {
        Self { senders, ready_receiver, conn }
    }

    pub async fn query_all_years(&mut self) -> Result<bool> {
        let years: Years = sqlx::query_as("SELECT MAX(season_year) AS max_year, MIN(season_year) AS min_year FROM anime_metadata;").fetch_one(&mut self.conn).await?;
        for year in years.min_year..years.max_year+1 {
            let rows = self.query_year(year).await.unwrap();
            println!("Year: {}, num rows: {}", year, rows.len());
            if rows.is_empty() {
                continue;
            }
            self.handle_year(rows).await;
            // let _ = self.sender.send(Some(rows)).await;
        }
        // let _ = self.sender.send(None).await;
        for idx in 0..self.senders.len() {
            let _ = self.senders[idx].send(None).await;
        }
        println!("Finished sending metadata");
        Ok(true)
    }

    async fn handle_year(&mut self, rows: Vec<AnimeMetadata>) {
        for row in rows {
            match self.ready_receiver.recv().await {
                Some(idx) => {
                    let response = self.senders[idx].send(Some(row)).await;
                    if response.is_err() {
                        println!("Error sending metadata to {}, err: {}", idx, response.unwrap_err());
                    }
                },
                None => todo!(),
            }
        }
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
