use std::sync::LazyLock;

use crate::backend::{
    api::{
        endpoint_api::{
            self, ArtistAlbums, GetAlbumsParams, GetArtistsParams, GetGenresParams,
            GetPlaylistsParams, GetTracksParams, MusicEndpoint, Pagination, SearchParams,
            SearchResult, Sort, UserPasswordAuth,
        },
        jellyfin::{
            data::{BaseItemDto, BaseItemDtoQueryResult, BaseItemKind, ItemSortBy},
            errors::RequestError,
            normalize::Normalize,
        },
    },
    data_view::{AlbumView, ArtistView, GenreView, PlaylistView, TrackView},
};

use async_trait::async_trait;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap};
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
}

pub type Result<T> = endpoint_api::Result<T>;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

impl UserPasswordAuth for JellyfinApi {
    async fn auth_user_password(
        url: Url,
        username: String,
        password: String,
    ) -> Result<JellyfinApi> {
        log::debug!("JellyfinApi::auth_user_password: {url:?}, {username:?}, {password:?}");

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
            .await?
            .error_for_status()?
            .text()
            .await?;

        let response_json: Value = serde_json::from_str(&authentication_response).map_err(|e| {
            RequestError::SerdeError(
                "Couldn't parse server response while authorizing".to_owned(),
                e.to_string(),
                authentication_response.clone(),
            )
        })?;

        let access_token =
            response_json["AccessToken"]
                .as_str()
                .ok_or(RequestError::SerdeError(
                    "The field AccessToken doesn't exist".to_owned(),
                    "Jellyfin authentication response didn't contain a token".to_owned(),
                    authentication_response.clone(),
                ))?;

        let user_id = response_json["User"]["Id"]
            .as_str()
            .ok_or(RequestError::SerdeError(
                "The path User.Id doesn't exist".to_owned(),
                "Jellyfin authentication response didn't contain the user id".to_owned(),
                authentication_response,
            ))?;

        headers.insert(
            AUTHORIZATION,
            format!(r#"{auth_header}, Token="{access_token}""#)
                .parse()
                .unwrap(),
        );

        Ok(JellyfinApi {
            url,
            user_id: user_id.to_owned(),
            folder_id: None,
            headers,
        })
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
    /// The response is parsed and normalized.
    pub async fn query_item<P, T>(&self, relative_path: &str, additional_params: &P) -> Result<T>
    where
        P: Serialize + ?Sized,
        Self: Normalize<BaseItemDto, T>,
    {
        dbg!(&self.headers);

        let request_builder = CLIENT
            .get(self.url.join(relative_path)?)
            .headers(self.headers.clone());

        let response = request_builder
            .query(additional_params)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let item: BaseItemDto = serde_json::from_str(&response).map_err(|e| {
            RequestError::SerdeError(
                e.to_string(),
                format!(
                    "Couldn't parse server response into JSON from request url {relative_path}"
                ),
                response.clone(),
            )
        })?;

        Ok(self.normalize(item))
    }

    /// Make a request to a relative path e.g. /Users/.../Songs
    ///
    /// query parameters should be a slice of key value pairs: &[("key", "val")]
    ///
    /// The response is parsed and normalized.
    pub async fn query_items<S, P, T>(
        &self,
        relative_path: &str,
        item_kind: BaseItemKind,
        pagination: Option<Pagination>,
        sorting: Option<Sort<S>>,
        additional_params: &P,
    ) -> Result<Vec<T>>
    where
        // Query paramters
        P: Serialize + ?Sized,
        S: Into<ItemSortBy> + std::fmt::Debug + Clone + Default,
        Self: Normalize<BaseItemDto, T>,
    {
        dbg!(relative_path, &self.headers);

        let mut request_builder = CLIENT
            .get(self.url.join(relative_path)?)
            .headers(self.headers.clone());

        // Add pagination
        if let Some(Pagination {
            start_page: start,
            limit,
        }) = pagination
        {
            request_builder = request_builder.query(&json!({
                // Jellyfin uses a record-unit start, meaning
                // `start=50` indicates we start with the 50tieth record, not page
                "Start": start * limit,
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
            .query(additional_params)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let query_result: BaseItemDtoQueryResult =
            serde_json::from_str(&response).map_err(|e| {
                RequestError::SerdeError(
                    e.to_string(),
                    format!(
                        "Couldn't parse server response into JSON from request url {relative_path}"
                    ),
                    response.clone(),
                )
            })?;

        let items = query_result
            .items
            .into_iter()
            .map(|item| self.normalize(item))
            .collect();

        Ok(items)
    }
}

#[async_trait]
impl MusicEndpoint for JellyfinApi {
    fn get_id(&self) -> String {
        format!("Jellyfin_{}_{}", self.url, self.user_id)
    }

    async fn get_track(&self, track_id: String) -> Result<TrackView> {
        let user_id = &self.user_id;

        self.query_item(&format!("/Users/{user_id}/Items/{track_id}"), &Value::Null)
            .await
    }

    async fn get_tracks(&self, params: GetTracksParams) -> Result<Vec<TrackView>> {
        log::debug!("get_tracks: {params:?}");

        let user_id = &self.user_id;
        let GetTracksParams {
            pagination,
            sorting,
        } = params;

        self.query_items(
            &format!("/Users/{user_id}/Items"),
            BaseItemKind::Audio,
            pagination,
            sorting,
            // https://typescript-sdk.jellyfin.org/interfaces/generated-client.ItemsApiGetItemsRequest.html#fields
            &json!({
                "Fields": "Genres",
            }),
        )
        .await
    }

    async fn get_album_tracks(
        &self,
        album_id: String,
        params: GetTracksParams,
    ) -> Result<Vec<TrackView>> {
        let user_id = &self.user_id;
        let GetTracksParams {
            pagination,
            sorting,
        } = params;

        let tracks: Vec<TrackView> = self
            .query_items(
                &format!("/Users/{user_id}/Items"),
                BaseItemKind::Audio,
                pagination,
                sorting,
                &json!({
                    "Fields": "Genres",
                    "ParentId": album_id,
                }),
            )
            .await?;

        Ok(tracks)
    }

    async fn get_album(&self, album_id: String) -> Result<AlbumView> {
        let user_id = &self.user_id;

        self.query_item(&format!("/Users/{user_id}/Items/{album_id}"), &Value::Null)
            .await
    }

    /// This returns flat [`AlbumView`] values, these do not contain tracks
    async fn get_albums(&self, params: GetAlbumsParams) -> Result<Vec<AlbumView>> {
        let user_id = &self.user_id;
        let GetAlbumsParams {
            pagination,
            sorting,
        } = params;

        self.query_items(
            &format!("/Users/{user_id}/Items"),
            BaseItemKind::MusicAlbum,
            pagination,
            sorting,
            // https://typescript-sdk.jellyfin.org/interfaces/generated-client.ItemsApiGetItemsRequest.html#fields
            &json!({
                "Fields": "Genres",
            }),
        )
        .await
    }

    async fn get_artist_albums(
        &self,
        artist_id: String,
        params: GetAlbumsParams,
    ) -> Result<ArtistAlbums> {
        log::debug!("get_artist_albums: {artist_id:?} {params:?}");

        let user_id = &self.user_id;
        let GetAlbumsParams {
            pagination,
            sorting,
        } = params;

        let appears_on = self
            .query_items(
                &format!("/Users/{user_id}/Items"),
                BaseItemKind::MusicAlbum,
                pagination.clone(),
                sorting.clone(),
                &json!({
                    "ContributingArtistIds": artist_id,
                }),
            )
            .await?;

        let created = self
            .query_items(
                &format!("/Users/{user_id}/Items"),
                BaseItemKind::MusicAlbum,
                pagination,
                sorting,
                &json!({
                    "AlbumArtistIds": artist_id,
                }),
            )
            .await?;

        Ok(ArtistAlbums {
            appears_on,
            created,
        })
    }

    async fn get_artist(&self, artist_id: String) -> Result<ArtistView> {
        let user_id = &self.user_id;

        self.query_item(&format!("/Users/{user_id}/Items/{artist_id}"), &Value::Null)
            .await
    }

    async fn get_artists(&self, params: GetArtistsParams) -> Result<Vec<ArtistView>> {
        let user_id = &self.user_id;
        let GetArtistsParams {
            pagination,
            sorting,
        } = params;

        self.query_items(
            "/Artists",
            BaseItemKind::MusicArtist,
            pagination,
            sorting,
            &json!({
                "UserId": user_id
            }),
        )
        .await
    }

    async fn get_genres(&self, params: GetGenresParams) -> Result<Vec<GenreView>> {
        let user_id = &self.user_id;
        let GetGenresParams {
            pagination,
            sorting,
        } = params;

        // https://typescript-sdk.jellyfin.org/interfaces/generated-client.GenresApiGetGenresRequest.html
        self.query_items(
            "/Genres",
            BaseItemKind::MusicGenre,
            pagination,
            sorting,
            &json!({
                "UserId": user_id
            }),
        )
        .await
    }

    async fn search(&self, search_term: String, params: SearchParams) -> Result<Vec<SearchResult>> {
        todo!()
    }

    async fn get_playlists(&self, params: GetPlaylistsParams) -> Result<Vec<PlaylistView>> {
        let user_id = &self.user_id;
        let GetPlaylistsParams {
            pagination,
            sorting,
        } = params;

        self.query_items(
            &format!("/Users/{user_id}/Items"),
            BaseItemKind::Playlist,
            pagination,
            sorting,
            &Value::Null,
        )
        .await
    }

    fn capabilities(&self) -> endpoint_api::Capabilities {
        todo!()
    }

    async fn get_playlist(&self, playlist_id: String) -> endpoint_api::Result<PlaylistView> {
        let user_id = &self.user_id;

        self.query_item(
            &format!("/Users/{user_id}/Items/{playlist_id}"),
            &Value::Null,
        )
        .await
    }

    async fn get_playlist_tracks(
        &self,
        playlist_id: String,
        params: GetPlaylistsParams,
    ) -> endpoint_api::Result<Vec<TrackView>> {
        let GetPlaylistsParams {
            pagination,
            sorting,
        } = params;

        self.query_items(
            &format!("/Playlist/{playlist_id}/Items"),
            BaseItemKind::Audio,
            pagination,
            sorting,
            &json!({
                "Fields": "Genres",
            }),
        )
        .await
    }
}
