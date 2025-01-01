use serde_json::json;
use reqwest::Client;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use crate::types::AnimeMetadata;

const QUERY: &str = "
query (
  $page: Int
  $id: Int
  $type: MediaType
  $isAdult: Boolean
  $search: String
  $format: [MediaFormat]
  $status: MediaStatus
  $countryOfOrigin: CountryCode
  $source: MediaSource
  $season: MediaSeason
  $seasonYear: Int
  $year: String
  $onList: Boolean
  $yearLesser: FuzzyDateInt
  $yearGreater: FuzzyDateInt
  $episodeLesser: Int
  $episodeGreater: Int
  $durationLesser: Int
  $durationGreater: Int
  $chapterLesser: Int
  $chapterGreater: Int
  $volumeLesser: Int
  $volumeGreater: Int
  $licensedBy: [Int]
  $isLicensed: Boolean
  $genres: [String]
  $excludedGenres: [String]
  $tags: [String]
  $excludedTags: [String]
  $minimumTagRank: Int
  $sort: [MediaSort] = [ID]
) {
  Page(page: $page, perPage: 50) {
    pageInfo {
      hasNextPage
    }
    media(
      id: $id
      type: $type
      season: $season
      format_in: $format
      status: $status
      countryOfOrigin: $countryOfOrigin
      source: $source
      search: $search
      onList: $onList
      seasonYear: $seasonYear
      startDate_like: $year
      startDate_lesser: $yearLesser
      startDate_greater: $yearGreater
      episodes_lesser: $episodeLesser
      episodes_greater: $episodeGreater
      duration_lesser: $durationLesser
      duration_greater: $durationGreater
      chapters_lesser: $chapterLesser
      chapters_greater: $chapterGreater
      volumes_lesser: $volumeLesser
      volumes_greater: $volumeGreater
      licensedById_in: $licensedBy
      isLicensed: $isLicensed
      genre_in: $genres
      genre_not_in: $excludedGenres
      tag_in: $tags
      tag_not_in: $excludedTags
      minimumTagRank: $minimumTagRank
      sort: $sort
      isAdult: $isAdult
    ) {
      id
      title {
        romaji
      }
      season
      seasonYear
      description
      popularity
      meanScore
    }
  }
}
";

pub struct Downloader {
    sender: mpsc::Sender<Option<Vec<AnimeMetadata>>>,
}

impl Downloader {
    pub fn new(sender: mpsc::Sender<Option<Vec<AnimeMetadata>>>) -> Self {
        Self { sender }
    }

    pub async fn download(&mut self) -> Result<bool, reqwest::Error> {
        let mut page = 1;
        let mut season_year = 2025;
        let media_type = "ANIME";
        let mut has_next_page = true;

        while season_year > 1999 || has_next_page {
            let response = Downloader::fire_request(page, media_type, season_year)
                .await
                .map(Downloader::handle_response);
            match response {
                Ok((media, new_has_next_page)) => {
                    println!("Finished downloading season {} page {}", season_year, page);
                    has_next_page = new_has_next_page;
                    let send_response = self.sender.send(Some(media)).await;
                    if send_response.is_err() {
                        println!("{:?}", send_response.unwrap_err());
                        eprintln!("Failed to push data to the queue");
                        return Ok(false);
                    }
                    println!("Sent season {} page {}", season_year, page);

                    page += 1;
                    if !has_next_page {
                        season_year -= 1;
                        page = 1;
                    }
                    sleep(Duration::from_secs(1)).await;
                },
                Err(e) => return Err(e),
            }
        }

        Ok(true)
    }

    pub async fn fire_request(
        page: i32,
        media_type: &str,
        season_year: i32,
    ) -> Result<serde_json::Value, reqwest::Error> {
        let client = Client::new();
        let json = json!({"query": QUERY, "variables": {"page": page, "type": media_type, "seasonYear": season_year}});
        let response = client.post("https://graphql.anilist.co/")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(json.to_string())
            .send()
            .await
            .unwrap();
    
        // println!("{}", response.headers()["x-ratelimit-remaining"].to_str().unwrap());
        response.text()
            .await
            .map(|response| serde_json::from_str(&response).unwrap())
    }

    pub fn handle_response(
        response: serde_json::Value,
    ) -> (Vec<AnimeMetadata>, bool) {
        let data = &response["data"]["Page"];
        let has_next_page = &data["pageInfo"]["hasNextPage"].as_bool().unwrap_or(false);
        let media = data["media"].as_array().map(|arr| {
            let anime_metadata_vec: Vec<AnimeMetadata> = arr
                .iter()
                .filter_map(|val| {
                    let anime_metadata: Option<AnimeMetadata> = match serde_json::from_value(val.clone()) {
                        Ok(anime_metadata) => Some(anime_metadata),
                        Err(_) => None,
                    };
                    anime_metadata
                })
                .collect();
            anime_metadata_vec
        }).unwrap_or_default();
        if media.is_empty() {
            println!("{:#}", response);
            println!("Empty list, something is wrong!");
        }
        (media, *has_next_page)
    }
}
