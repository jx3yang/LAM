use lam::downloader::Downloader;
use lam::db_loader::DbLoader;
use sqlx::{migrate::MigrateDatabase, Connection, Error, Sqlite, SqliteConnection};

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
  let mut conn = SqliteConnection::connect(database_url).await?;
  let result = Downloader::fire_request(1, "ANIME", 2024)
      .await
      .map(Downloader::handle_response);
  match result {
      Ok((media, has_next_page)) => {
          // println!("{:?}", media);

          // println!("{}", has_next_page);
          DbLoader::create_table_if_not_exists(&mut conn).await?;
          println!("finished creating");
          DbLoader::load_metadata(&mut conn, media).await?;
          println!("finished loading");
          Ok(())
      },
      Err(_) => {
        println!("Error");
        Ok(())
      },
  }
}
