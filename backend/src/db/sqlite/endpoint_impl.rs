use std::{
    collections::{HashMap, HashSet},
    result,
};

use sqlx::{AssertSqlSafe, QueryBuilder};
use sqlx::{prelude::FromRow, sqlite::SqliteRow};

use super::sorting::*;
use super::{AlbumView, ArtistView, GenreView, PlaylistView, SearchResult, TrackView};
use crate::{
    api::{
        endpoint_api::{
            Capabilities, GetAlbumsParams, GetArtistsParams, GetGenresParams, GetPlaylistsParams,
            GetTracksParams, MusicEndpoint, SearchParams,
        },
        jellyfin::errors::ApiError,
    },
    data_view::{RelatedArtist, RelatedGenre},
    db::{
        models::{
            album::Album,
            artist::Artist,
            genre::Genre,
            playlist::Playlist,
            track::{Lyrics, Track},
        },
        sqlite::EndpointDB,
    },
};

pub type Result<T> = result::Result<T, ApiError>;

/// Ids per `IN (...)` clause
const IN_CHUNK: usize = 900;

const TRACK_SELECT: &str =
    "SELECT t.*, a.title AS album_name FROM track t LEFT JOIN album a ON a.id = t.album_id";

#[derive(FromRow)]
struct TrackRow {
    #[sqlx(flatten)]
    track: Track,
    album_name: Option<String>,
}

impl EndpointDB {
    async fn fetch_in<T>(
        &self,
        prefix: &'static str,
        ids: &[String],
        suffix: &'static str,
    ) -> Result<Vec<T>>
    where
        T: for<'r> FromRow<'r, SqliteRow> + Send + Unpin,
    {
        let mut out = Vec::new();
        for chunk in ids.chunks(IN_CHUNK) {
            let mut qb = QueryBuilder::new(prefix);
            let mut sep = qb.separated(", ");
            for id in chunk {
                sep.push_bind(id);
            }
            qb.push(suffix);
            out.extend(qb.build_query_as::<T>().fetch_all(&self.db).await?);
        }
        Ok(out)
    }

    async fn related_artists_by_track(
        &self,
        ids: &[String],
    ) -> Result<HashMap<String, Vec<RelatedArtist>>> {
        let rows = self
            .fetch_in::<(String, String, Option<String>)>(
                "SELECT at.track_id, ar.id, ar.name FROM artist_tracks at \
             JOIN artist ar ON ar.id = at.artist_id WHERE at.track_id IN (",
                ids,
                ")",
            )
            .await?;
        Ok(rows
            .into_iter()
            .fold(HashMap::new(), |mut m, (tid, id, name)| {
                m.entry(tid).or_default().push(RelatedArtist { id, name });
                m
            }))
    }

    async fn related_genres_by_track(
        &self,
        ids: &[String],
    ) -> Result<HashMap<String, Vec<RelatedGenre>>> {
        let rows = self
            .fetch_in::<(String, String, Option<String>)>(
                "SELECT tg.track_id, g.id, g.name FROM track_genres tg \
             JOIN genre g ON g.id = tg.genre_id WHERE tg.track_id IN (",
                ids,
                ")",
            )
            .await?;
        Ok(rows
            .into_iter()
            .fold(HashMap::new(), |mut m, (tid, id, name)| {
                m.entry(tid).or_default().push(RelatedGenre { id, name });
                m
            }))
    }

    async fn track_views(&self, rows: Vec<TrackRow>) -> Result<Vec<TrackView>> {
        let ids: Vec<String> = rows.iter().map(|r| r.track.id.clone()).collect();
        let artists = self.related_artists_by_track(&ids).await?;
        let genres = self.related_genres_by_track(&ids).await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let id = r.track.id.clone();
                TrackView {
                    track: r.track,
                    album_name: r.album_name,
                    artists: artists.get(&id).cloned().unwrap_or_default(),
                    genres: genres.get(&id).cloned().unwrap_or_default(),
                }
            })
            .collect())
    }

    async fn album_artists_by_album(
        &self,
        ids: &[String],
    ) -> Result<HashMap<String, Vec<RelatedArtist>>> {
        let rows = self
            .fetch_in::<(String, String, Option<String>)>(
                "SELECT aa.album_id, ar.id, ar.name FROM artist_albums aa \
             JOIN artist ar ON ar.id = aa.artist_id WHERE aa.album_id IN (",
                ids,
                ")",
            )
            .await?;
        Ok(rows
            .into_iter()
            .fold(HashMap::new(), |mut m, (aid, id, name)| {
                m.entry(aid).or_default().push(RelatedArtist { id, name });
                m
            }))
    }

    async fn genres_by_album(&self, ids: &[String]) -> Result<HashMap<String, Vec<RelatedGenre>>> {
        let rows = self
            .fetch_in::<(String, String, Option<String>)>(
                "SELECT t.album_id, g.id, g.name FROM track_genres tg \
             JOIN track t ON t.id = tg.track_id \
             JOIN genre g ON g.id = tg.genre_id WHERE t.album_id IN (",
                ids,
                ")",
            )
            .await?;
        let mut out: HashMap<String, Vec<RelatedGenre>> = HashMap::new();
        let mut seen: HashSet<(String, String)> = HashSet::new();
        for (aid, id, name) in rows {
            if seen.insert((aid.clone(), id.clone())) {
                out.entry(aid).or_default().push(RelatedGenre { id, name });
            }
        }
        Ok(out)
    }

    async fn album_durations(&self, ids: &[String]) -> Result<HashMap<String, Option<i64>>> {
        let rows = self
            .fetch_in::<(String, Option<i64>)>(
                "SELECT album_id, SUM(duration) FROM track WHERE album_id IN (",
                ids,
                ") GROUP BY album_id",
            )
            .await?;
        Ok(rows.into_iter().collect())
    }

    async fn artist_durations(&self, ids: &[String]) -> Result<HashMap<String, Option<i64>>> {
        let rows = self
            .fetch_in::<(String, Option<i64>)>(
                "SELECT at.artist_id, SUM(t.duration) FROM artist_tracks at \
             JOIN track t ON t.id = at.track_id WHERE at.artist_id IN (",
                ids,
                ") GROUP BY at.artist_id",
            )
            .await?;
        Ok(rows.into_iter().collect())
    }

    async fn playlist_stats(&self, ids: &[String]) -> Result<HashMap<String, (i64, Option<i64>)>> {
        let rows = self
            .fetch_in::<(String, i64, Option<i64>)>(
                "SELECT pt.playlist_id, COUNT(*), SUM(t.duration) FROM playlist_tracks pt \
             JOIN track t ON t.id = pt.track_id WHERE pt.playlist_id IN (",
                ids,
                ") GROUP BY pt.playlist_id",
            )
            .await?;
        Ok(rows.into_iter().map(|(id, c, d)| (id, (c, d))).collect())
    }

    async fn album_views(&self, albums: Vec<Album>) -> Result<Vec<AlbumView>> {
        let ids: Vec<String> = albums.iter().map(|a| a.id.clone()).collect();
        let artists = self.album_artists_by_album(&ids).await?;
        let genres = self.genres_by_album(&ids).await?;
        let durations = self.album_durations(&ids).await?;
        Ok(albums
            .into_iter()
            .map(|album| {
                let id = album.id.clone();
                AlbumView {
                    album_artists: artists.get(&id).cloned().unwrap_or_default(),
                    genres: genres.get(&id).cloned().unwrap_or_default(),
                    duration: durations.get(&id).copied().flatten(),
                    album,
                }
            })
            .collect())
    }

    async fn artist_views(&self, artists: Vec<Artist>) -> Result<Vec<ArtistView>> {
        let ids: Vec<String> = artists.iter().map(|a| a.id.clone()).collect();
        let durations = self.artist_durations(&ids).await?;
        Ok(artists
            .into_iter()
            .map(|artist| ArtistView {
                duration: durations.get(&artist.id).copied().flatten(),
                artist,
            })
            .collect())
    }

    async fn playlist_views(&self, playlists: Vec<Playlist>) -> Result<Vec<PlaylistView>> {
        let ids: Vec<String> = playlists.iter().map(|p| p.id.clone()).collect();
        let stats = self.playlist_stats(&ids).await?;
        Ok(playlists
            .into_iter()
            .map(|playlist| {
                let (count, duration) = stats.get(&playlist.id).copied().unwrap_or((0, None));
                PlaylistView {
                    playlist,
                    track_count: Some(count),
                    duration,
                }
            })
            .collect())
    }
}

#[async_trait::async_trait]
impl MusicEndpoint for EndpointDB {
    fn capabilities(&self) -> Capabilities {
        // DB-backed cache: map to whatever your Capabilities type expresses
        crate::api::endpoint_api::FULL_CAPABILITIES.clone()
    }

    async fn get_track(&self, track_id: String) -> Result<TrackView> {
        let sql = format!("{TRACK_SELECT} WHERE t.id = ?");
        let mut views = self
            .track_views(
                sqlx::query_as::<_, TrackRow>(AssertSqlSafe(sql))
                    .bind(track_id)
                    .fetch_all(&self.db)
                    .await?,
            )
            .await?;
        views.pop().ok_or(ApiError::DataNotFound)
    }

    async fn get_tracks(&self, params: GetTracksParams) -> Result<Vec<TrackView>> {
        let (page, binds) = pagination(&params.pagination);
        let order = track_order_by(params.sorting.unwrap_or_default());
        let sql = format!("{TRACK_SELECT} {order}{page}");
        let mut q = sqlx::query_as::<_, TrackRow>(AssertSqlSafe(sql));
        if let Some((l, o)) = binds {
            q = q.bind(l).bind(o);
        }
        self.track_views(q.fetch_all(&self.db).await?).await
    }

    async fn get_album(&self, album_id: String) -> Result<AlbumView> {
        let mut views = self
            .album_views(
                sqlx::query_as::<_, Album>("SELECT a.* FROM album a WHERE a.id = ?")
                    .bind(album_id)
                    .fetch_all(&self.db)
                    .await?,
            )
            .await?;
        views.pop().ok_or(ApiError::DataNotFound)
    }

    async fn get_albums(&self, params: GetAlbumsParams) -> Result<Vec<AlbumView>> {
        let (page, binds) = pagination(&params.pagination);
        let order = album_order_by(params.sorting.unwrap_or_default());
        let sql = format!("SELECT a.* FROM album a {}{}", order, page);
        let mut q = sqlx::query_as::<_, Album>(AssertSqlSafe(sql));
        if let Some((l, o)) = binds {
            q = q.bind(l).bind(o);
        }
        self.album_views(q.fetch_all(&self.db).await?).await
    }

    async fn get_album_tracks(
        &self,
        album_id: String,
        params: GetTracksParams,
    ) -> Result<Vec<TrackView>> {
        let (page, binds) = pagination(&params.pagination);
        let order = params
            .sorting
            .map(track_order_by)
            .unwrap_or_else(|| "ORDER BY t.disc_number, t.track_number, t.id".into());
        let sql = format!("{TRACK_SELECT} WHERE t.album_id = ? {order}{page}");
        let mut q = sqlx::query_as::<_, TrackRow>(AssertSqlSafe(sql)).bind(album_id);
        if let Some((l, o)) = binds {
            q = q.bind(l).bind(o);
        }
        self.track_views(q.fetch_all(&self.db).await?).await
    }

    async fn get_artist_albums(
        &self,
        artist_id: String,
        params: GetAlbumsParams,
    ) -> Result<Vec<AlbumView>> {
        let (page, binds) = pagination(&params.pagination);
        let order: String = album_order_by(params.sorting.unwrap_or_default());
        let sql = format!(
            "SELECT a.* FROM artist_albums aa JOIN album a ON a.id = aa.album_id \
             WHERE aa.artist_id = ? {}{}",
            order, page
        );
        let mut q = sqlx::query_as::<_, Album>(AssertSqlSafe(sql)).bind(artist_id);
        if let Some((l, o)) = binds {
            q = q.bind(l).bind(o);
        }
        self.album_views(q.fetch_all(&self.db).await?).await
    }

    async fn get_artist(&self, artist_id: String) -> Result<ArtistView> {
        let mut views = self
            .artist_views(
                sqlx::query_as::<_, Artist>("SELECT ar.* FROM artist ar WHERE ar.id = ?")
                    .bind(artist_id)
                    .fetch_all(&self.db)
                    .await?,
            )
            .await?;
        views.pop().ok_or(ApiError::DataNotFound)
    }

    async fn get_artists(&self, params: GetArtistsParams) -> Result<Vec<ArtistView>> {
        let (page, binds) = pagination(&params.pagination);
        let order = artist_order_by(params.sorting.unwrap_or_default());
        let sql = format!("SELECT ar.* FROM artist ar {}{}", order, page);
        let mut q = sqlx::query_as::<_, Artist>(AssertSqlSafe(sql));
        if let Some((l, o)) = binds {
            q = q.bind(l).bind(o);
        }
        self.artist_views(q.fetch_all(&self.db).await?).await
    }

    async fn get_album_artists(&self, params: GetArtistsParams) -> Result<Vec<ArtistView>> {
        let (page, binds) = pagination(&params.pagination);
        let order = artist_order_by(params.sorting.unwrap_or_default());
        let sql = format!(
            "SELECT ar.* FROM artist ar WHERE EXISTS \
             (SELECT 1 FROM artist_albums aa WHERE aa.artist_id = ar.id) {}{}",
            order, page
        );
        let mut q = sqlx::query_as::<_, Artist>(AssertSqlSafe(sql));
        if let Some((l, o)) = binds {
            q = q.bind(l).bind(o);
        }
        self.artist_views(q.fetch_all(&self.db).await?).await
    }

    async fn get_genres(&self, params: GetGenresParams) -> Result<Vec<GenreView>> {
        let (page, binds) = pagination(&params.pagination);
        let order = genre_order_by(params.sorting.unwrap_or_default());
        let sql = format!("SELECT g.* FROM genre g {}{}", order, page);
        let mut q = sqlx::query_as::<_, Genre>(AssertSqlSafe(sql));
        if let Some((l, o)) = binds {
            q = q.bind(l).bind(o);
        }
        Ok(q.fetch_all(&self.db)
            .await?
            .into_iter()
            .map(|genre| GenreView { genre })
            .collect())
    }

    async fn get_playlists(&self, params: GetPlaylistsParams) -> Result<Vec<PlaylistView>> {
        let (page, binds) = pagination(&params.pagination);
        let order = playlist_order_by(params.sorting.unwrap_or_default());
        let sql = format!("SELECT p.* FROM playlist p {}{}", order, page);
        let mut q = sqlx::query_as::<_, Playlist>(AssertSqlSafe(sql));
        if let Some((l, o)) = binds {
            q = q.bind(l).bind(o);
        }
        self.playlist_views(q.fetch_all(&self.db).await?).await
    }

    async fn get_playlist(&self, playlist_id: String) -> Result<PlaylistView> {
        let mut views = self
            .playlist_views(
                sqlx::query_as::<_, Playlist>("SELECT p.* FROM playlist p WHERE p.id = ?")
                    .bind(playlist_id)
                    .fetch_all(&self.db)
                    .await?,
            )
            .await?;
        views.pop().ok_or(ApiError::DataNotFound)
    }

    async fn get_playlist_tracks(
        &self,
        playlist_id: String,
        params: GetTracksParams,
    ) -> Result<Vec<TrackView>> {
        let (page, binds) = pagination(&params.pagination);
        let order = params
            .sorting
            .map(track_order_by)
            .unwrap_or_else(|| "ORDER BY pt.position, t.id".into());
        let sql = format!(
            "SELECT t.*, a.title AS album_name FROM playlist_tracks pt \
             JOIN track t ON t.id = pt.track_id \
             LEFT JOIN album a ON a.id = t.album_id \
             WHERE pt.playlist_id = ? {order}{page}"
        );
        let mut q = sqlx::query_as::<_, TrackRow>(AssertSqlSafe(sql)).bind(playlist_id);
        if let Some((l, o)) = binds {
            q = q.bind(l).bind(o);
        }
        self.track_views(q.fetch_all(&self.db).await?).await
    }

    async fn search(&self, term: String, params: SearchParams) -> Result<Vec<SearchResult>> {
        let pat = like_pattern(&term);
        let mut out: Vec<SearchResult> = Vec::new();

        // Tracks
        let sql = format!(
            "{TRACK_SELECT} WHERE t.title LIKE ? ESCAPE '\\' ORDER BY t.title COLLATE NOCASE"
        );
        let rows = sqlx::query_as::<_, TrackRow>(AssertSqlSafe(sql))
            .bind(&pat)
            .fetch_all(&self.db)
            .await?;
        out.extend(
            self.track_views(rows)
                .await?
                .into_iter()
                .map(SearchResult::Track),
        );

        // Albums
        let sql = "SELECT a.* FROM album a WHERE a.title LIKE ? ESCAPE '\\' ORDER BY a.title COLLATE NOCASE";
        let rows = sqlx::query_as::<_, Album>(sql)
            .bind(&pat)
            .fetch_all(&self.db)
            .await?;
        out.extend(
            self.album_views(rows)
                .await?
                .into_iter()
                .map(SearchResult::Album),
        );

        // Artists
        let sql = "SELECT ar.* FROM artist ar WHERE ar.name LIKE ? ESCAPE '\\' ORDER BY ar.name COLLATE NOCASE";
        let rows = sqlx::query_as::<_, Artist>(sql)
            .bind(&pat)
            .fetch_all(&self.db)
            .await?;
        out.extend(
            self.artist_views(rows)
                .await?
                .into_iter()
                .map(SearchResult::Artist),
        );

        // Playlists
        let sql = "SELECT p.* FROM playlist p WHERE p.name LIKE ? ESCAPE '\\' ORDER BY p.name COLLATE NOCASE";
        let rows = sqlx::query_as::<_, Playlist>(sql)
            .bind(&pat)
            .fetch_all(&self.db)
            .await?;
        out.extend(
            self.playlist_views(rows)
                .await?
                .into_iter()
                .map(SearchResult::Playlist),
        );

        // Genres
        let sql = "SELECT g.* FROM genre g WHERE g.name LIKE ? ESCAPE '\\' ORDER BY g.name COLLATE NOCASE";
        let rows = sqlx::query_as::<_, Genre>(sql)
            .bind(&pat)
            .fetch_all(&self.db)
            .await?;
        out.extend(
            rows.into_iter()
                .map(|genre| SearchResult::Genre(GenreView { genre })),
        );

        // Lyrics: LIKE over the JSON text column, matched line extracted in Rust
        let sql = format!("{TRACK_SELECT} WHERE t.lyrics LIKE ? ESCAPE '\\'");
        let views = self
            .track_views(
                sqlx::query_as::<_, TrackRow>(AssertSqlSafe(sql))
                    .bind(&pat)
                    .fetch_all(&self.db)
                    .await?,
            )
            .await?;
        let needle = term.to_lowercase();
        for v in views {
            let matched = match &v.track.lyrics {
                Some(Lyrics::Plain(lines)) => lines
                    .iter()
                    .find(|l| l.to_lowercase().contains(&needle))
                    .cloned(),
                Some(Lyrics::Synchronized(lines)) => lines
                    .values()
                    .find(|l| l.to_lowercase().contains(&needle))
                    .cloned(),
                None => continue,
            };
            out.push(SearchResult::LyricMatch {
                matched_lyric: matched,
                track_view: v,
            });
        }

        let (skip, take) = match &params.pagination {
            Some(p) => (p.start_page.saturating_mul(p.limit), p.limit),
            None => (0, u64::MAX),
        };
        Ok(out
            .into_iter()
            .skip(usize::try_from(skip).unwrap_or(usize::MAX))
            .take(usize::try_from(take).unwrap_or(usize::MAX))
            .collect())
    }
}
