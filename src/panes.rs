use crate::metadata::{Album, Artist, Song};
use crate::FocusedPane;
use std::boxed::Box;
use std::io::{Stdout, Write};
use termion::color;
use termion::cursor;
use termion::raw::RawTerminal;

const HORZ_BOUNDARY: &'static str = "─";
const VERT_BOUNDARY: &'static str = "│";

const TOP_LEFT_CORNER: &'static str = "┌";
const TOP_RIGHT_CORNER: &'static str = "┐";
const BOTTOM_LEFT_CORNER: &'static str = "└";
const BOTTOM_RIGHT_CORNER: &'static str = "┘";

pub struct Pane {
    options: Vec<String>,
    reference: usize,
    height: u16,
    width: u16,
    cursor_pos: usize,
    pane_pos: (u16, u16),
    pane_type: FocusedPane,
    pub child_pane: Option<Box<Pane>>,
    pub child_final: Option<AlbumView>,
}
impl Pane {
    pub fn init_artist_pane(
        artists: &Vec<Artist>,
        albums: &Vec<Album>,
        height: u16,
    ) -> Pane {
        let options: Vec<String> = artists
            .clone()
            .into_iter()
            .map(|artist| artist.name)
            .collect();
        return Pane {
            options: options.clone(),
            reference: 0 as usize,
            height: height,
            width: 25,
            cursor_pos: 0,
            pane_pos: (1, 2),
            pane_type: FocusedPane::Pane1,
            child_final: None,
            child_pane: Some(Box::new(Pane::init_artist_album_pane(
                options[0].clone(),
                albums,
                height,
            ))),
        };
    }

    fn init_artist_album_pane(
        artist: String,
        albums: &Vec<Album>,
        height: u16,
    ) -> Pane {
        let options: Vec<String> = albums
            .clone()
            .into_iter()
            .filter(|album| album.artists.binary_search(&artist).is_ok())
            .map(|album| album.title)
            .collect();
        return Pane {
            options: options.clone(),
            reference: 0 as usize,
            height: height,
            width: 25,
            cursor_pos: 0,
            pane_pos: (27, 2),
            pane_type: FocusedPane::Pane2,
            child_pane: None,
            child_final: Some(AlbumView {
                album: options[0].clone(),
                pos: (52, 2),
                pane_type: FocusedPane::Pane3,
                cursor_pos: 0,
            }),
        };
    }
    pub fn draw(
        &self,
        stdout: &mut RawTerminal<Stdout>,
        focused_pane: &FocusedPane,
    ) {
        self.draw_pane(stdout, focused_pane);
        match self.child_pane {
            Some(ref pane) => pane.draw_pane(stdout, focused_pane),
            None => {}
        }
    }

    fn draw_pane(
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
        write!(stdout, "{}", cursor::Goto(self.pane_pos.0, self.pane_pos.1));
        for num in 0..self.height {
            let mut spacer = std::string::String::new();
            let mut option = std::string::String::new();
            if shown_options.len() > num as usize {
                option = shown_options[num as usize].clone();
            }
            if option.chars().count() > self.width as usize {
                option.truncate(self.width as usize);
                option.push_str("..");
            }
            while option.chars().count() + spacer.len() < self.width as usize {
                spacer.push(' ');
            }
            if self.cursor_pos as u16 == num && focused_pane == &self.pane_type
            {
                write!(
                    stdout,
                    "{}{}{}{}{}{}{}{}",
                    VERT_BOUNDARY,
                    color::Bg(color::White),
                    color::Fg(color::Black),
                    option,
                    spacer,
                    color::Bg(color::Reset),
                    color::Fg(color::Reset),
                    cursor::Goto(self.pane_pos.0, (num + 3)),
                )
                .unwrap();
            } else {
                write!(
                    stdout,
                    "{}{}{}{}",
                    VERT_BOUNDARY,
                    option,
                    spacer,
                    termion::cursor::Goto(self.pane_pos.0, (num + 3))
                )
                .unwrap();
            }
        }
    }

    pub fn move_down(&mut self, albums: &Vec<Album>, height: u16) {
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
        self.reset_child(albums, height);
    }
    pub fn move_up(&mut self, albums: &Vec<Album>, height: u16) {
        if self.reference > 0 {
            if self.cursor_pos > 0 {
                self.cursor_pos -= 1;
            } else {
                self.reference -= 1;
            }
        } else if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
        self.reset_child(albums, height);
    }

    pub fn move_child_down(&self, albums: &Vec<Album>, height: u16) {
        match self.child_pane {
            Some(ref mut pane) => pane.move_down(albums, height),
            None => {}
        }
    }

    pub fn get_selected(&self) -> String {
        return self.options[self.reference + self.cursor_pos].clone();
    }

    pub fn reset_child(&mut self, albums: &Vec<Album>, height: u16) {
        match self.child_pane {
            Some(ref p) => {
                self.child_pane = Some(Box::new(Pane::init_artist_album_pane(
                    self.get_selected(),
                    albums,
                    height,
                )));
            }
            None => {}
        }
    }
}

pub struct AlbumView {
    album: String,
    //songs: Vec<String>,
    pos: (u16, u16),
    pane_type: FocusedPane,
    cursor_pos: usize,
}

impl AlbumView {
    pub fn draw(&self, stdout: &mut RawTerminal<Stdout>) {
        write!(stdout, "{}", cursor::Goto(self.pos.0, self.pos.1));
    }
}

// Draw border for screen.
pub fn draw_box(
    stdout: &mut RawTerminal<Stdout>,
    width: u16,
    height: u16,
    pos: (u16, u16),
) {
    write!(stdout, "{}", cursor::Goto(pos.0, pos.1)).unwrap();
    // Init border for top row.
    stdout.write(TOP_LEFT_CORNER.as_bytes()).unwrap();
    for _ in 0..width {
        stdout.write(HORZ_BOUNDARY.as_bytes()).unwrap();
    }
    stdout.write(TOP_RIGHT_CORNER.as_bytes()).unwrap();
    stdout.write(b"\n\r").unwrap();

    // Init border for middle cells.
    for _ in 0..(height) {
        stdout.write(VERT_BOUNDARY.as_bytes()).unwrap();

        for _ in 0..width {
            stdout.write(b" ").unwrap();
        }
        stdout.write(VERT_BOUNDARY.as_bytes()).unwrap();
        stdout.write(b"\n\r").unwrap();
    }

    // init border for bottom row
    stdout.write(BOTTOM_LEFT_CORNER.as_bytes()).unwrap();
    for _ in 0..width {
        stdout.write(HORZ_BOUNDARY.as_bytes()).unwrap();
    }
    stdout.write(BOTTOM_RIGHT_CORNER.as_bytes()).unwrap();
}
