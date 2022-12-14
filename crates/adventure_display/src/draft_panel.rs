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

use core_ui::button::Button;
use core_ui::design::FontSize;
use core_ui::full_screen_image::FullScreenImage;
use core_ui::panels::Panels;
use core_ui::prelude::*;
use core_ui::style;
use core_ui::text::Text;
use data::adventure::DraftData;
use data::adventure_action::AdventureAction;
use deck_card::{CardHeight, DeckCard};
use panel_address::{Panel, PanelAddress};
use protos::spelldawn::FlexJustify;

pub struct DraftPanel<'a> {
    pub data: &'a DraftData,
}

impl<'a> Panel for DraftPanel<'a> {
    fn address(&self) -> PanelAddress {
        PanelAddress::DraftCard
    }
}

impl<'a> Component for DraftPanel<'a> {
    fn build(self) -> Option<Node> {
        FullScreenImage::new()
            .image(style::sprite("TPR/EnvironmentsHQ/Dungeons, Shrines & Altars/Images/MountainTomb/ScenerySnowMountain_1"))
            .content(Row::new("DraftPanel").style(Style::new().justify_content(FlexJustify::Center)).children(self.data.choices.iter().enumerate().map(
                |(i, choice)| {
                    Column::new("Choice")
                        .style(Style::new().margin(Edge::All, 32.px()))
                        .child(
                            DeckCard::new(choice.card)
                                .layout(Layout::new().margin(Edge::All, 8.px()))
                                .height(CardHeight::vh(50.0)),
                        )
                        .child(
                            Text::new(format!("{}x", choice.quantity))
                                .font_size(FontSize::Headline)
                                .layout(Layout::new().position(Edge::Top, (-8).px())),
                        )
                        .child(
                            Button::new("Pick")
                                .layout(
                                    Layout::new()
                                        .margin(Edge::Horizontal, 8.px())
                                        .margin(Edge::Top, 16.px()),
                                )
                                .action(Panels::close(self.address())
                                    .action(AdventureAction::DraftCard(i))),
                        )
                },
            )))
            .build()
    }
}
