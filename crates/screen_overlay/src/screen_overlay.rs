// Copyright © Spelldawn 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core_ui::button::{IconButton, IconButtonType};
use core_ui::design::{BackgroundColor, FontSize};
use core_ui::icons;
use core_ui::prelude::*;
use core_ui::style::Corner;
use core_ui::text::Text;
use data::player_data::PlayerData;
use protos::spelldawn::{FlexAlign, FlexJustify, FlexPosition};

#[allow(dead_code)]
pub struct ScreenOverlay<'a> {
    player: &'a PlayerData,
}

impl<'a> ScreenOverlay<'a> {
    pub fn new(player: &'a PlayerData) -> Self {
        Self { player }
    }
}

impl<'a> Component for ScreenOverlay<'a> {
    fn build(self) -> Option<Node> {
        Row::new("Navbar")
            .style(
                Style::new()
                    .position_type(FlexPosition::Absolute)
                    .position(Edge::Left, 1.safe_area_left())
                    .position(Edge::Right, 1.safe_area_right())
                    .position(Edge::Top, 1.safe_area_top())
                    .align_items(FlexAlign::FlexStart)
                    .justify_content(FlexJustify::SpaceBetween),
            )
            .child(
                Row::new("Left")
                    .style(Style::new().align_items(FlexAlign::Center))
                    .child(
                        IconButton::new(icons::BUG)
                            .button_type(IconButtonType::NavbarBlue)
                            .layout(Layout::new().margin(Edge::All, 12.px())),
                    )
                    .child(self.player.adventure.as_ref().map(|adventure| {
                        Row::new("CoinCount")
                            .style(
                                Style::new()
                                    .margin(Edge::Horizontal, 12.px())
                                    .padding(Edge::Horizontal, 8.px())
                                    .height(80.px())
                                    .background_color(BackgroundColor::CoinCountOverlay)
                                    .border_radius(Corner::All, 12.px()),
                            )
                            .child(Text::new(
                                format!(
                                    "{} <color=yellow>{}</color>",
                                    adventure.coins,
                                    icons::COINS
                                ),
                                FontSize::CoinCount,
                            ))
                    })),
            )
            .child(
                Row::new("Right")
                    .child(
                        IconButton::new(icons::DECK)
                            .button_type(IconButtonType::NavbarBrown)
                            .layout(Layout::new().margin(Edge::All, 12.px())),
                    )
                    .child(
                        IconButton::new(icons::BARS)
                            .button_type(IconButtonType::NavbarBrown)
                            .layout(Layout::new().margin(Edge::All, 12.px())),
                    ),
            )
            .build()
    }
}