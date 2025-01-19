use std::fmt::Error;

use futures::future;
use lam::{constants::DATABASE_URL, db_loader::{DbLoader, SummaryLoader}, db_query::DbQuery, summarizer::Summarizer, types::{AnimeMetadata, AnimeSummary}};
use sqlx::{Connection, SqliteConnection};
use tokio::{sync::mpsc, task};

macro_rules! zip {
    ($x: expr) => ($x);
    ($x: expr, $($y: expr), +) => (
        $x.into_iter().zip(
            zip!($($y), +))
    )
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let api_keys: Vec<String> = std::env::var("GROQ_API_KEYS_LAM")
            .expect("Environment variable GROQ_API_KEYS_LAM must be set")
            .split("---")
            .map(|s| s.to_string())
            .collect();
    let mut metadata_senders: Vec<mpsc::Sender<Option<AnimeMetadata>>> = vec![];
    let mut metadata_receivers: Vec<mpsc::Receiver<Option<AnimeMetadata>>> = vec![];

    for _ in 0..api_keys.len() {
        let (metadata_sender, metadata_receiver) = mpsc::channel::<Option<AnimeMetadata>>(1);
        metadata_senders.push(metadata_sender);
        metadata_receivers.push(metadata_receiver);
    }

    let (ready_sender, ready_receiver) = mpsc::channel::<usize>(api_keys.len()+1);

    // let (metadata_sender, metadata_receiver) = mpsc::channel::<Option<AnimeMetadata>>(4);
    let (summary_sender, summary_receiver) = mpsc::channel::<Option<AnimeSummary>>(128);
    let db_query_handle = task::spawn(async move {
        let conn = SqliteConnection::connect(DATABASE_URL).await.unwrap();
        let mut db_query = DbQuery::new(metadata_senders, ready_receiver, conn);
        db_query.query_all_years().await
    });

    let db_loader_handle = task::spawn(async move {
        let conn = SqliteConnection::connect(DATABASE_URL).await.unwrap();
        let mut db_loader = SummaryLoader::new(summary_receiver, conn);
        db_loader.start_load_job().await
    });

    let summarizer_handle = task::spawn(async move {
        let range: Vec<usize> = (0..api_keys.len()).collect();
        let zipped = zip!(range, metadata_receivers, api_keys);
        future::try_join_all(
            zipped.map(|(idx, (metadata_receiver, api_key))| {
                let summary_sender_clone = summary_sender.clone();
                let ready_sender_clone = ready_sender.clone();
                task::spawn(async move {
                    let mut summarizer = Summarizer::new(
                        metadata_receiver,
                        summary_sender_clone,
                        ready_sender_clone,
                        idx,
                        api_key,
                    );
                    summarizer.start_summarize_job().await
                })
            })
        ).await
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
