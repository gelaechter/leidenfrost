use crate::backend::{
    api::{
        endpoint_api::{ApiContract, SearchResult},
        jellyfin::{data::{BaseItemDtoQueryResult, BaseItemKind}, normalize::Normalize},
    },
    db::{
        models::{Album, Artist, Disc, Genre, Playlist, Track},
        sqlite::DB,
    },
};
use std::collections::HashSet;

use crate::backend::api::{
    endpoint_api::UserPasswordAuth,
    jellyfin::errors::{ApiError, RequestError},
};

use iced::task::{Sipper, sipper};
use reqwest::header::ACCEPT;
use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::HeaderMap;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use url::Url;

#[tokio::test]
pub async fn test_create_jf_api() {
    let jf = JellyfinApi::auth_user_password(
        Url::parse("http://***REMOVED***").unwrap(),
        "***REMOVED***".to_string(),
        "***REMOVED***".to_string(),
    )
    .await;

    let db = DB::open().await.unwrap();

    jf.index_data(&db).await;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JellyfinApi {
    url: Url,
    user_id: String,
    folder_id: Option<String>,
    #[serde(with = "http_serde::header_map")]
    headers: HeaderMap,
    #[serde(skip)]
    client: reqwest::Client,
}

#[allow(refining_impl_trait)]
impl UserPasswordAuth for JellyfinApi {
    async fn auth_user_password(url: Url, username: String, password: String) -> JellyfinApi {
        let client = reqwest::Client::new();
        let auth_header = r#"MediaBrowser Client="Randale", Device="Android", DeviceId="aaihsgdfoiuaghsdfuawbfayuzgbsdfufzg", Version="0.0.1""#;

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert(AUTHORIZATION, auth_header.parse().unwrap());

        let authentication_response = client
            .post(url.join("/Users/AuthenticateByName").unwrap())
            .headers(headers.clone())
            .json(&json!({
                "Username": &username,
                "Pw": &password,
            }))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .text()
            .await
            .unwrap();

        let response_json: Value = serde_json::from_str(&authentication_response)
            .map_err(|e| {
                RequestError::SerdeError(
                    "Couldn't parse server response while authorizing".to_owned(),
                    e.to_string(),
                    authentication_response.clone(),
                )
            })
            .unwrap();

        let access_token = response_json["AccessToken"]
            .as_str()
            .ok_or(RequestError::SerdeError(
                "The field AccessToken doesn't exist".to_owned(),
                "Jellyfin authentication response didn't contain a token".to_owned(),
                authentication_response.clone(),
            ))
            .unwrap();

        let user_id = response_json["User"]["Id"]
            .as_str()
            .ok_or(RequestError::SerdeError(
                "The path User.Id doesn't exist".to_owned(),
                "Jellyfin authentication response didn't contain the user id".to_owned(),
                authentication_response,
            ))
            .unwrap();

        headers.insert(
            AUTHORIZATION,
            format!(r#"{auth_header}, Token="{access_token}""#)
                .parse()
                .unwrap(),
        );

        JellyfinApi {
            url,
            user_id: user_id.to_owned(),
            folder_id: None,
            headers,
            client,
        }
    }
}

impl JellyfinApi {
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Make a request to a relative path e.g. /Users/.../Songs
    ///
    /// query parameters should be a slice of key value pairs: &[("key", "val")]
    ///
    /// The response is parsed into a json serde value.
    pub async fn request<T>(
        &self,
        relative_path: String,
        query_parameters: &T,
    ) -> Result<Value, ApiError>
    where
        T: Serialize + ?Sized,
    {
        let response = self
            .client
            .get(self.url.join(&relative_path).unwrap())
            .headers(self.headers.clone())
            .query(query_parameters)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let json: Value = serde_json::from_str(&response).map_err(|e| {
            RequestError::SerdeError(
                e.to_string(),
                format!(
                    "Couldn't parse server response into JSON from request url {}",
                    relative_path
                ),
                response.clone(),
            )
        })?;

        Ok(json)
    }

    pub async fn index_data(&self, db: &DatabaseConnection) {
        let mut sipper = self.fetch_items().pin();

        while let Some(progress) = sipper.sip().await {
            println!("{progress}%");
        }

        let types = sipper.await;

        dbg!(types);
    }

    pub fn fetch_items(&self) -> impl Sipper<HashSet<BaseItemKind>, u64> + '_ {
        sipper(move |mut sender| async move {
            let mut types = HashSet::new();

            let json = self.request(
                    "/Users/07bd2df2d1bf4b51b33082acf300aeba/Items".to_owned(),
                    &json!({
                        "Recursive": true,
                        "IncludeItemTypes": BaseItemKind::Audio,
                        "Limit": 1
                    }),
                )
                .await
                .unwrap();

            dbg!(&json);

            let query_result: BaseItemDtoQueryResult = serde_json::from_value(json)
            .unwrap();

            dbg!(&query_result);

            return types;

            const STEP_SIZE: u64 = 100;
            for index in 0..((query_result.total_record_count as u64).div_ceil(STEP_SIZE)) {
                let response = self
                    .request(
                        "/Users/07bd2df2d1bf4b51b33082acf300aeba/Items".to_owned(),
                        &[
                            ("Recursive", "true"),
                            ("Limit", &STEP_SIZE.to_string()),
                            ("IncludeItemTypes", "Playlist"),
                            ("StartIndex", &format!("{}", index * STEP_SIZE)),
                        ],
                    )
                    .await
                    .unwrap();

                let list: BaseItemDtoQueryResult = serde_json::from_value(response).unwrap();
                dbg!(&list);
                for item in list.items {
                    if let Some(type_) = item.type_ {
                        types.insert(type_);
                    }
                }
                sender.send(index).await;
            }

            types
        })
    }
}

impl ApiContract for JellyfinApi {
    async fn get_track(&self, song_id: String) -> Track {
        todo!()
    }

    async fn get_tracks(&self) -> color_eyre::Result<Vec<Track>> {
        let query_result: BaseItemDtoQueryResult = serde_json::from_value(
            self.request(
                "/Users/07bd2df2d1bf4b51b33082acf300aeba/Items".to_owned(),
                &json!({
                    "Recursive": true,
                    "IncludeItemTypes": BaseItemKind::Audio,
                    "Limit": 100
                }),
            )
            .await
            .unwrap(),
        )
        .unwrap();

        let tracks = query_result
            .items
            .into_iter()
            .map(|item| self.normalize(item))
            .collect();

        Ok(tracks)
    }

    async fn get_songs_from_album(&self, album_id: String) -> color_eyre::Result<Vec<Disc>> {
        todo!()
    }

    async fn get_album(&self, album_id: String) -> color_eyre::Result<Album> {
        todo!()
    }

    async fn get_albums(&self) -> color_eyre::Result<Vec<Album>> {
        todo!()
    }

    async fn get_albums_from_artist(&self, artist_id: String) -> color_eyre::Result<Vec<Album>> {
        todo!()
    }

    async fn get_artist(&self, artist_id: String) -> color_eyre::Result<Artist> {
        todo!()
    }

    async fn get_artists(&self) -> color_eyre::Result<Vec<Artist>> {
        todo!()
    }

    async fn get_genres(&self) -> color_eyre::Result<Vec<Genre>> {
        todo!()
    }

    async fn get_playlists(&self) -> color_eyre::Result<Vec<Playlist>> {
        todo!()
    }

    async fn search(&self, search_term: String) -> color_eyre::Result<Vec<SearchResult>> {
        todo!()
    }
}
