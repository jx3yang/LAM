use lam::{downloader::Downloader, types::AnimeMetadata};
use lam::db_loader::DbLoader;
use sqlx::{migrate::MigrateDatabase, Connection, Error, Sqlite, SqliteConnection};
use tokio::sync::mpsc;
use tokio::task;

#[tokio::main]
async fn main() -> Result<(), Error> {
  let database_url = "sqlite://anime_metadata.db";
    if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
      println!("Creating database {}", database_url);
      match Sqlite::create_database(database_url).await {
          Ok(_) => println!("Create db success"),
          Err(error) => panic!("error: {}", error),
      }
  }
  let conn = SqliteConnection::connect(database_url).await?;
  let (sender, receiver) = mpsc::channel::<Option<Vec<AnimeMetadata>>>(4);
  let mut downloader = Downloader::new(sender);
  let mut db_loader = DbLoader::new(receiver, conn);

  let downloader_handle = task::spawn(async move {
    downloader.download().await
  });

  let loader_handle = task::spawn(async move {
    db_loader.load().await
  });

  let results = tokio::try_join!(
    downloader_handle,
    loader_handle,
  );
  if results.is_err() {
    eprintln!("{:?}", results);
  }

  Ok(())

}
