use std::fs::File;
use std::io::{stdin, stdout, BufReader, Write};

use termion::input::TermRead;
use termion::raw::IntoRawMode;

use rodio::Sink;

use simplelog::*;

pub mod panes;
use crate::panes::Pane;

pub mod metadata;
use crate::metadata::{Album, Artist, Song};

#[macro_use]
extern crate serde_derive;

fn main() {
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("my_rust_bin.log").unwrap(),
    );
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let mut size = refresh_size();

    write!(
        stdout,
        "{}{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        termion::cursor::Hide
    )
    .unwrap();

    let (mut artists, mut albums, mut songs) = metadata::init();

    let mut focused_pane = FocusedPane::Pane1;

    let mut artist_pane = Pane::init_artist_pane(&artists, &albums, size);

    artist_pane.draw(&mut stdout, &focused_pane, size);
    stdout.flush().unwrap();

    let device = rodio::default_output_device().unwrap();
    let sink = Sink::new(&device);

    let mut stdin = stdin.keys();
    loop {
        size = refresh_size();
        let event = stdin.next().unwrap().unwrap();
        use termion::event::Key::*;
        match event {
            Char('k') | Up => {
                move_up(&albums, size, &focused_pane, &mut artist_pane);
                artist_pane.draw(&mut stdout, &focused_pane, size);
            }
            Char('j') | Down => {
                move_down(&albums, size, &focused_pane, &mut artist_pane);
                artist_pane.draw(&mut stdout, &focused_pane, size);
            }
            Char('l') | Right => {
                focused_pane = move_right(&focused_pane);
                artist_pane.draw(&mut stdout, &focused_pane, size)
            }
            Char('h') | Left => {
                focused_pane = move_left(&focused_pane);
                artist_pane.draw(&mut stdout, &focused_pane, size)
            }
            Char('\n') | Char(' ') => {
                play_song(&focused_pane, &artist_pane, &songs, &sink);
            }
            Char('q') => return (),
            _ => {}
        }
        stdout.flush().unwrap();
    }
}

fn refresh_size() -> (u16, u16) {
    let termsize = termion::terminal_size().ok();
    let width = termsize.map(|(w, _)| w - 2).unwrap();
    let height = termsize.map(|(_, h)| h - 2).unwrap();
    return (width, height);
}

fn move_right(focused_pane: &FocusedPane) -> FocusedPane {
    match focused_pane {
        FocusedPane::Pane1 => return FocusedPane::Pane2,
        FocusedPane::Pane2 => return FocusedPane::Pane3,
        FocusedPane::Pane3 => return FocusedPane::Pane3,
    }
}
fn move_left(focused_pane: &FocusedPane) -> FocusedPane {
    match focused_pane {
        FocusedPane::Pane1 => return FocusedPane::Pane1,
        FocusedPane::Pane2 => return FocusedPane::Pane1,
        FocusedPane::Pane3 => return FocusedPane::Pane2,
    }
}

fn move_up<'a, 'b>(
    albums: &'a Vec<Album>,
    size: (u16, u16),
    focused_pane: &FocusedPane,
    root_pane: &'b mut Pane<'a>,
) {
    match focused_pane {
        FocusedPane::Pane1 => {
            root_pane.move_up(albums, size);
        }
        FocusedPane::Pane2 => {
            root_pane.move_child_up(albums, size);
        }
        FocusedPane::Pane3 => match root_pane.child_pane {
            Some(ref mut pane) => pane.move_child_up(albums, size),
            None => {}
        },
    }
}
fn move_down<'a, 'b>(
    albums: &'a Vec<Album>,
    size: (u16, u16),
    focused_pane: &FocusedPane,
    root_pane: &'b mut Pane<'a>,
) {
    match focused_pane {
        FocusedPane::Pane1 => {
            root_pane.move_down(albums, size);
        }
        FocusedPane::Pane2 => {
            root_pane.move_child_down(albums, size);
        }
        FocusedPane::Pane3 => match root_pane.child_pane {
            Some(ref mut pane) => pane.move_child_down(albums, size),
            None => {}
        },
    }
}

fn get_selected<'a>(
    focused_pane: &FocusedPane,
    root_pane: &'a Pane,
) -> &'a str {
    match focused_pane {
        FocusedPane::Pane1 => root_pane.get_selected(),
        FocusedPane::Pane2 => root_pane.get_child_selected(),
        FocusedPane::Pane3 => match root_pane.child_pane {
            Some(ref pane) => pane.get_child_selected(),
            None => return "error",
        },
    }
}

fn play_song(
    focused_pane: &FocusedPane,
    root_pane: &Pane,
    songs: &Vec<Song>,
    sink: &Sink,
) {
    if *focused_pane == FocusedPane::Pane3 {
        let song_title = get_selected(focused_pane, root_pane);
        let song: &Song = songs
            .into_iter()
            .find(|song| song.title == song_title)
            .unwrap();

        let file = File::open(song.path.clone()).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
        sink.append(source);
    }
}

#[derive(PartialEq)]
pub enum UiState {
    AlbumArtistView,
    SearchView,
}

#[derive(PartialEq)]
pub enum FocusedPane {
    Pane1,
    Pane2,
    Pane3,
}
