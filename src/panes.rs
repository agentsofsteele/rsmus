use crate::metadata::{Album, Artist, Song};
use crate::FocusedPane;
use std::boxed::Box;
use std::io::{Stdout, Write};
use termion::color;
use termion::cursor;
use termion::raw::RawTerminal;
use termion::style::*;

const HORZ_BOUNDARY: &'static str = "─";
const VERT_BOUNDARY: &'static str = "│";

const TOP_LEFT_CORNER: &'static str = "┌";
const TOP_RIGHT_CORNER: &'static str = "┐";
const BOTTOM_LEFT_CORNER: &'static str = "└";
const BOTTOM_RIGHT_CORNER: &'static str = "┘";

#[derive(PartialEq)]
enum PaneType {
    MenuPane,
    AlbumPane,
}

pub struct Pane<'a> {
    options: Vec<String>,
    reference: usize,
    height: u16,
    width: u16,
    cursor_pos: usize,
    pos: (u16, u16),
    focus: FocusedPane,
    pane_type: PaneType,
    pub child_pane: Option<Box<Pane<'a>>>,
    album: Option<&'a Album>,
}
impl<'a> Pane<'a> {
    pub fn init_artist_pane(
        artists: &Vec<Artist>,
        albums: &'a Vec<Album>,
        size: (u16, u16),
    ) -> Pane<'a> {
        let options: Vec<String> = artists
            .clone()
            .into_iter()
            .map(|artist| artist.name)
            .collect();
        let height = size.1;
        let width = size.0 / 5;
        return Pane {
            options: options.clone(),
            reference: 0 as usize,
            height: height,
            width: width,
            cursor_pos: 0,
            pos: (1, 2),
            album: None,
            focus: FocusedPane::Pane1,
            pane_type: PaneType::MenuPane,
            child_pane: Some(Box::new(Pane::init_artist_album_pane(
                &options[0],
                albums,
                size,
            ))),
        };
    }

    fn init_artist_album_pane(
        artist: &str,
        albums: &'a Vec<Album>,
        size: (u16, u16),
    ) -> Pane<'a> {
        let options: Vec<String> = albums
            .clone()
            .into_iter()
            .filter(|album| {
                album.artists.binary_search(&artist.to_string()).is_ok()
            })
            .map(|album| album.title)
            .collect();
        let height = size.1;
        let width = size.0 / 5;
        let x = size.0 / 5 + 2;
        return Pane {
            options: options.clone(),
            reference: 0 as usize,
            height: height,
            width: width,
            cursor_pos: 0,
            pos: (x, 2),
            album: None,
            pane_type: PaneType::MenuPane,
            focus: FocusedPane::Pane2,
            child_pane: Some(Box::new(Pane::init_album_view_pane(
                &options[0],
                albums,
                size,
            ))),
        };
    }

    fn init_album_view_pane(
        album_title: &str,
        albums: &'a Vec<Album>,
        size: (u16, u16),
    ) -> Pane<'a> {
        let album: &Album = albums
            .into_iter()
            .find(|album| album.title == album_title)
            .unwrap();
        let options: Vec<String> = album
            .clone()
            .songs
            .into_iter()
            .map(|song| song.title)
            .collect();
        let height = size.1;
        let width = size.0 - (size.0 / 5 * 2) - 1;
        let x = (size.0 / 5) * 2 + 2;
        return Pane {
            options: options,
            reference: 0 as usize,
            height: height,
            width: width,
            cursor_pos: 0,
            pos: (x, 2),
            album: Some(album),
            pane_type: PaneType::AlbumPane,
            focus: FocusedPane::Pane3,
            child_pane: None,
        };
    }

    pub fn draw(
        &mut self,
        stdout: &mut RawTerminal<Stdout>,
        focused_pane: &FocusedPane,
        size: (u16, u16),
    ) {
        match self.pane_type {
            PaneType::MenuPane => {
                self.height = size.1;
                self.width = size.0 / 5;
                self.draw_menu_pane(stdout, focused_pane);
            }
            PaneType::AlbumPane => {
                self.height = size.1;
                self.width = size.0 - (size.0 / 5 * 2) - 1;
                self.draw_album_view_pane(stdout, focused_pane)
            }
        }
        match self.child_pane {
            Some(ref mut pane) => pane.draw(stdout, focused_pane, size),
            None => {}
        }
    }

    fn draw_album_view_pane(
        &self,
        stdout: &mut RawTerminal<Stdout>,
        focused_pane: &FocusedPane,
    ) {
        let x = self.pos.0;
        let mut y = self.pos.1;
        draw_box(stdout, self.width, self.height, (x, y - 1));
        let mut shown_options: &[String] = &[];
        if self.options.len() > self.height as usize {
            shown_options = &self.options
                [self.reference..(self.height as usize + self.reference)];
        } else {
            shown_options = &self.options[..]
        }

        let mut title: &str = "";
        let mut artists: &Vec<String> = &vec![];

        write!(stdout, "{}", cursor::Goto(x, y));
        match &self.album {
            Some(ref album) => {
                title = &album.title;
                artists = &album.artists;
            }
            None => {}
        }
        y += 1;
        write!(
            stdout,
            "{}{}{}{}",
            VERT_BOUNDARY,
            Bold,
            title,
            cursor::Goto(x, y)
        );
        write!(stdout, "{}{}", VERT_BOUNDARY, NoBold);
        for artist in artists {
            write!(stdout, "{} ", artist);
        }

        y += 1;
        for num in 0..shown_options.len() {
            let mut option = shown_options[num as usize].clone();

            if option.chars().count() > self.width as usize - 4{
                option.truncate(self.width as usize - 6);
                option.push_str("..");
            }

            write!(stdout, "{}", cursor::Goto(x, y));
            if self.cursor_pos == num && focused_pane == &self.focus {
                write!(
                    stdout,
                    "{}    {}{}{}",
                    VERT_BOUNDARY, Invert, option, NoInvert
                );
            } else {
                write!(
                    stdout,
                    "{}    {}",
                    VERT_BOUNDARY, option
                );
            }
            y += 1;
        }
    }

    fn draw_menu_pane(
        &self,
        stdout: &mut RawTerminal<Stdout>,
        focused_pane: &FocusedPane,
    ) {
        let mut shown_options: &[String] = &[];
        if self.options.len() > self.height as usize {
            shown_options = &self.options
                [self.reference..(self.height as usize + self.reference)];
        } else {
            shown_options = &self.options[..]
        }
        draw_box(
            stdout,
            self.width,
            self.height,
            (self.pos.0, self.pos.1 - 1),
        );
        write!(stdout, "{}", cursor::Goto(self.pos.0, self.pos.1));
        for num in 0..self.height {
            let mut option = std::string::String::new();
            if shown_options.len() > num as usize {
                option = shown_options[num as usize].clone();
            }
            if option.chars().count() > self.width as usize {
                while option.chars().count() >= self.width as usize - 2 {
                    option.pop();
                }
                option.push_str("..");
            }
            if self.cursor_pos as u16 == num && focused_pane == &self.focus {
                write!(
                    stdout,
                    "{}{}{}{}{}",
                    VERT_BOUNDARY,
                    Invert,
                    option,
                    NoInvert,
                    cursor::Goto(self.pos.0, (num + 3)),
                )
                .unwrap();
            } else {
                write!(
                    stdout,
                    "{}{}{}",
                    VERT_BOUNDARY,
                    option,
                    termion::cursor::Goto(self.pos.0, (num + 3))
                )
                .unwrap();
            }
        }
    }

    pub fn move_down(&mut self, albums: &'a Vec<Album>, size: (u16, u16)) {
        if (self.reference as i16)
            < (self.options.len() as i16 - self.height as i16)
        {
            if self.cursor_pos < self.height as usize - 1 {
                self.cursor_pos += 1;
            } else {
                self.reference += 1;
            }
        } else if self.cursor_pos < self.height as usize - 1
            && self.cursor_pos < (self.options.len() - 1)
        {
            self.cursor_pos += 1;
        }
        self.reset_child(albums, size);
    }
    pub fn move_up(&mut self, albums: &'a Vec<Album>, size: (u16, u16)) {
        if self.reference > 0 {
            if self.cursor_pos > 0 {
                self.cursor_pos -= 1;
            } else {
                self.reference -= 1;
            }
        } else if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
        self.reset_child(albums, size);
    }

    pub fn move_child_down(
        &mut self,
        albums: &'a Vec<Album>,
        size: (u16, u16),
    ) {
        match self.child_pane {
            Some(ref mut pane) => pane.move_down(albums, size),
            None => {}
        }
    }

    pub fn move_child_up(&mut self, albums: &'a Vec<Album>, size: (u16, u16)) {
        match self.child_pane {
            Some(ref mut pane) => pane.move_up(albums, size),
            None => {}
        }
    }

    pub fn get_child_selected(&self) -> &str {
        match self.child_pane {
            Some(ref pane) => return pane.get_selected(),
            None => return "error",
        }
    }

    pub fn get_selected(&self) -> &str {
        return &self.options[self.reference + self.cursor_pos];
    }

    pub fn reset_child<'b>(
        &mut self,
        albums: &'a Vec<Album>,
        size: (u16, u16),
    ) {
        match self.child_pane {
            Some(ref p) => match p.pane_type {
                PaneType::MenuPane => {
                    self.child_pane =
                        Some(Box::new(Pane::init_artist_album_pane(
                            self.get_selected(),
                            albums,
                            size,
                        )));
                }
                PaneType::AlbumPane => {
                    self.child_pane =
                        Some(Box::new(Pane::init_album_view_pane(
                            self.get_selected(),
                            albums,
                            size,
                        )));
                }
            },
            None => {}
        }
    }
}

// Draw border for screen.
pub fn draw_box(
    stdout: &mut RawTerminal<Stdout>,
    width: u16,
    height: u16,
    pos: (u16, u16),
) {
    let x = pos.0;
    let mut y = pos.1;
    write!(stdout, "{}", cursor::Goto(x, y)).unwrap();
    // Init border for top row.
    stdout.write(TOP_LEFT_CORNER.as_bytes()).unwrap();
    for _ in 0..width {
        stdout.write(HORZ_BOUNDARY.as_bytes()).unwrap();
    }
    stdout.write(TOP_RIGHT_CORNER.as_bytes()).unwrap();

    y += 1;
    write!(stdout, "{}", cursor::Goto(x, y));

    // Init border for middle cells.
    for _ in 0..height {
        stdout.write(VERT_BOUNDARY.as_bytes()).unwrap();

        for _ in 0..width {
            stdout.write(b" ").unwrap();
        }
        stdout.write(VERT_BOUNDARY.as_bytes()).unwrap();
        y += 1;
        write!(stdout, "{}", cursor::Goto(x, y));
    }

    // init border for bottom row
    stdout.write(BOTTOM_LEFT_CORNER.as_bytes()).unwrap();
    for _ in 0..width {
        stdout.write(HORZ_BOUNDARY.as_bytes()).unwrap();
    }
    stdout.write(BOTTOM_RIGHT_CORNER.as_bytes()).unwrap();
}

pub fn clear_area(
    stdout: &mut RawTerminal<Stdout>,
    width: u16,
    height: u16,
    pos: (u16, u16),
) {
    write!(stdout, "{}", cursor::Goto(pos.0, pos.1)).unwrap();
    for y in 0..height {
        for x in 0..width {
            write!(stdout, " ");
        }
        write!(stdout, "{}", cursor::Goto(pos.0, pos.1 + y)).unwrap();
    }
}
