use std::collections::{BTreeSet, HashSet};

use async_trait::async_trait;
use sqlx::{Pool, QueryBuilder, Sqlite};

use crate::api::{
    endpoint_api::{
        GetAlbumsParams, GetArtistsParams, GetGenresParams, GetPlaylistsParams, GetTracksParams,
        MusicEndpoint, Pagination,
    },
    jellyfin::errors::ApiError,
};

use super::{AlbumView, ArtistView, GenreView, PlaylistView, TrackView};

/// Rows per remote fetch
const PAGE_SIZE: u64 = 100;

/// Rows per INSERT batch (tracks: 23 binds/row -> 23k binds)
/// SQLite's default maximum is 32766
const ROW_CHUNK: usize = 1000;

/// Ids per `IN (...)` clause
const IN_CHUNK: usize = 900;

/// Bulk upsert:
///
/// `INSERT INTO <table> (<key>, <cols..>) VALUES (..), (..)
///  ON CONFLICT(<key>) DO UPDATE SET <col> = excluded.<col>, ...`
///
/// The bind body must push exactly one bind per column, key first, in order.
/// Columns listed in `update` are refreshed on conflict; anything omitted
/// (e.g. `date_added`) keeps its originally inserted value.
macro_rules! upsert {
    ($db:expr, $table:literal, key: $key:literal, cols: [$($col:literal),+ $(,)?],
     update: [$($ucol:literal),+ $(,)?],
     rows: $rows:expr, bind: |$b:ident, $row:ident| $body:block) => {{
        let rows: &[_] = $rows;
        if !rows.is_empty() {
            let mut qb = QueryBuilder::<Sqlite>::new("INSERT INTO ");
            qb.push($table).push(" (").push($key);
            $( qb.push(", ").push($col); )+
            qb.push(")");
            // push_values passes (&mut QueryBuilder, &row) — builder FIRST,
            // matching both the pattern above and the call-site closures.
            qb.push_values(rows, |mut $b, $row| $body);
            qb.push(" ON CONFLICT(").push($key).push(") DO UPDATE SET ");
            let mut sets = qb.separated(", ");
            $( sets.push(concat!($ucol, " = excluded.", $ucol)); )+
            qb.build().execute($db).await?;
        }
    }};
}

type Result<T> = std::result::Result<T, ApiError>;

/// Returns the subset of `ids` that currently exist in `table`.
async fn existing_ids(
    db: &Pool<Sqlite>,
    table: &'static str,
    ids: &[&String],
) -> Result<HashSet<String>> {
    let mut existing = HashSet::new();
    for chunk in ids.chunks(IN_CHUNK) {
        let mut qb = QueryBuilder::new("SELECT id FROM ");
        qb.push(table).push(" WHERE id IN (");
        let mut sep = qb.separated(", ");
        for id in chunk {
            sep.push_bind(*id);
        }
        qb.push(")");
        for (id,) in qb.build_query_as::<(String,)>().fetch_all(db).await? {
            existing.insert(id);
        }
    }
    Ok(existing)
}

/// Inserts `(parent_id, child_id)` junction rows. Rows whose `parent_id` has no
/// row in `parent_table` are skipped (e.g. an album credits an artist the
/// endpoint never returned via `get_artists`). Idempotent via DO NOTHING.
async fn insert_junctions(
    db: &Pool<Sqlite>,
    table: &'static str,
    parent_col: &'static str,
    child_col: &'static str,
    parent_table: &'static str,
    rows: &[(String, String)],
) -> Result<()> {
    if rows.is_empty() {
        return Ok(());
    }

    let parent_ids: Vec<&String> = rows
        .iter()
        .map(|(p, _)| p)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let existing = existing_ids(db, parent_table, &parent_ids).await?;

    let valid: Vec<&(String, String)> = rows.iter().filter(|(p, _)| existing.contains(p)).collect();
    log::debug!(
        "Inserting {} rows into {table} (skipping {} with missing parent)",
        valid.len(),
        rows.len() - valid.len()
    );

    for chunk in valid.chunks(ROW_CHUNK) {
        let mut qb = QueryBuilder::new("INSERT INTO ");
        qb.push(table)
            .push(" (")
            .push(parent_col)
            .push(", ")
            .push(child_col)
            .push(") ");
        qb.push_values(chunk, |mut b, (parent_id, child_id)| {
            b.push_bind(parent_id).push_bind(child_id);
        });
        qb.push(" ON CONFLICT DO NOTHING");
        qb.build().execute(db).await?;
    }
    Ok(())
}

/// Recreates one playlist's entries: remote order is authoritative (positions,
/// removals, reorders, duplicate entries). Entries whose track is not in the
/// local library are skipped.
async fn replace_playlist_tracks(
    db: &Pool<Sqlite>,
    playlist_id: &str,
    entries: &[(i64, String)],
) -> Result<()> {
    let mut tx = db.begin().await?;

    sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ?")
        .bind(playlist_id)
        .execute(&mut *tx)
        .await?;

    let track_ids: Vec<&String> = entries
        .iter()
        .map(|(_, t)| t)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let existing = existing_ids(db, "track", &track_ids).await?;

    let valid: Vec<&(i64, String)> = entries
        .iter()
        .filter(|(_, t)| existing.contains(t))
        .collect();
    if valid.len() != entries.len() {
        log::debug!(
            "Playlist {playlist_id}: skipping {} entries with unindexed tracks",
            entries.len() - valid.len()
        );
    }

    for chunk in valid.chunks(ROW_CHUNK) {
        let mut qb =
            QueryBuilder::new("INSERT INTO playlist_tracks (playlist_id, position, track_id) ");
        qb.push_values(chunk, |mut b, (position, track_id)| {
            b.push_bind(playlist_id)
                .push_bind(position)
                .push_bind(track_id);
        });
        qb.push(" ON CONFLICT DO NOTHING");
        qb.build().execute(&mut *tx).await?;
    }

    tx.commit().await?;
    Ok(())
}

async fn upsert_artists(db: &Pool<Sqlite>, endpoint_id: &str, views: &[ArtistView]) -> Result<()> {
    for chunk in views.chunks(ROW_CHUNK) {
        log::debug!("Inserting {} artists", chunk.len());
        upsert!(
            db, "artist",
            key: "id",
        cols: ["endpoint_id", "name", "biography", "image_url", "image_blur_hash",
               "background_image_url", "background_image_blurhash",
               "play_count", "user_favorite", "user_rating", "date_added"],
        update: ["endpoint_id", "name", "biography", "image_url", "image_blur_hash",
                 "background_image_url", "background_image_blurhash",
                 "play_count", "user_favorite", "user_rating"],
        rows: chunk,
        bind: |b, view| {
            b.push_bind(&view.artist.id)
                .push_bind(endpoint_id)
                .push_bind(&view.artist.name)
                .push_bind(&view.artist.biography)
                .push_bind(&view.artist.image_url)
                .push_bind(&view.artist.image_blur_hash)
                .push_bind(&view.artist.background_image_url)
                .push_bind(&view.artist.background_image_blurhash)
                .push_bind(view.artist.play_count)
                .push_bind(view.artist.user_favorite)
                .push_bind(view.artist.user_rating)
                .push_bind(view.artist.date_added);
        });
    }
    Ok(())
}

async fn upsert_albums(db: &Pool<Sqlite>, endpoint_id: &str, views: &[AlbumView]) -> Result<()> {
    for chunk in views.chunks(ROW_CHUNK) {
        log::debug!("Inserting {} albums", chunk.len());
        upsert!(db, "album", key: "id",
        cols: ["endpoint_id", "release_date", "title", "image_url", "image_blur_hash",
               "user_favorite", "date_added"],
        update: ["endpoint_id", "release_date", "title", "image_url", "image_blur_hash",
                 "user_favorite"],
        rows: chunk,
        bind: |b, view| {
            b.push_bind(&view.album.id)
                .push_bind(endpoint_id)
                .push_bind(view.album.release_date)
                .push_bind(&view.album.title)
                .push_bind(&view.album.image_url)
                .push_bind(&view.album.image_blur_hash)
                .push_bind(view.album.user_favorite)
                .push_bind(view.album.date_added);
        });
    }
    Ok(())
}

async fn upsert_genres(db: &Pool<Sqlite>, endpoint_id: &str, views: &[GenreView]) -> Result<()> {
    for chunk in views.chunks(ROW_CHUNK) {
        log::debug!("Inserting {} genres", chunk.len());
        upsert!(db, "genre", key: "id",
        cols: ["endpoint_id", "name", "image_url", "image_blur_hash"],
        update: ["endpoint_id", "name", "image_url", "image_blur_hash"],
        rows: chunk,
        bind: |b, view| {
            b.push_bind(&view.genre.id)
                .push_bind(endpoint_id)
                .push_bind(&view.genre.name)
                .push_bind(&view.genre.image_url)
                .push_bind(&view.genre.image_blur_hash);
        });
    }
    Ok(())
}

async fn upsert_tracks(
    db: &Pool<Sqlite>,
    endpoint_id: &str,
    views: &mut [TrackView],
) -> Result<()> {
    // Tracks may reference albums that were never indexed; spec says album_id
    // is optional, so clear those instead of failing the FK constraint.
    let album_ids: Vec<&String> = views
        .iter()
        .filter_map(|v| v.track.album_id.as_ref())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let known_albums = existing_ids(db, "album", &album_ids).await?;

    for v in &mut *views {
        if let Some(album_id) = &v.track.album_id
            && !known_albums.contains(album_id)
        {
            log::debug!(
                "Track {}: album {album_id} not indexed, clearing album_id",
                v.track.id
            );
            v.track.album_id = None;
        }
    }

    for chunk in views.chunks(ROW_CHUNK) {
        log::debug!("Inserting {} tracks", chunk.len());
        upsert!(db, "track", key: "id",
        cols: ["endpoint_id", "album_id", "disc_number", "bit_rate", "bpm", "channels",
               "container", "duration", "image_url", "image_blur_hash", "last_played_at",
               "lyrics", "title", "file_path", "play_count", "release_date", "file_size",
               "stream_url", "track_number", "user_favorite", "date_added"],
        update: ["endpoint_id", "album_id", "disc_number", "bit_rate", "bpm", "channels",
                 "container", "duration", "image_url", "image_blur_hash", "last_played_at",
                 "lyrics", "title", "file_path", "play_count", "release_date", "file_size",
                 "stream_url", "track_number", "user_favorite"],
        rows: chunk,
        bind: |b, view| {
            b.push_bind(&view.track.id)
                .push_bind(endpoint_id)
                .push_bind(&view.track.album_id)
                .push_bind(view.track.disc_number)
                .push_bind(view.track.bit_rate)
                .push_bind(view.track.bpm)
                .push_bind(view.track.channels)
                .push_bind(&view.track.container)
                .push_bind(view.track.duration)
                .push_bind(&view.track.image_url)
                .push_bind(&view.track.image_blur_hash)
                .push_bind(view.track.last_played_at)
                .push_bind(&view.track.lyrics)
                .push_bind(&view.track.title)
                .push_bind(&view.track.file_path)
                .push_bind(view.track.play_count)
                .push_bind(view.track.release_date)
                .push_bind(view.track.file_size)
                .push_bind(&view.track.stream_url)
                .push_bind(view.track.track_number)
                .push_bind(view.track.user_favorite)
                .push_bind(view.track.date_added);
        });
    }
    Ok(())
}

async fn upsert_playlists(
    db: &Pool<Sqlite>,
    endpoint_id: &str,
    views: &[PlaylistView],
) -> Result<()> {
    for chunk in views.chunks(ROW_CHUNK) {
        log::debug!("Inserting {} playlists", chunk.len());
        upsert!(db, "playlist", key: "id",
        cols: ["endpoint_id", "name", "image_url", "image_blur_hash", "user_favorite"],
        update: ["endpoint_id", "name", "image_url", "image_blur_hash", "user_favorite"],
        rows: chunk,
        bind: |b, view| {
            b.push_bind(&view.playlist.id)
                .push_bind(endpoint_id)
                .push_bind(&view.playlist.name)
                .push_bind(&view.playlist.image_url)
                .push_bind(&view.playlist.image_blur_hash)
                .push_bind(view.playlist.user_favorite);
        });
    }
    Ok(())
}

#[async_trait]
pub trait IndexableEndpoint: MusicEndpoint {
    /// Provides a unique ID for this API
    /// This is used to match indexed data in the database to the respective API
    fn get_id(&self) -> String;

    /// Naive implementation which just requests all data from the endpoint
    async fn index_data(&self, db: &Pool<Sqlite>) -> Result<()> {
        let endpoint_id = self.get_id();
        let capabilities = self.capabilities();
        let paginated = capabilities.pagination;

        log::info!("Indexing endpoint {endpoint_id}");

        sqlx::query("INSERT INTO endpoint (id) VALUES (?) ON CONFLICT(id) DO NOTHING")
            .bind(&endpoint_id)
            .execute(db)
            .await?;

        // Artists
        let mut page = 0u64;
        loop {
            let views = self
                .get_artists(GetArtistsParams {
                    pagination: paginated.then_some(Pagination {
                        start_page: page,
                        limit: PAGE_SIZE,
                    }),
                    sorting: None,
                })
                .await?;
            if views.is_empty() {
                break;
            }
            page += 1;

            upsert_artists(db, &endpoint_id, &views).await?;
            if !paginated {
                break;
            }
        }

        // Album artists (AFAIK Jellyfin specific) — merged into `artist`
        if capabilities.split_artists {
            let mut page = 0u64;
            loop {
                let views = self
                    .get_album_artists(GetArtistsParams {
                        pagination: paginated.then_some(Pagination {
                            start_page: page,
                            limit: PAGE_SIZE,
                        }),
                        sorting: None,
                    })
                    .await?;
                if views.is_empty() {
                    break;
                }
                page += 1;

                upsert_artists(db, &endpoint_id, &views).await?;
                if !paginated {
                    break;
                }
            }
        }

        // Albums + artist_albums
        let mut page = 0u64;
        loop {
            let views = self
                .get_albums(GetAlbumsParams {
                    pagination: paginated.then_some(Pagination {
                        start_page: page,
                        limit: PAGE_SIZE,
                    }),
                    sorting: None,
                })
                .await?;
            if views.is_empty() {
                break;
            }
            page += 1;

            upsert_albums(db, &endpoint_id, &views).await?;

            let pairs: Vec<(String, String)> = views
                .iter()
                .flat_map(|v| {
                    v.album_artists
                        .iter()
                        .map(|a| (a.id.clone(), v.album.id.clone()))
                })
                .collect();
            insert_junctions(
                db,
                "artist_albums",
                "artist_id",
                "album_id",
                "artist",
                &pairs,
            )
            .await?;
            if !paginated {
                break;
            }
        }

        // Genres
        let mut page = 0u64;
        loop {
            let views = self
                .get_genres(GetGenresParams {
                    pagination: paginated.then_some(Pagination {
                        start_page: page,
                        limit: PAGE_SIZE,
                    }),
                    sorting: None,
                })
                .await?;
            if views.is_empty() {
                break;
            }
            page += 1;

            upsert_genres(db, &endpoint_id, &views).await?;
            if !paginated {
                break;
            }
        }

        // Tracks + artist_tracks + track_genres
        let mut page = 0u64;
        loop {
            let mut views = self
                .get_tracks(GetTracksParams {
                    pagination: paginated.then_some(Pagination {
                        start_page: page,
                        limit: PAGE_SIZE,
                    }),
                    sorting: None,
                })
                .await?;
            if views.is_empty() {
                break;
            }
            page += 1;

            upsert_tracks(db, &endpoint_id, &mut views).await?;

            let artist_pairs: Vec<(String, String)> = views
                .iter()
                .flat_map(|v| v.artists.iter().map(|a| (a.id.clone(), v.track.id.clone())))
                .collect();
            let genre_pairs: Vec<(String, String)> = views
                .iter()
                .flat_map(|v| v.genres.iter().map(|g| (g.id.clone(), v.track.id.clone())))
                .collect();
            insert_junctions(
                db,
                "artist_tracks",
                "artist_id",
                "track_id",
                "artist",
                &artist_pairs,
            )
            .await?;
            insert_junctions(
                db,
                "track_genres",
                "genre_id",
                "track_id",
                "genre",
                &genre_pairs,
            )
            .await?;
            if !paginated {
                break;
            }
        }

        // Playlists + playlist_tracks (full replace per playlist)
        let mut page = 0u64;
        loop {
            let views = self
                .get_playlists(GetPlaylistsParams {
                    pagination: paginated.then_some(Pagination {
                        start_page: page,
                        limit: PAGE_SIZE,
                    }),
                    sorting: None,
                })
                .await?;
            if views.is_empty() {
                break;
            }
            page += 1;

            upsert_playlists(db, &endpoint_id, &views).await?;

            for view in &views {
                let playlist_id = view.playlist.id.clone();
                let mut entries: Vec<(i64, String)> = Vec::new();
                let mut position = 0i64;
                let mut track_page = 0u64;
                loop {
                    let tracks = self
                        .get_playlist_tracks(
                            playlist_id.clone(),
                            GetTracksParams {
                                pagination: paginated.then_some(Pagination {
                                    start_page: track_page,
                                    limit: PAGE_SIZE,
                                }),
                                sorting: None,
                            },
                        )
                        .await?;
                    if tracks.is_empty() {
                        break;
                    }
                    track_page += 1;

                    entries.extend(tracks.into_iter().map(|tv| {
                        let p = position;
                        position += 1;
                        (p, tv.track.id)
                    }));
                    if !paginated {
                        break;
                    }
                }
                replace_playlist_tracks(db, &playlist_id, &entries).await?;
            }
            if !paginated {
                break;
            }
        }

        log::info!("Done indexing endpoint {endpoint_id}");
        Ok(())
    }
}
