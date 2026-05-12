use crate::backend::{
    api::{
        endpoint_api::{ApiContract, SearchResult},
        jellyfin::{
            data::{BaseItemDtoQueryResult, BaseItemKind},
            normalize::Normalize,
        },
    },
    data_view::{AlbumView, PlaylistView, TrackView},
    db::models::{Album, Artist, Disc, Genre},
};

use crate::backend::api::{
    endpoint_api::UserPasswordAuth,
    jellyfin::errors::{ApiError, RequestError},
};

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

impl UserPasswordAuth for JellyfinApi {
    async fn auth_user_password(url: Url, username: String, password: String) -> JellyfinApi {
        let client = reqwest::Client::new();
        let auth_header = r#"MediaBrowser Client="Randale",Device="Android",DeviceId="aaihsgdfoiuaghsdfuawbfayuzgbsdfufzg",Version="0.0.1""#;

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
        relative_path: &str,
        query_parameters: &T,
    ) -> Result<Value, ApiError>
    where
        T: Serialize + ?Sized,
    {
        let response = self
            .client
            .get(self.url.join(relative_path).unwrap())
            .headers(self.headers.clone())
            // See here for query parameters
            // https://typescript-sdk.jellyfin.org/interfaces/generated-client.ItemsApiGetItemsRequest.html
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
                    "Couldn't parse server response into JSON from request url {relative_path}"
                ),
                response.clone(),
            )
        })?;

        Ok(json)
    }

    pub async fn index_data(&self, db: DatabaseConnection) {}
}

impl ApiContract for JellyfinApi {
    async fn get_track(&self, song_id: String) -> TrackView {
        todo!()
    }

    async fn get_tracks(&self) -> color_eyre::Result<Vec<TrackView>> {
        let user_id = &self.user_id;

        let query_result = self
            .request(
                &format!("/Users/{user_id}/Items"),
                &json!({
                    "Fields": "Genres",
                    "Recursive": true,
                    "IncludeItemTypes": BaseItemKind::Audio,
                    "Limit": 100
                }),
            )
            .await
            .unwrap();

        let query_result: BaseItemDtoQueryResult = serde_json::from_value(query_result).unwrap();

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

    async fn get_album(&self, album_id: String) -> color_eyre::Result<AlbumView> {
        todo!()
    }

    /// This returns flat [`AlbumView`] values, these do not contain tracks
    async fn get_albums(&self) -> color_eyre::Result<Vec<AlbumView>> {
        let user_id = &self.user_id;

        let query_result = self
            .request(
                &format!("/Users/{user_id}/Items"),
                &json!({
                    "Fields": "Genres",
                    "Recursive": true,
                    "IncludeItemTypes": BaseItemKind::MusicAlbum,
                    "Limit": 100
                }),
            )
            .await
            .unwrap();

        let query_result: BaseItemDtoQueryResult = serde_json::from_value(query_result).unwrap();

        let tracks = query_result
            .items
            .into_iter()
            .map(|item| self.normalize(item))
            .collect();

        Ok(tracks)
    }

    async fn get_albums_from_artist(
        &self,
        artist_id: String,
    ) -> color_eyre::Result<Vec<AlbumView>> {
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

    async fn search(&self, search_term: String) -> color_eyre::Result<Vec<SearchResult>> {
        todo!()
    }

    async fn get_playlists(&self) -> color_eyre::Result<Vec<PlaylistView>> {
        let query_result: BaseItemDtoQueryResult = serde_json::from_value(
            self.request(
                "/Users/07bd2df2d1bf4b51b33082acf300aeba/Items",
                &json!({
                    "Recursive": true,
                    "IncludeItemTypes": BaseItemKind::Playlist,
                    // "Limit": 100
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

    async fn get_playlist(
        &self,
        playlist_id: String,
    ) -> color_eyre::Result<crate::backend::data_view::PlaylistView> {
        todo!()
    }

    async fn get_playlist_tracks(&self, playlist_id: String) -> color_eyre::Result<Vec<TrackView>> {
        todo!()
    }
}
