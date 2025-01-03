use std::fmt::Error;

use lam::{constants::DATABASE_URL, db_loader::{DbLoader, SummaryLoader}, db_query::DbQuery, summarizer::Summarizer, types::{AnimeMetadata, AnimeSummary}};
use sqlx::{Connection, SqliteConnection};
use tokio::{sync::mpsc, task};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (metadata_sender, metadata_receiver) = mpsc::channel::<Option<Vec<AnimeMetadata>>>(4);
    let (summary_sender, summary_receiver) = mpsc::channel::<Option<AnimeSummary>>(128);
    let db_query_handle = task::spawn(async move {
        let conn = SqliteConnection::connect(DATABASE_URL).await.unwrap();
        let mut db_query = DbQuery::new(metadata_sender, conn);
        db_query.query_all_years().await
    });

    let db_loader_handle = task::spawn(async move {
        let conn = SqliteConnection::connect(DATABASE_URL).await.unwrap();
        let mut db_loader = SummaryLoader::new(summary_receiver, conn);
        db_loader.start_load_job().await
    });

    let summarizer_handle = task::spawn(async move {
        let mut anime_summarizer = Summarizer::new(
            metadata_receiver,
            summary_sender,
        );
        anime_summarizer.start_summarize_job().await
    });

    let results = tokio::try_join!(
        db_query_handle,
        db_loader_handle,
        summarizer_handle,
    );

    if results.is_err() {
        eprintln!("{:?}", results);
    }

    Ok(())
}
