use crate::backend::{
    api::{
        endpoint_api::{
            self, GetAlbumsParams, GetArtistsParams, GetGenresParams, GetPlaylistParams,
            GetTracksParams, MusicEndpoint, Pagination, SearchParams, SearchResult, Sort,
            UserPasswordAuth,
        },
        jellyfin::{
            data::{BaseItemDtoQueryResult, BaseItemKind, ItemSortBy},
            errors::RequestError,
            normalize::Normalize,
        },
    },
    data_view::{AlbumView, PlaylistView, TrackView},
    db::models::{Artist, Disc, Genre},
};

use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
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

pub type Result<T> = endpoint_api::Result<T>;

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
    pub async fn query_items<T, S>(
        &self,
        relative_path: &str,
        item_kind: BaseItemKind,
        pagination: Option<Pagination>,
        sorting: Option<Sort<S>>,
        additional_parameters: &T,
    ) -> Result<BaseItemDtoQueryResult>
    where
        T: Serialize + ?Sized,
        S: Into<ItemSortBy> + std::fmt::Debug + Clone + Default,
    {
        let mut request_builder = self
            .client
            .get(self.url.join(relative_path)?)
            .headers(self.headers.clone());

        // Add pagination
        if let Some(Pagination { start, limit }) = pagination {
            request_builder = request_builder.query(&json!({
                "Start": start,
                "Limit": limit
            }));
        }

        // Add sorting
        if let Some(Sort { order, by }) = sorting {
            request_builder = request_builder.query(&json!({
                "SortOrder": order,
                "SortBy": by.into()
            }));
        }

        let response = request_builder
            // See here for query parameters
            // https://typescript-sdk.jellyfin.org/interfaces/generated-client.ItemsApiGetItemsRequest.html
            .query(&json!({
                "Recursive": true,
                "IncludeItemTypes": item_kind,
            }))
            .query(additional_parameters)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let json: BaseItemDtoQueryResult = serde_json::from_str(&response).map_err(|e| {
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

impl MusicEndpoint for JellyfinApi {
    async fn get_track(&self, song_id: String) -> Result<TrackView> {
        todo!()
    }

    async fn get_tracks(&self, params: GetTracksParams) -> Result<Vec<TrackView>> {
        let user_id = &self.user_id;
        let GetTracksParams {
            pagination,
            sorting,
        } = params;

        let query_result = self
            .query_items(
                &format!("/Users/{user_id}/Items"),
                BaseItemKind::Audio,
                pagination,
                sorting,
                &json!({
                    "Fields": "Genres",
                }),
            )
            .await
            .unwrap();

        let tracks = query_result
            .items
            .into_iter()
            .map(|item| self.normalize(item))
            .collect();

        Ok(tracks)
    }

    async fn get_album_tracks(
        &self,
        album_id: String,
        params: GetTracksParams,
    ) -> Result<Vec<Disc>> {
        todo!()
    }

    async fn get_album(&self, album_id: String) -> Result<AlbumView> {
        todo!()
    }

    /// This returns flat [`AlbumView`] values, these do not contain tracks
    async fn get_albums(&self, params: GetAlbumsParams) -> Result<Vec<AlbumView>> {
        let user_id = &self.user_id;
        let GetAlbumsParams {
            pagination,
            sorting,
        } = params;

        let query_result = self
            .query_items(
                &format!("/Users/{user_id}/Items"),
                BaseItemKind::MusicAlbum,
                pagination,
                sorting,
                &json!({
                    "Fields": "Genres",
                }),
            )
            .await
            .unwrap();

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
        params: GetAlbumsParams,
    ) -> Result<Vec<AlbumView>> {
        todo!()
    }

    async fn get_artist(&self, artist_id: String) -> Result<Artist> {
        todo!()
    }

    async fn get_artists(&self, params: GetArtistsParams) -> Result<Vec<Artist>> {
        todo!()
    }

    async fn get_genres(&self, params: GetGenresParams) -> Result<Vec<Genre>> {
        todo!()
    }

    async fn search(&self, search_term: String, params: SearchParams) -> Result<Vec<SearchResult>> {
        todo!()
    }

    async fn get_playlists(&self, params: GetPlaylistParams) -> Result<Vec<PlaylistView>> {
        let user_id = &self.user_id;
        let GetPlaylistParams {
            pagination,
            sorting,
        } = params;

        let query_result = self
            .query_items(
                &format!("/Users/{user_id}/Items"),
                BaseItemKind::Playlist,
                pagination,
                sorting,
                &Value::Null,
            )
            .await
            .unwrap();

        let tracks = query_result
            .items
            .into_iter()
            .map(|item| self.normalize(item))
            .collect();

        Ok(tracks)
    }

    fn capabilities() -> endpoint_api::Capabilities {
        todo!()
    }

    async fn get_playlist(&self, playlist_id: String) -> endpoint_api::Result<PlaylistView> {
        todo!()
    }

    async fn get_playlist_tracks(
        &self,
        playlist_id: String,
        params: GetPlaylistParams,
    ) -> endpoint_api::Result<Vec<TrackView>> {
        todo!()
    }
}
