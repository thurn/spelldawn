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

//! Shows an initial introduction screen to the deck editor window

use core_ui::button::{Button, ButtonType};
use core_ui::full_screen_loading::FullScreenLoading;
use core_ui::prelude::*;
use core_ui::prompt_panel::PromptPanel;
use core_ui::{panels, style};
use data::player_data::PlayerData;
use panel_address::{CollectionBrowserFilters, DeckEditorData, Panel, PanelAddress};
use screen_overlay::ScreenOverlay;

pub struct DeckEditorPromptPanel<'a> {
    pub player: &'a PlayerData,
}

impl<'a> Panel for DeckEditorPromptPanel<'a> {
    fn address(&self) -> PanelAddress {
        PanelAddress::DeckEditorPrompt
    }

    fn screen_overlay(&self) -> Option<Node> {
        ScreenOverlay::new(self.player).show_deck_button(false).build()
    }
}

impl<'a> Component for DeckEditorPromptPanel<'a> {
    fn build(self) -> Option<Node> {
        PromptPanel::new()
            .image(style::sprite(
                "TPR/EnvironmentsHQ/Castles, Towers & Keeps/Images/Library/SceneryLibrary_inside_1",
            ))
            .prompt(
                "Retiring to the library lets you freely reconfigure the cards in your deck",
            )
            .buttons(vec![
                Button::new("Continue")
                    .action(panels::transition(
                        self.address(),
                        PanelAddress::DeckEditor(DeckEditorData {
                            collection_filters: CollectionBrowserFilters { offset: 0 },
                        }),
                        FullScreenLoading::new(
                            "TPR/EnvironmentsHQ/Castles, Towers & Keeps/Images/Library/SceneryLibrary_inside_1",
                        ),
                    ))
                    .layout(Layout::new().margin(Edge::All, 8.px())),
                Button::new("Close")
                    .button_type(ButtonType::Secondary)
                    .action(panels::close(self.address()))
                    .layout(Layout::new().margin(Edge::All, 8.px())),
            ])
            .build()
    }
}
