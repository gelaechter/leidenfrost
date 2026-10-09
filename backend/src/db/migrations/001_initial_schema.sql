-- SQLite 3
-- Base Entities
CREATE TABLE
    "endpoint" ("id" TEXT NOT NULL PRIMARY KEY);

CREATE TABLE
    "artist" (
        "id" TEXT NOT NULL PRIMARY KEY,
        "endpoint_id" TEXT NOT NULL,
        "name" TEXT,
        "biography" TEXT,
        "image_url" TEXT,
        "image_blur_hash" TEXT,
        "background_image_url" TEXT,
        "background_image_blurhash" TEXT,
        "play_count" INTEGER,
        "date_added" TEXT,
        "user_favorite" INTEGER,
        "user_rating" INTEGER,
        FOREIGN KEY ("endpoint_id") REFERENCES "endpoint" ("id")
    );

CREATE TABLE
    "album" (
        "id" TEXT NOT NULL PRIMARY KEY,
        "endpoint_id" TEXT NOT NULL,
        "release_date" TEXT,
        "date_added" TEXT,
        "title" TEXT,
        "image_url" TEXT,
        "image_blur_hash" TEXT,
        "user_favorite" INTEGER,
        FOREIGN KEY ("endpoint_id") REFERENCES "endpoint" ("id")
    );

CREATE TABLE
    "track" (
        "id" TEXT NOT NULL PRIMARY KEY,
        "endpoint_id" TEXT NOT NULL,
        "album_id" TEXT,
        "disc_number" INTEGER NOT NULL,
        "bit_rate" INTEGER,
        "bpm" INTEGER,
        "channels" INTEGER,
        "container" TEXT,
        "duration" INTEGER,
        "image_url" TEXT,
        "image_blur_hash" TEXT,
        "last_played_at" TEXT,
        "lyrics" TEXT,
        "title" TEXT,
        "file_path" TEXT,
        "play_count" INTEGER,
        "release_date" TEXT,
        "date_added" TEXT,
        "file_size" INTEGER,
        "stream_url" TEXT,
        "track_number" INTEGER,
        "user_favorite" INTEGER,
        FOREIGN KEY ("endpoint_id") REFERENCES "endpoint" ("id"),
        FOREIGN KEY ("album_id") REFERENCES "album" ("id")
    );

CREATE TABLE
    "genre" (
        "id" TEXT NOT NULL PRIMARY KEY,
        "endpoint_id" TEXT NOT NULL,
        "name" TEXT,
        "image_url" TEXT,
        "image_blur_hash" TEXT,
        FOREIGN KEY ("endpoint_id") REFERENCES "endpoint" ("id")
    );

CREATE TABLE
    "playlist" (
        "id" TEXT NOT NULL PRIMARY KEY,
        "endpoint_id" TEXT NOT NULL,
        "name" TEXT,
        "image_url" TEXT,
        "image_blur_hash" TEXT,
        "user_favorite" INTEGER,
        FOREIGN KEY ("endpoint_id") REFERENCES "endpoint" ("id")
    );

-- Junctions:
CREATE TABLE
    "artist_albums" (
        "artist_id" TEXT NOT NULL,
        "album_id" TEXT NOT NULL,
        CONSTRAINT "pk-artist_albums" PRIMARY KEY ("artist_id", "album_id"),
        FOREIGN KEY ("artist_id") REFERENCES "artist" ("id"),
        FOREIGN KEY ("album_id") REFERENCES "album" ("id")
    );

CREATE TABLE
    "artist_tracks" (
        "artist_id" TEXT NOT NULL,
        "track_id" TEXT NOT NULL,
        CONSTRAINT "pk-artist_tracks" PRIMARY KEY ("artist_id", "track_id"),
        FOREIGN KEY ("artist_id") REFERENCES "artist" ("id"),
        FOREIGN KEY ("track_id") REFERENCES "track" ("id")
    );

CREATE TABLE
    "playlist_tracks" (
        "playlist_id" TEXT NOT NULL,
        "position" INTEGER NOT NULL,
        "track_id" TEXT NOT NULL,
        -- We want to allow tracks doubly
        PRIMARY KEY ("playlist_id", "position"),
        FOREIGN KEY ("track_id") REFERENCES "track" ("id"),
        FOREIGN KEY ("playlist_id") REFERENCES "playlist" ("id")
    );

CREATE TABLE
    "track_genres" (
        "track_id" TEXT NOT NULL,
        "genre_id" TEXT NOT NULL,
        CONSTRAINT "pk-track_genres" PRIMARY KEY ("track_id", "genre_id"),
        FOREIGN KEY ("track_id") REFERENCES "track" ("id"),
        FOREIGN KEY ("genre_id") REFERENCES "genre" ("id")
    );

-- Indices:
CREATE INDEX idx_artist_endpoint_id ON artist (endpoint_id);
CREATE INDEX idx_album_endpoint_id ON album (endpoint_id);
CREATE INDEX idx_track_album_id ON track (album_id);
CREATE INDEX idx_track_endpoint_id ON track (endpoint_id);

-- Reverse-direction junction lookups (UNIQUE doubles as a constraint)
CREATE UNIQUE INDEX idx_artist_albums_album ON artist_albums (album_id, artist_id);
CREATE UNIQUE INDEX idx_artist_tracks_track ON artist_tracks (track_id, artist_id);
CREATE INDEX idx_playlist_tracks_playlist ON playlist_tracks (playlist_id, track_id);
CREATE INDEX idx_track_genres_genre ON track_genres (genre_id, track_id);

-- Text sort/search support
CREATE INDEX idx_track_title ON track (title COLLATE NOCASE);
CREATE INDEX idx_album_title ON album (title COLLATE NOCASE);
CREATE INDEX idx_artist_name ON artist (name COLLATE NOCASE);
CREATE INDEX idx_genre_name ON genre (name COLLATE NOCASE);
CREATE INDEX idx_playlist_name ON playlist (name COLLATE NOCASE);