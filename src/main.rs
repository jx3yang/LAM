use serde_json::json;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde;

// Query to use in request
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

#[derive(Deserialize, Serialize, Debug)]
struct Title {
    romaji: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct AnimeMetadata {
    id: i32,
    title: Title,
    season: String,

    #[serde(rename = "seasonYear")]
    season_year: i32,
    description: String,
    popularity: i32,

    #[serde(rename = "meanScore")]
    mean_score: i32,
}

async fn fire_request(
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

fn handle_response(
    response: serde_json::Value,
) -> (Vec<AnimeMetadata>, bool) {
    let data = &response["data"]["Page"];
    let has_next_page = &data["pageInfo"]["hasNextPage"].as_bool().unwrap_or(false);
    let media = data["media"].as_array().map(|arr| {
        let anime_metadata_vec: Vec<AnimeMetadata> = arr
            .into_iter()
            .map(|val| {
                let anime_metadata: Result<AnimeMetadata, _> = serde_json::from_value(val.clone());
                anime_metadata.unwrap()
            })
            .collect();
        anime_metadata_vec
    }).unwrap_or_else(|| Vec::new());
    (media, *has_next_page)
}

#[tokio::main]
async fn main() {
    let result = fire_request(1, "ANIME", 2024).await.map(handle_response);
    match result {
        Ok((media, has_next_page)) => {
            println!("{:?}", media);
            println!("{}", has_next_page);
        },
        Err(_) => println!("Error"),
    }
}
