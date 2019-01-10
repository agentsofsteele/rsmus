use std::io::{stdin, stdout, Write};

use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub mod panes;
use crate::panes::Pane;

pub mod metadata;
use crate::metadata::{Album, Artist, Song};

#[macro_use]
extern crate serde_derive;

fn main() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let termsize = termion::terminal_size().ok();
    let width = termsize.map(|(w, _)| w - 2).unwrap();
    let height = termsize.map(|(_, h)| h - 2).unwrap();

    write!(
        stdout,
        "{}{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        termion::cursor::Hide
    )
    .unwrap();

    panes::draw_box(&mut stdout, width, height, (1, 1));

    let (mut artists, mut albums, mut songs) = metadata::init();

    let mut focused_pane = FocusedPane::Pane1;

    let mut artist_pane = Pane::init_artist_pane(&artists, &albums, height);

    artist_pane.draw(&mut stdout, &focused_pane);
    stdout.flush().unwrap();

    let mut stdin = stdin.keys();
    loop {
        let event = stdin.next().unwrap().unwrap();
        use termion::event::Key::*;
        match event {
            Char('k') | Up => {
                move_up(&albums, height, &focused_pane, &mut artist_pane);
                artist_pane.draw(&mut stdout, &focused_pane);
            }
            Char('j') | Down => {
                move_down(&albums, height, &focused_pane, &mut artist_pane);
                artist_pane.draw(&mut stdout, &focused_pane);
            }
            Char('l') | Right => {
                focused_pane = move_right(&focused_pane);
                artist_pane.draw(&mut stdout, &focused_pane)
            }
            Char('h') | Left => {
                focused_pane = move_left(&focused_pane);
                artist_pane.draw(&mut stdout, &focused_pane)
            }
            Char('\n') | Char(' ') => {
                println!("{}", artist_pane.get_selected());
            }
            Char('q') => return (),
            _ => {}
        }
        stdout.flush().unwrap();
    }
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

fn move_up(
    albums: &Vec<Album>,
    height: u16,
    focused_pane: &FocusedPane,
    root_pane: &mut Pane,
) {
    match focused_pane {
        FocusedPane::Pane1 => {
            root_pane.move_up(albums, height);
        }
        FocusedPane::Pane2 => {
            root_pane.move_child_up(albums, height);
        }
        FocusedPane::Pane3 => {}
    }
}
fn move_down(
    albums: &Vec<Album>,
    height: u16,
    focused_pane: &FocusedPane,
    root_pane: &mut Pane,
) {
    match focused_pane {
        FocusedPane::Pane1 => {
            root_pane.move_down(albums, height);
        }
        FocusedPane::Pane2 => {
            root_pane.move_child_down(albums, height);
        }
        FocusedPane::Pane3 => {}
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
