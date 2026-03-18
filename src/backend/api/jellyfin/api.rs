use crate::backend::{api::endpoint_api::{ApiContract, MultiUsers}, data::User};

use super::data::JFAlbum;
use super::data::JFAlbumArtist;
use super::data::JFGenre;
use super::data::JFPlaylist;
use super::data::JFSong;
use super::data::JFUser;
use super::normalize::Normalize;

use reqwest::header::ACCEPT;
use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::HeaderMap;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;

use std::any::type_name;
use std::collections::HashMap;
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

impl JellyfinApi {
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Make a request to a relative path e.g. /Users/.../Songs
    ///
    /// query parameters are anything that reqwest can serialize into a parameter
    ///
    /// The response is parsed into a json serde value.
    pub async fn request<T>(
        &self,
        relative_path: String,
        query_parameters: &T,
    ) -> color_eyre::Result<Value>
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

        let json: Value = serde_json::from_str(&response).with_context(|e| {
            format!(
                "Couldn't parse server response into JSON from request url {}",
                relative_path
            )
        })?;

        Ok(json)
    }

    /// Make a request to a relative path e.g. /Users/.../Songs
    ///
    /// The response is expected to be a Vec<JF> in Json format.
    ///
    /// The Vec<JF> is the normalized into a Vec<T> using the Normalize<JF, T> trait.
    pub async fn get_and_parse<JF, T, S>(
        &self,
        relative_path: String,
        query_parameters: &S,
    ) -> color_eyre::Result<Vec<T>>
    where
        JF: for<'de> Deserialize<'de>,
        Self: Normalize<JF, T>,
        S: Serialize + ?Sized,
    {
        let mut json = self.request(relative_path, query_parameters).await?;

        let json = json["Items"].take();
        let items = serde_json::from_value::<Vec<JF>>(json.clone()).map_err(|e| {
            RequestError::SerdeError(
                e.to_string(),
                format!(
                    "Couldn't parse {} JSON into {} structure",
                    type_name::<JF>(),
                    type_name::<T>()
                ),
                json.to_string().to_owned(),
            )
        })?;

        let items: Vec<T> = items.into_iter().map(|i| self.normalize(i)).collect();

        Ok(items)
    }
}

#[allow(refining_impl_trait)]
impl UrlUserPasswordAuth for JellyfinApi {
    async fn auth_username_password(
        url: Url,
        username: String,
        password: String,
    ) -> Result<JellyfinApi, ApiError> {
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
            client,
        })
    }
}

impl MultiUsers for JellyfinApi {
    /// Fetches all available users
    async fn get_users(url: Url) -> Result<Vec<User>, ApiError> {
        let jf = JellyfinApi {
            url,
            user_id: "".to_owned(),
            folder_id: None,
            headers: HeaderMap::new(),
            client: reqwest::Client::new(),
        };

        let items = jf
            .get_and_parse::<JFUser, User>("/Users".to_owned(), &())
            .await?;

        Ok(items)
    }
}

impl ApiContract for JellyfinApi {
    /// Fetches all available users
    /// Fetches a specific song
    ///
    /// # Arguments
    ///
    /// * `song_id` - The id of the song
    async fn get_song(&self, song_id: String) -> Result<Song, ApiError> {
        let user_id = &self.user_id;

        let json: Value = self
            .request(format!("/Users/{user_id}/Items/{song_id}"), &())
            .await?;
        let song = self.normalize(serde_json::from_value::<JFSong>(json.clone()).map_err(|e| {
            RequestError::SerdeError(
                e.to_string(),
                "Couldn't parse server response into JSON while fetching an album".to_owned(),
                json.to_string(),
            )
        })?);

        Ok(song)
    }

    /// Fetches all songs
    ///
    /// # Arguments
    ///
    /// * `start` - The start index from which to get the songs
    /// * `limit` - The amount of songs to fetch
    async fn get_songs(&self, start: i32, limit: i32) -> Result<Vec<Song>, ApiError> {
        self.search_songs(String::new(), start, limit).await
    }

    async fn get_songs_from_album(
        &self,
        album_id: String,
    ) -> Result<Vec<Vec<Song>>, ApiError> {
        let mut disc_map: HashMap<i32, Vec<Song>> = HashMap::new();

        let json: Value = self
            .request(
                format!("/Users/{}/Items", self.user_id),
                &[
                    ("fields", "Path"),
                    ("startIndex", &start.to_string()),
                    ("includeItemTypes", "Audio"),
                    ("albumIds", &album_id),
                    ("sortOrder", "Ascending"),
                    ("recursive", "true"),
                    ("limit", &limit.to_string()),
                ],
            )
            .await?;

        let mut songs: Vec<Song> = json["Items"]
            .as_array()
            .expect("Couldn't parse song list response")
            .iter()
            .map(|v| self.normalize(serde_json::from_value::<JFSong>(v.to_owned()).unwrap()))
            .collect();

        // Sort by discs and track numbers
        songs.sort_unstable_by_key(|song| (song.disc_number, song.track_number));

        // Order into discs
        for ele in songs {
            disc_map.entry(ele.disc_number).or_default().push(ele);
        }

        // Create list of lists
        let discs = disc_map.drain().map(|v| v.1).collect();

        Ok(discs)
    }

    /// Fetches a specific album
    ///
    /// # Arguments
    ///
    /// * `album_id` - The start index from which to get the songs
    async fn get_album(&self, album_id: String) -> Result<Album, ApiError> {
        let json: Value = self
            .request(format!("/Users/{}/Items/{}", self.user_id, album_id), &())
            .await?;

        let album = self.normalize(serde_json::from_value::<JFAlbum>(json.clone()).map_err(
            |e| {
                RequestError::SerdeError(
                    e.to_string(),
                    "Couldn't parse server response into Album from JSON".to_owned(),
                    json.to_string(),
                )
            },
        )?);

        Ok(album)
    }

    async fn get_albums(&self, start: i32, limit: i32) -> Result<Vec<Album>, ApiError> {
        self.search_albums(String::new(), start, limit).await
    }

    async fn get_albums_from_artist(
        &self,
        artist_id: String,
        start: i32,
        limit: i32,
    ) -> Result<Vec<Album>, ApiError> {
        let albums: Vec<Album> = self
            .get_and_parse::<JFAlbum, Album>(
                format!("/Users/{}/Items", self.user_id),
                &[
                    ("fields", "Path"),
                    ("startIndex", &start.to_string()),
                    ("includeItemTypes", "MusicAlbum"),
                    ("artistIds", &artist_id),
                    ("recursive", "true"),
                    ("enableImageTypes", "Primary, Backdrop, Logo"),
                    ("limit", &limit.to_string()),
                ],
            )
            .await?;

        Ok(albums)
    }

    async fn get_artist(&self, artist_id: String) -> Result<Artist, ApiError> {
        let json: Value = self
            .request(format!("/Users/{}/Items/{}", self.user_id, artist_id), &())
            .await?;

        let artist = self.normalize(
            serde_json::from_value::<JFAlbumArtist>(json.clone()).map_err(|e| {
                RequestError::SerdeError(
                    e.to_string(),
                    "Couldn't parse server response into Album from JSON".to_string(),
                    json.to_string(),
                )
            })?,
        );

        Ok(artist)
    }

    async fn get_artists(&self, start: i32, limit: i32) -> Result<Vec<Artist>, ApiError> {
        self.search_artists(String::new(), start, limit).await
    }

    async fn search_songs(
        &self,
        search_term: String,
        start: i32,
        limit: i32,
    ) -> Result<Vec<Song>, ApiError> {
        let songs = self
            .get_and_parse::<JFSong, Song>(
                format!("/Users/{}/Items", self.user_id),
                &[
                    ("fields", "Path"),
                    ("startIndex", &start.to_string()),
                    ("includeItemTypes", "Audio"),
                    ("sortOrder", "Ascending"),
                    ("sortBy", "Album"),
                    ("recursive", "true"),
                    ("searchTerm", &search_term),
                    ("enableImageTypes", "Primary, Backdrop, Logo"),
                    ("limit", &limit.to_string()),
                ],
            )
            .await?;

        Ok(songs)
    }

    async fn search_albums(
        &self,
        search_term: String,
        start: i32,
        limit: i32,
    ) -> Result<Vec<Album>, ApiError> {
        let albums = self
            .get_and_parse::<JFAlbum, Album>(
                format!("/Users/{}/Items", self.user_id),
                &[
                    ("fields", "Path"),
                    ("startIndex", &start.to_string()),
                    ("includeItemTypes", "MusicAlbum"),
                    ("searchTerm", &search_term),
                    ("recursive", "true"),
                    ("enableImageTypes", "Primary, Backdrop, Logo"),
                    ("limit", &limit.to_string()),
                ],
            )
            .await?;

        Ok(albums)
    }

    async fn search_artists(
        &self,
        search_term: String,
        start: i32,
        limit: i32,
    ) -> Result<Vec<Artist>, ApiError> {
        let artists = self
            .get_and_parse::<JFAlbumArtist, Artist>(
                "/Artists".to_string(),
                &[
                    ("fields", "Path"),
                    ("startIndex", &start.to_string()),
                    ("includeItemTypes", "MusicAlbum"),
                    ("searchTerm", &search_term),
                    ("recursive", "true"),
                    ("enableImageTypes", "Primary, Backdrop, Logo"),
                    ("limit", &limit.to_string()),
                ],
            )
            .await?;

        Ok(artists)
    }

    async fn get_genres(&self, start: i32, limit: i32) -> Result<Vec<Genre>, ApiError> {
        let items = self
            .client
            .get(self.url.join("/Genres").unwrap())
            .headers(self.headers.clone())
            .query(&[
                ("recursive", "true"),
                ("startIndex", &start.to_string()),
                ("parentId", &self.folder_id.clone().unwrap_or_default()),
                ("enableImageTypes", "Primary, Backdrop, Logo"),
                ("limit", &limit.to_string()),
            ])
            .send()
            .await?
            .text()
            .await?;

        let json: Value = serde_json::from_str(&items).unwrap();

        let genres: Vec<Genre> = json["Items"]
            .as_array()
            .expect("Couldn't parse genre list response")
            .iter()
            .cloned()
            .map(|v| serde_json::from_value::<JFGenre>(v).unwrap())
            .filter(|genre| genre.r#type == "MusicGenre")
            .map(|g| self.normalize(g))
            .collect();

        Ok(genres)
    }

    /// Fetches a list of genres
    /// 
    /// # Arguments
    /// * `start` - The start index from which to get the genres
    /// * `limit` - The amount of genres to fetch
    async fn get_playlists(
        &self,
        start: i32,
        limit: i32,
    ) -> Result<Vec<Playlist>, common::errors::ApiError> {
        let items = self
            .client
            .get(
                self.url
                    .join(&format!("/Users/{}/Items", self.user_id))
                    .unwrap(),
            )
            .headers(self.headers.clone())
            .query(&[
                ("fields", "Path"),
                ("startIndex", &start.to_string()),
                ("includeItemTypes", "Playlist"),
                ("recursive", "true"),
                ("enableImageTypes", "Primary, Backdrop, Logo"),
                ("limit", &limit.to_string()),
            ])
            .send()
            .await?
            .text()
            .await?;

        let json: Value = serde_json::from_str(&items).unwrap();

        let albums = json["Items"]
            .as_array()
            .expect("Couldn't parse song list response")
            .iter()
            .map(|v| self.normalize(serde_json::from_value::<JFPlaylist>(v.to_owned()).unwrap()))
            .collect();

        Ok(albums)
    }
}
