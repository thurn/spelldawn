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

//! Panel rendering. A 'panel' is a discrete rectangular piece of UI which can
//! be opened or closed by the user, such as a game menu or window.

use adventure_display::adventure_panels;
use adventure_display::shop_panel::ShopPanel;
use anyhow::Result;
use data::adventure::AdventureState;
use data::player_data::PlayerData;
use deck_editor::deck_editor_panel::DeckEditorPanel;
use deck_editor::deck_editor_prompt::DeckEditorPromptPanel;
use old_deck_editor::deck_editor_panel::OldDeckEditorPanel;
use old_deck_editor::pick_deck_name::PickDeckName;
use old_deck_editor::pick_deck_school::PickDeckSchool;
use old_deck_editor::pick_deck_side::PickDeckSide;
use panel_address::{CreateDeckState, Panel, PanelAddress};
use panels::about_panel::AboutPanel;
use panels::adventure_menu::AdventureMenu;
use panels::debug_panel::DebugPanel;
use panels::disclaimer_panel::DisclaimerPanel;
use panels::game_menu_panel::GameMenuPanel;
use panels::game_over_panel::GameOverPanel;
use panels::loading_panel::LoadingPanel;
use panels::main_menu_panel::MainMenuPanel;
use panels::set_player_name_panel::SetPlayerNamePanel;
use panels::settings_panel::SettingsPanel;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{InterfacePanel, InterfacePanelAddress, UpdatePanelsCommand};
use serde_json::de;
use with_error::WithError;

pub fn main_menu_panels() -> Vec<PanelAddress> {
    vec![
        PanelAddress::MainMenu,
        PanelAddress::Settings,
        PanelAddress::About,
        PanelAddress::Disclaimer,
    ]
}

pub fn adventure_panels(adventure: &AdventureState) -> Vec<PanelAddress> {
    adventure
        .tiles
        .iter()
        .filter_map(|(position, state)| {
            state.entity.as_ref().map(|_| PanelAddress::TilePrompt(*position))
        })
        .chain(adventure.tiles.iter().filter_map(|(position, state)| {
            state.entity.as_ref().map(|_| PanelAddress::TileLoading(*position))
        }))
        .chain(vec![
            PanelAddress::AdventureMenu,
            PanelAddress::Settings,
            PanelAddress::DeckEditorPrompt,
            PanelAddress::DeckEditorLoading,
        ])
        .collect()
}

pub fn render_panels(
    commands: &mut Vec<Command>,
    player: &PlayerData,
    addresses: Vec<PanelAddress>,
) -> Result<()> {
    for address in addresses {
        commands.push(Command::UpdatePanels(render_panel(player, address.into())?));
    }
    Ok(())
}

pub fn render_panel(
    player: &PlayerData,
    client_address: InterfacePanelAddress,
) -> Result<UpdatePanelsCommand> {
    let server_address =
        de::from_slice(&client_address.serialized).with_error(|| "deserialization failed")?;
    let panel = render_server_panel(player, server_address)?;
    Ok(UpdatePanelsCommand { panels: panel.map_or_else(Vec::new, |p| vec![p]) })
}

fn render_server_panel(
    player: &PlayerData,
    server_address: PanelAddress,
) -> Result<Option<InterfacePanel>> {
    Ok(match server_address {
        PanelAddress::MainMenu => MainMenuPanel::new().build_panel(),
        PanelAddress::About => AboutPanel::new().build_panel(),
        PanelAddress::Settings => SettingsPanel::new().build_panel(),
        PanelAddress::Disclaimer => DisclaimerPanel::new().build_panel(),
        PanelAddress::DebugPanel => DebugPanel::new().build_panel(),
        PanelAddress::GameMenu => GameMenuPanel::new().build_panel(),
        PanelAddress::AdventureMenu => AdventureMenu::new().build_panel(),
        PanelAddress::SetPlayerName(side) => SetPlayerNamePanel::new(side).build_panel(),
        PanelAddress::DeckEditorLoading => LoadingPanel::new(
            server_address,
            "TPR/EnvironmentsHQ/Castles, Towers & Keeps/Images/Library/SceneryLibrary_inside_1",
        )
        .build_panel(),
        PanelAddress::DeckEditorPrompt => DeckEditorPromptPanel { player }.build_panel(),
        PanelAddress::DeckEditor(data) => {
            DeckEditorPanel { player, data, deck: player.find_deck(data.deck_id)? }.build_panel()
        }
        PanelAddress::OldDeckEditor(data) => {
            let open_deck = if let Some(id) = data.deck { Some(player.deck(id)?) } else { None };
            OldDeckEditorPanel { player, open_deck, data }.build_panel()
        }
        PanelAddress::CreateDeck(state) => match state {
            CreateDeckState::PickSide => PickDeckSide::new().build_panel(),
            CreateDeckState::PickSchool(side) => PickDeckSchool::new(side).build_panel(),
            CreateDeckState::PickName(side, school) => {
                PickDeckName::new(side, school).build_panel()
            }
        },
        PanelAddress::GameOver(data) => GameOverPanel { data, player }.build_panel(),
        PanelAddress::TileLoading(position) => {
            adventure_panels::render_tile_loading_panel(position, player)?
        }
        PanelAddress::TilePrompt(position) => {
            adventure_panels::render_tile_prompt_panel(position, player)?
        }
        PanelAddress::DraftCard => render_adventure_choice(player)?,
        PanelAddress::AdventureOver => render_adventure_choice(player)?,
        PanelAddress::Shop(position) => ShopPanel::new(player, position)?.build_panel(),
    })
}

fn render_adventure_choice(player: &PlayerData) -> Result<Option<InterfacePanel>> {
    // It's normal for the client to request screens which aren't always valid,
    // e.g. refreshing the cached choice screen after it's been removed.

    let Some(adventure) = &player.adventure else {
        return Ok(None)
    };

    let Some(choice_screen) = &adventure.choice_screen else {
        return Ok(None)
    };

    let rendered = adventure_display::render_adventure_choice_screen(adventure, choice_screen)?;

    Ok(rendered.panel)
}
