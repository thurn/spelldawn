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

use anyhow::Result;
use core_ui::prelude::*;
use data::adventure::{TileEntity, TilePosition};
use data::player_data::PlayerData;
use protos::spelldawn::Node;
use with_error::{fail, WithError};

use crate::explore_panel::ExplorePanel;

/// Renders a panel for the entity at the provided [TilePosition].
pub fn render(position: TilePosition, player: &PlayerData) -> Result<Option<Node>> {
    let Some(adventure) = &player.adventure else {
        fail!("Expected active adventure");
    };

    let tile = adventure.tiles.get(&position).with_error(|| "Tile not found")?;

    Ok(match tile.entity.with_error(|| "Expected entity")? {
        TileEntity::Draft => None,
        TileEntity::Explore => ExplorePanel {}.build(),
    })
}