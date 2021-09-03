mod user;
mod torrent_listing;
mod torrent_file;

pub type User = user::User;
pub type TorrentListing = torrent_listing::TorrentListing;
pub type Torrent = torrent_file::Torrent;
pub type File = torrent_file::File;
pub type Info = torrent_file::Info;
pub type Node = torrent_file::Node;