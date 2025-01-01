use sqlx::{Result, SqliteConnection};
use tokio::sync::mpsc;

use crate::types::AnimeMetadata;

pub struct DbLoader {
    receiver: mpsc::Receiver<Option<Vec<AnimeMetadata>>>,
    conn: SqliteConnection,
}

impl DbLoader {
    pub fn new(receiver: mpsc::Receiver<Option<Vec<AnimeMetadata>>>, conn: SqliteConnection) -> Self {
        Self { receiver, conn }
    }

    pub async fn load(&mut self) -> Result<bool> {
        DbLoader::create_table_if_not_exists(&mut self.conn).await?;
        loop {
            match self.receiver.recv().await {
                Some(maybe_media) => {
                    println!("Received!");
                    match maybe_media {
                        Some(media) => {
                            let response = DbLoader::load_metadata(&mut self.conn, media).await;
                            if response.is_err() {
                                println!("{:?}", response.unwrap_err());
                            }
                        },
                        None => return Ok(true),
                    }
                },
                None => return Ok(false),
            };
        }
    }

    pub async fn create_table_if_not_exists(conn: &mut SqliteConnection) -> Result<()> {
        let sql = "
            CREATE TABLE IF NOT EXISTS anime_metadata (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                season TEXT NOT NULL,
                season_year INTEGER NOT NULL,
                description TEXT NOT NULL,
                popularity INTEGER NOT NULL,
                mean_score INTGER NOT NULL
            );
        ";
        sqlx::query(sql).execute(conn).await?;
        Ok(())
    }

    pub async fn load_metadata(conn: &mut SqliteConnection, media: Vec<AnimeMetadata>) -> Result<()> {
        let insert_sql = "
            INSERT OR REPLACE INTO anime_metadata (id, title, season, season_year, description, popularity, mean_score)
        ";
        let mut query = sqlx::QueryBuilder::new(insert_sql);
        query.push_values(media, |mut b, anime| {
            b.push_bind(anime.id)
                .push_bind(anime.title.romaji.clone())
                .push_bind(anime.season.clone())
                .push_bind(anime.season_year)
                .push_bind(anime.description.clone())
                .push_bind(anime.popularity)
                .push_bind(anime.mean_score);
        });
        let built_query = query.build();
        // println!("{}", built_query.sql());
        built_query.execute(conn).await?;
        println!("Loaded!");
        Ok(())
    }
}
