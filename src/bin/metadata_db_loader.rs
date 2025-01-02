use lam::{downloader::Downloader, types::AnimeMetadata};
use lam::db_loader::{MetadataLoader, DbLoader};
use lam::constants::DATABASE_URL;
use sqlx::{migrate::MigrateDatabase, Connection, Error, Sqlite, SqliteConnection};
use tokio::sync::mpsc;
use tokio::task;

#[tokio::main]
async fn main() -> Result<(), Error> {
    if !Sqlite::database_exists(DATABASE_URL).await.unwrap_or(false) {
      println!("Creating database {}", DATABASE_URL);
      match Sqlite::create_database(DATABASE_URL).await {
          Ok(_) => println!("Create db success"),
          Err(error) => panic!("error: {}", error),
      }
  }
  let conn = SqliteConnection::connect(DATABASE_URL).await?;
  let (sender, receiver) = mpsc::channel::<Option<Vec<AnimeMetadata>>>(4);
  let mut downloader = Downloader::new(sender);
  let mut db_loader = MetadataLoader::new(receiver, conn);

  let downloader_handle = task::spawn(async move {
    downloader.download().await
  });

  let loader_handle = task::spawn(async move {
    db_loader.start_load_job().await
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
