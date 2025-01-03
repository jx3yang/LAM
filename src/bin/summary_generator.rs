use std::fmt::Error;

use lam::{constants::DATABASE_URL, db_query::DbQuery, summarizer::Summarizer, types::{AnimeMetadata, AnimeSummary}};
use sqlx::{Connection, SqliteConnection};
use tokio::{sync::mpsc, task};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let conn = SqliteConnection::connect(DATABASE_URL).await;
    if conn.is_err() {
        eprintln!("Could not form connection to DB: {:?}", conn);
        return Ok(());
    }
    
    let (metadata_sender, metadata_receiver) = mpsc::channel::<Option<Vec<AnimeMetadata>>>(4);
    let (summary_sender, mut summary_receiver) = mpsc::channel::<Option<AnimeSummary>>(128);
    let mut anime_summarizer = Summarizer::new(
        metadata_receiver,
        summary_sender,
    );
    let mut db_query = DbQuery::new(conn.unwrap());
    let anime = db_query.query_year(2025)
        .await
        .unwrap().remove(1);
    task::spawn(async move {
        println!("{:?}", anime);
        metadata_sender.send(Some(vec![anime])).await;
    });
    task::spawn(async move {
        anime_summarizer.start_summarize_job().await;
    });
    
    println!("{:?}", summary_receiver.recv().await.unwrap().unwrap());

    Ok(())
}
