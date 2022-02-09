// use crate::song::Song;
use crate::player::GeneralP;
use crate::song::Song;
use crate::ui::{Id, Model, Msg, Status};

use if_chain::if_chain;
use std::time::Duration;
use tui_realm_stdlib::ProgressBar;
// use tuirealm::command::CmdResult;
use crate::ui::components::StyleColorSymbol;
use tuirealm::event::NoUserEvent;
use tuirealm::props::{Alignment, BorderType, Borders, Color, PropPayload, PropValue};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent};

#[derive(MockComponent)]
pub struct Progress {
    component: ProgressBar,
}

impl Progress {
    pub fn new(style_color_symbol: &StyleColorSymbol) -> Self {
        Self {
            component: ProgressBar::default()
                .borders(
                    Borders::default()
                        .color(
                            style_color_symbol
                                .progress_border()
                                .unwrap_or(Color::LightMagenta),
                        )
                        .modifiers(BorderType::Rounded),
                )
                .background(
                    style_color_symbol
                        .progress_background()
                        .unwrap_or(Color::Reset),
                )
                .foreground(
                    style_color_symbol
                        .progress_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .label("Song Name")
                .title("Playing", Alignment::Center)
                .progress(0.0),
        }
    }
}

impl Component<Msg, NoUserEvent> for Progress {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        Some(Msg::None)
    }
}

impl Model {
    pub fn progress_reload(&mut self) {
        assert!(self.app.umount(&Id::Progress).is_ok());
        assert!(self
            .app
            .mount(
                Id::Progress,
                Box::new(Progress::new(&self.config.style_color_symbol)),
                Vec::new()
            )
            .is_ok());
        self.progress_update_title();
    }

    pub fn progress_update_title(&mut self) {
        if_chain! {
            if let Some(song) = &self.current_song;
            let artist = song.artist().unwrap_or("Unknown Artist");
            let title = song.title().unwrap_or("Unknown Title");
            let progress_title = format!(
                "Playing: {:^.20} - {:^.20} | Volume: {}",
                            artist, title, self.config.volume
                );
            then {
                self.app.attr( &Id::Progress,
                    Attribute::Title,
                    AttrValue::Title((progress_title,Alignment::Center)),
                    ).ok();

            }
        }
    }

    pub fn progress_update(&mut self) {
        if let Ok((progress, time_pos, duration)) = self.player.get_progress() {
            // for unsupported file format, don't update progress
            if duration == 0 {
                return;
            }
            // below line is left for debug, for the bug of comsume 2 or more songs when start app
            // eprintln!("{},{},{}", new_prog, time_pos, duration);

            // let duration = self.progress_compare_duration(duration);
            // if time_pos >= duration {
            //     self.status = Some(Status::Stopped);
            //     return;
            // }
            if time_pos > self.time_pos && time_pos - self.time_pos < 1 {
                return;
            }

            // if time_pos == self.time_pos {
            //     self.time_pos_repeat = 1;
            //     return;
            // }

            // if self.time_pos_repeat >= 1000 {
            //     self.time_pos_repeat = 0;
            //     self.status = Some(Status::Stopped);
            //     return;
            // }
            self.time_pos = time_pos;
            let new_prog = Self::progress_safeguard(progress);
            self.progress_set(new_prog, duration);
        }
    }

    // #[allow(clippy::cast_possible_wrap)]
    // const fn progress_compare_duration(&self, duration: i64) -> i64 {
    //     if let Some(song) = &self.current_song {
    //         let duration_song = song.duration().as_secs() as i64;
    //         if duration_song > duration {
    //             return duration_song;
    //         }
    //     }
    //     duration
    // }

    fn progress_safeguard(progress: f64) -> f64 {
        let mut new_prog = progress / 100.0;
        if new_prog > 1.0 {
            new_prog = 1.0;
        }
        new_prog
    }

    fn progress_set(&mut self, progress: f64, duration: i64) {
        self.app
            .attr(
                &Id::Progress,
                Attribute::Value,
                AttrValue::Payload(PropPayload::One(PropValue::F64(progress))),
            )
            .ok();

        self.app
            .attr(
                &Id::Progress,
                Attribute::Text,
                AttrValue::String(format!(
                    "{}    -    {}",
                    Song::duration_formatted_short(&Duration::from_secs(
                        self.time_pos.try_into().unwrap_or(0)
                    )),
                    Song::duration_formatted_short(&Duration::from_secs(
                        duration.try_into().unwrap_or(0)
                    ))
                )),
            )
            .ok();
    }
}
