use std::fs::{self, File};
use std::io::{BufReader, Read, Stdin, Stdout, Write};
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::vec::Vec;

use walkdir::DirEntry;
use walkdir::WalkDir;

use termion::raw::RawTerminal;

use crate::config;
use crate::panes;
use bincode::{deserialize, serialize};

pub fn init_songs() -> Vec<Song> {
    let mut data_path: PathBuf = dirs::config_dir().unwrap();
    data_path.push("rsmus/metadata.bin");
    if data_path.exists() {
        return metadata_from_binary(data_path);
    } else {
        return scan_library_dir();
    }
}

fn scan_library_dir() -> Vec<Song> {
    // Walk through music dir recursively, getting metadata.
    let config = config::Config::from_config_file();
    let mut file_data = Vec::new();
    let mut music_path = PathBuf::new();
    let music_path_str = &config.music_dir;
    if music_path_str.starts_with("~/") {
        music_path = dirs::home_dir().unwrap();
        let x: &[_] = &['~', '/'];
        music_path.push(music_path_str.trim_start_matches(x));
    } else {
        music_path.push(music_path_str);
    }

    let entries: Vec<DirEntry> = WalkDir::new(music_path)
        .into_iter()
        .map(|a| a.unwrap())
        .collect();

    // Send data to threads for (theoretically) faster processing.
    // Needs work.
    let mut slices = entries.chunks(entries.len() / 4);
    let slice1 = slices.next().unwrap().to_vec();
    let slice2 = slices.next().unwrap().to_vec();
    let slice3 = slices.next().unwrap().to_vec();
    let slice4 = slices.next().unwrap().to_vec();
    let data1 = std::thread::spawn(move || thread_closure(slice1));
    let data2 = std::thread::spawn(move || thread_closure(slice2));
    let data3 = std::thread::spawn(move || thread_closure(slice3));
    let data4 = std::thread::spawn(move || thread_closure(slice4));
    let result = data1.join().unwrap();
    let result2 = data2.join().unwrap();
    let result3 = data3.join().unwrap();
    let result4 = data4.join().unwrap();
    file_data.extend(result);
    file_data.extend(result2);
    file_data.extend(result3);
    file_data.extend(result4);

    //store data in json for faster loading
    let data: Vec<u8> = serialize(&file_data).unwrap();
    let mut data_path: PathBuf = dirs::config_dir().unwrap();
    data_path.push("rsmus");

    //create .config dir if it doesnt already exist
    if !data_path.exists() {
        fs::create_dir(data_path.clone());
    }
    data_path.push("metadata.bin");
    let mut metadata = File::create(data_path).unwrap();
    metadata.write_all(&data).unwrap();

    //run library_init to init program
    return file_data;
}

// Recieves a chunk of files and gets metadata for each valid file type
fn thread_closure(entries: Vec<DirEntry>) -> Vec<Song> {
    let mut file_data = Vec::new();
    for entry in entries {
        let path = entry.path().to_str().unwrap();
        if path.ends_with(".flac") {
            file_data.push(get_file_metadata(entry));
        } else if path.ends_with(".mp3") {
            file_data.push(get_file_metadata(entry));
        } else if path.ends_with(".wav") {
            file_data.push(get_file_metadata(entry));
        }
    }
    return file_data;
}

// Gets metadata using taglib, might change in future.
fn get_file_metadata(entry: DirEntry) -> Song {
    let file = taglib::File::new(entry.path()).unwrap();
    let duration = file.audioproperties().unwrap().length();
    let meta = file.tag().unwrap();
    return Song {
        artist: meta.artist().unwrap_or("Unknown".to_string()),
        album: meta.album().unwrap_or("Unknown".to_string()),
        title: meta.title().unwrap_or("Unknown".to_string()),
        path: entry.path().to_str().unwrap().to_string(),
        duration: Some(Duration::new(duration as u64, 0)),
        year: meta.year().unwrap_or(0),
        track: meta.track().unwrap_or(0),
        genre: meta.genre().unwrap_or("Unknown".to_string()),
    };
}

fn metadata_from_binary(data_path: PathBuf) -> Vec<Song> {
    // Open data file and read binary to objects.
    let mut data_file = File::open(data_path).unwrap();
    let mut buffer = Vec::new();
    data_file.read_to_end(&mut buffer).unwrap();

    // Create song objects with data.
    let songs: Vec<Song> = deserialize(&buffer[..]).unwrap();
    return songs;
}

pub fn init_albums(file_data: &Vec<Song>) -> Vec<Album> {
    // Get list of alphabetically sorted unique albums in library.
    let mut albums_list = Vec::new();
    for song in file_data.iter() {
        albums_list.push(&song.album);
    }
    albums_list.sort();
    albums_list.dedup();

    // Create list of album objects containing songs and info.
    let mut albums = Vec::new();

    // TODO: Find a way to differentiate albums with same name while
    // preserving the display of split LPs in both artists' view.
    // Probably by album release date.
    for album in albums_list {
        let songs: Vec<&Song> = file_data
            .iter()
            .filter(|song| &song.album == album)
            .collect();
        let mut artists: Vec<&str> =
            songs.iter().map(|song| song.artist.as_ref()).collect();
        artists.sort();
        artists.dedup();
        albums.push(Album {
            artists: artists,
            songs: songs,
            title: album,
        });
    }
    return albums;
}

pub fn init_artists<'a>(
    songs: &'a Vec<Song>,
    albums: &'a Vec<Album>,
) -> Vec<Artist<'a>> {
    //create alphabetically sorted list of unique artists
    let mut artists_list = Vec::new();
    for song in songs.iter() {
        artists_list.push(&song.artist)
    }
    artists_list.sort();
    artists_list.dedup();

    //create a list of artist objects containing albums and info
    let mut artists = Vec::new();

    for artist in artists_list {
        let albums: Vec<&Album> = albums
            .iter()
            .filter(|album| {
                album.artists.binary_search(&&artist.as_ref()).is_ok()
            })
            .collect();
        artists.push(Artist {
            albums: albums,
            name: artist,
        });
    }
    return artists;
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Song {
    pub artist: String,
    pub album: String,
    pub title: String,
    pub path: String,
    pub duration: Option<std::time::Duration>,
    pub track: u32,
    pub year: u32,
    pub genre: String,
}

#[derive(Clone)]
pub struct Album<'a> {
    pub songs: Vec<&'a Song>,
    pub artists: Vec<&'a str>,
    pub title: &'a str,
}
#[derive(Clone)]
pub struct Artist<'a> {
    pub albums: Vec<&'a Album<'a>>,
    pub name: &'a str,
}
