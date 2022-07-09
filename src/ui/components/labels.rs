/**
 * MIT License
 *
 * tui-realm - Copyright (C) 2021 Christian Visintin
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use super::Msg;
use crate::config::Settings;
use tui_realm_stdlib::Label;
use tuirealm::event::NoUserEvent;
use tuirealm::props::{Alignment, Color, TextModifiers};
use tuirealm::{Component, Event, MockComponent};

/// ## Label
///
/// Simple label component; just renders a text
/// NOTE: since I need just one label, I'm not going to use different object; I will directly implement Component for Label.
/// This is not ideal actually and in a real app you should differentiate Mock Components from Application Components.

#[derive(MockComponent)]
pub struct LabelHelp {
    component: Label,
}

impl LabelHelp {
    pub fn new(config: &Settings, text: &str) -> Self {
        Self {
            component: Label::default()
                .text(text)
                .alignment(Alignment::Left)
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Reset),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::Cyan),
                )
                .modifiers(TextModifiers::BOLD),
        }
    }
}

impl Component<Msg, NoUserEvent> for LabelHelp {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None
    }
}
