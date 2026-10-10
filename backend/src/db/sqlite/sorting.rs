//! This module contains a bunch of helpers for creating SQL queries for sorting
//!
//! TODO: One thing to consider is that RANDOM with pagination currently makes
//! no sense as the result will be randomize per page

use crate::api::endpoint_api::{
    AlbumSorting, ArtistSorting, GenreSorting, Pagination, PlaylistSorting, Sort, SortOrder,
    TrackSorting,
};

fn order_dir(order: SortOrder) -> &'static str {
    match order {
        SortOrder::Ascending => "ASC",
        SortOrder::Descending => "DESC",
    }
}

pub fn track_order_by(sorting: Sort<TrackSorting>) -> String {
    use TrackSorting as S;
    let expr: &str = match sorting.by {
        S::Album => "a.title COLLATE NOCASE",
        S::AlbumArtist => {
            "(SELECT ar.name FROM artist_albums aa JOIN artist ar ON ar.id = aa.artist_id \
                            WHERE aa.album_id = t.album_id ORDER BY ar.name COLLATE NOCASE LIMIT 1)"
        }
        S::Number => "t.disc_number, t.track_number",
        S::Title => "t.title COLLATE NOCASE",
        S::Artist => {
            "(SELECT ar.name FROM artist_tracks at2 JOIN artist ar ON ar.id = at2.artist_id \
                            WHERE at2.track_id = t.id ORDER BY ar.name COLLATE NOCASE LIMIT 1)"
        }
        S::Duration => "t.duration",
        S::PlayCount => "t.play_count",
        S::Random => "RANDOM()",
        S::DateAdded => "t.date_added",
        S::DatePlayed => "t.last_played_at",
        S::DateReleased => "t.release_date",
    };
    format!("ORDER BY {expr} {}, t.id", order_dir(sorting.order))
}

pub fn album_order_by(sorting: Sort<AlbumSorting>) -> String {
    use AlbumSorting as S;
    let expr: &str = match sorting.by {
        S::Name => "a.title COLLATE NOCASE",
        S::AlbumArtist => {
            "(SELECT ar.name FROM artist_albums aa JOIN artist ar ON ar.id = aa.artist_id \
                            WHERE aa.album_id = a.id ORDER BY ar.name COLLATE NOCASE LIMIT 1)"
        }
        S::TrackCount => "(SELECT COUNT(*) FROM track t WHERE t.album_id = a.id)",
        S::Duration => "(SELECT SUM(t.duration) FROM track t WHERE t.album_id = a.id)",
        S::DateAdded => "a.date_added",
        S::DateReleased => "a.release_date",
        S::Random => "RANDOM()",
    };
    format!("ORDER BY {expr} {}, a.id", order_dir(sorting.order))
}

pub fn artist_order_by(sorting: Sort<ArtistSorting>) -> String {
    use ArtistSorting as S;
    let expr: &str = match sorting.by {
        S::Name => "ar.name COLLATE NOCASE",
        S::AlbumCount => "(SELECT COUNT(*) FROM artist_albums aa WHERE aa.artist_id = ar.id)",
        S::TrackCount => "(SELECT COUNT(*) FROM artist_tracks at WHERE at.artist_id = ar.id)",
        S::Duration => {
            "(SELECT SUM(t.duration) FROM artist_tracks at JOIN track t ON t.id = at.track_id \
                           WHERE at.artist_id = ar.id)"
        }
        S::Random => "RANDOM()",
    };
    format!("ORDER BY {expr} {}, ar.id", order_dir(sorting.order))
}

pub fn genre_order_by(sorting: Sort<GenreSorting>) -> String {
    use GenreSorting as S;
    let expr: &str = match sorting.by {
        S::Name => "g.name COLLATE NOCASE",
        S::TrackCount => "(SELECT COUNT(*) FROM track_genres tg WHERE tg.genre_id = g.id)",
        S::AlbumCount => {
            "(SELECT COUNT(DISTINCT t.album_id) FROM track_genres tg \
                           JOIN track t ON t.id = tg.track_id WHERE tg.genre_id = g.id)"
        }
        S::Duration => {
            "(SELECT SUM(t.duration) FROM track_genres tg \
                           JOIN track t ON t.id = tg.track_id WHERE tg.genre_id = g.id)"
        }
        S::Random => "RANDOM()",
    };
    format!("ORDER BY {expr} {}, g.id", order_dir(sorting.order))
}

pub fn playlist_order_by(sorting: Sort<PlaylistSorting>) -> String {
    use PlaylistSorting as S;
    let expr: &str = match sorting.by {
        S::Name => "p.name COLLATE NOCASE",
        S::TrackCount => "(SELECT COUNT(*) FROM playlist_tracks pt WHERE pt.playlist_id = p.id)",
        S::AlbumCount => {
            "(SELECT COUNT(DISTINCT t.album_id) FROM playlist_tracks pt \
                           JOIN track t ON t.id = pt.track_id WHERE pt.playlist_id = p.id)"
        }
        S::Duration => {
            "(SELECT SUM(t.duration) FROM playlist_tracks pt \
                           JOIN track t ON t.id = pt.track_id WHERE pt.playlist_id = p.id)"
        }
        S::Random => "RANDOM()",
    };
    format!("ORDER BY {expr} {}, p.id", order_dir(sorting.order))
}

pub(crate) fn pagination(p: &Option<Pagination>) -> (String, Option<(i64, i64)>) {
    let Some(p) = p else {
        return (String::new(), None);
    };
    let limit = i64::try_from(p.limit).unwrap_or(i64::MAX);
    let offset = i64::try_from(p.start_page.saturating_mul(p.limit)).unwrap_or(i64::MAX);
    (" LIMIT ? OFFSET ?".into(), Some((limit, offset)))
}

pub(crate) fn like_pattern(term: &str) -> String {
    format!(
        "%{}%",
        term.replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    )
}
