use sqlx::{Result, SqliteConnection};
use tokio::sync::mpsc;

use crate::types::{AnimeMetadata, AnimeSummary};

#[allow(async_fn_in_trait)]
pub trait DbLoader<T> {
    fn loader_name(&mut self) -> String;
    fn get_conn(&mut self) -> &mut SqliteConnection;
    fn get_receiver(&mut self) -> &mut mpsc::Receiver<Option<Vec<T>>>;

    async fn create_table_if_not_exists(conn: &mut SqliteConnection) -> Result<()>;
    async fn load(conn: &mut SqliteConnection, data: Vec<T>) -> Result<()>;

    async fn start_load_job(&mut self) -> Result<bool> {
        Self::create_table_if_not_exists(self.get_conn()).await?;
        loop {
            match self.get_receiver().recv().await {
                Some(maybe_data) => {
                    println!("Loader {} has received data", self.loader_name());
                    match maybe_data {
                        Some(data) => {
                            if data.is_empty() {
                                continue;
                            }
                            let response = Self::load(self.get_conn(), data).await;
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
}

pub struct SummaryLoader {
    receiver: mpsc::Receiver<Option<Vec<AnimeSummary>>>,
    conn: SqliteConnection,
}

impl DbLoader<AnimeSummary> for SummaryLoader {
    fn loader_name(&mut self) -> String {
        "SummaryLoader".to_string()
    }

    fn get_conn(&mut self) -> &mut SqliteConnection {
        &mut self.conn
    }

    fn get_receiver(&mut self) -> &mut mpsc::Receiver<Option<Vec<AnimeSummary>>> {
        &mut self.receiver
    }

    async fn create_table_if_not_exists(conn: &mut SqliteConnection) -> Result<()> {
        let sql = "
            CREATE TABLE IF NOT EXISTS anime_summary (
                id INTEGER PRIMARY KEY,
                summary TEXT,
                generated_genres TEXT,
                generated_themes TEXT
            );
        ";
        sqlx::query(sql).execute(conn).await?;
        Ok(())
    }

    async fn load(conn: &mut SqliteConnection, data: Vec<AnimeSummary>) -> Result<()> {
        let insert_sql = "
            INSERT OR REPLACE INTO anime_summary (id, summary, generated_genres, generated_themes)
        ";
        let mut query = sqlx::QueryBuilder::new(insert_sql);
        query.push_values(data, |mut b, anime| {
            b.push_bind(anime.id)
                .push_bind(anime.summary)
                .push_bind(anime.generated_genres.join(","))
                .push_bind(anime.generated_themes.join(","));
        });
        let built_query = query.build();
        // println!("{}", built_query.sql());
        built_query.execute(conn).await?;
        println!("Loaded!");
        Ok(())
    }
}

impl SummaryLoader {
    pub fn new(receiver: mpsc::Receiver<Option<Vec<AnimeSummary>>>, conn: SqliteConnection) -> Self {
        Self { receiver, conn }
    }
}

pub struct MetadataLoader {
    receiver: mpsc::Receiver<Option<Vec<AnimeMetadata>>>,
    conn: SqliteConnection,
}

impl DbLoader<AnimeMetadata> for MetadataLoader {
    fn loader_name(&mut self) -> String {
        "MetadataLoader".to_string()
    }

    fn get_conn(&mut self) -> &mut SqliteConnection {
        &mut self.conn
    }

    fn get_receiver(&mut self) -> &mut mpsc::Receiver<Option<Vec<AnimeMetadata>>> {
        &mut self.receiver
    }

    async fn create_table_if_not_exists(conn: &mut SqliteConnection) -> Result<()> {
        let sql = "
            CREATE TABLE IF NOT EXISTS anime_metadata (
                id INTEGER PRIMARY KEY,
                romaji_title TEXT,
                english_title TEXT,
                season TEXT,
                season_year INTEGER NOT NULL,
                description TEXT,
                popularity INTEGER,
                mean_score INTEGER,
                genres TEXT
            );
        ";
        sqlx::query(sql).execute(conn).await?;
        Ok(())
    }

    async fn load(conn: &mut SqliteConnection, data: Vec<AnimeMetadata>) -> Result<()> {
        let insert_sql = "
            INSERT OR REPLACE INTO anime_metadata (id, romaji_title, english_title, season, season_year, description, popularity, mean_score, genres)
        ";
        let mut query = sqlx::QueryBuilder::new(insert_sql);
        query.push_values(data, |mut b, anime| {
            b.push_bind(anime.id)
                .push_bind(anime.title.romaji.clone())
                .push_bind(anime.title.english.clone())
                .push_bind(anime.season.clone())
                .push_bind(anime.season_year)
                .push_bind(anime.description.clone())
                .push_bind(anime.popularity)
                .push_bind(anime.mean_score)
                .push_bind(anime.genres.unwrap_or_default().join(","));
        });
        let built_query = query.build();
        // println!("{}", built_query.sql());
        built_query.execute(conn).await?;
        println!("Loaded!");
        Ok(())
    }
}

impl MetadataLoader {
    pub fn new(receiver: mpsc::Receiver<Option<Vec<AnimeMetadata>>>, conn: SqliteConnection) -> Self {
        Self { receiver, conn }
    }
}
