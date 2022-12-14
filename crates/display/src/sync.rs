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

//! Converts a GameState into GameView updates

use adapters::response_builder::ResponseBuilder;
use anyhow::Result;
use data::game::GameState;
use data::primitives::{RoomId, Side};
use protos::spelldawn::{
    ActionTrackerView, CardView, GameView, ManaView, PlayerInfo, PlayerView, ScoreView,
};
use rules::mana::ManaPurpose;
use rules::{flags, mana};
use {adapters, assets};

use crate::{card_sync, interface, positions};

pub fn run(builder: &mut ResponseBuilder, game: &GameState) -> Result<()> {
    let cards: Result<Vec<CardView>> = game
        .all_cards()
        .filter(|c| !c.position().shuffled_into_deck())
        .flat_map(|c| {
            let mut cards = card_sync::activated_ability_cards(builder, game, c);
            cards.push(card_sync::card_view(builder, game, c));
            cards
        })
        .collect();

    builder.push_game_view(GameView {
        user: Some(player_view(game, builder.user_side)?),
        opponent: Some(player_view(game, builder.user_side.opponent())?),
        cards: cards?,
        raid_active: game.data.raid.is_some(),
        game_object_positions: Some(positions::game_object_positions(builder, game)?),
        main_controls: if builder.state.is_final_update {
            // Only include controls on final update to ensure interface doesn't show
            // previous UI after click.
            interface::render(game, builder.user_side)?
        } else {
            None
        },
    });

    Ok(())
}

fn player_view(game: &GameState, side: Side) -> Result<PlayerView> {
    let identity = game.card(game.first_identity(side)?);
    Ok(PlayerView {
        side: adapters::player_side(side),
        player_info: Some(PlayerInfo {
            name: Some(identity.name.displayed_name()),
            portrait: Some(adapters::sprite(&rules::get(identity.name).image)),
            portrait_frame: Some(assets::identity_card_frame(side)),
            valid_rooms_to_visit: match side {
                Side::Overlord => enum_iterator::all::<RoomId>()
                    .filter(|room_id| flags::can_take_level_up_room_action(game, side, *room_id))
                    .map(adapters::room_identifier)
                    .collect(),
                Side::Champion => enum_iterator::all::<RoomId>()
                    .filter(|room_id| flags::can_take_initiate_raid_action(game, side, *room_id))
                    .map(adapters::room_identifier)
                    .collect(),
            },
            card_back: Some(assets::card_back(rules::get(identity.name).school)),
        }),
        score: Some(ScoreView { score: game.player(side).score }),
        mana: Some(ManaView {
            base_mana: mana::get(game, side, ManaPurpose::BaseMana),
            bonus_mana: mana::get(game, side, ManaPurpose::BonusForDisplay),
        }),
        action_tracker: Some(ActionTrackerView {
            available_action_count: game.player(side).actions,
        }),
        can_take_action: actions::can_take_action(game, side),
    })
}
