use crate::backend::{
    api::jellyfin::api::JellyfinApi, data::{Artist, Track}
};

pub enum Endpoint {
    Jellyfin(JellyfinApi),
}


pub trait API {
    fn get_track(&self) -> impl Future<Output = Track> + Send;
    fn get_tracks(&self) -> impl Future<Output = Vec<Track>> + Send;
    fn get_artist(&self) -> impl Future<Output = Artist> + Send;
    fn get_artists(&self) -> impl Future<Output = Vec<Artist>> + Send;
}
