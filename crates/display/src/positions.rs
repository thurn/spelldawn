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

use adapters;
use adapters::response_builder::ResponseBuilder;
use anyhow::Result;
use data::card_state::{CardPosition, CardState};
use data::game::{GamePhase, GameState, MulliganData, RaidData};
use data::game_actions::CardTarget;
use data::primitives::{AbilityId, CardId, GameObjectId, ItemLocation, RoomId, RoomLocation, Side};
use data::utils;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{
    ClientItemLocation, ClientRoomLocation, GameObjectPositions, ObjectPosition,
    ObjectPositionBrowser, ObjectPositionDeck, ObjectPositionDeckContainer,
    ObjectPositionDiscardPile, ObjectPositionDiscardPileContainer, ObjectPositionHand,
    ObjectPositionIdentity, ObjectPositionIdentityContainer, ObjectPositionIntoCard,
    ObjectPositionItem, ObjectPositionRaid, ObjectPositionRevealedCards, ObjectPositionRoom,
    ObjectPositionStaging, RevealedCardsBrowserSize, RoomIdentifier,
};
use raids::traits::RaidDisplayState;
use raids::RaidDataExt;
use rules::queries;
use with_error::fail;

pub const RELEASE_SORTING_KEY: u32 = 100;

pub fn for_card(card: &CardState, position: Position) -> ObjectPosition {
    ObjectPosition {
        position: Some(position),
        sorting_key: 1 + card.sorting_key,
        sorting_subkey: 0,
    }
}

pub fn for_ability(game: &GameState, ability_id: AbilityId, position: Position) -> ObjectPosition {
    ObjectPosition {
        position: Some(position),
        sorting_key: 1 + game.card(ability_id.card_id).sorting_key,
        sorting_subkey: 1 + (ability_id.index.value() as u32),
    }
}

pub fn for_sorting_key(sorting_key: u32, position: Position) -> ObjectPosition {
    ObjectPosition { sorting_key: 1 + sorting_key, sorting_subkey: 0, position: Some(position) }
}

pub fn room(room_id: RoomId, location: RoomLocation) -> Position {
    Position::Room(ObjectPositionRoom {
        room_id: adapters::room_identifier(room_id),
        room_location: match location {
            RoomLocation::Defender => ClientRoomLocation::Front,
            RoomLocation::Occupant => ClientRoomLocation::Back,
        }
        .into(),
    })
}

pub fn unspecified_room(location: RoomLocation) -> Position {
    Position::Room(ObjectPositionRoom {
        room_id: RoomIdentifier::Unspecified as i32,
        room_location: match location {
            RoomLocation::Defender => ClientRoomLocation::Front,
            RoomLocation::Occupant => ClientRoomLocation::Back,
        }
        .into(),
    })
}

pub fn item(location: ItemLocation) -> Position {
    Position::Item(ObjectPositionItem {
        item_location: match location {
            ItemLocation::Weapons => ClientItemLocation::Left,
            ItemLocation::Artifacts => ClientItemLocation::Right,
        }
        .into(),
    })
}

pub fn hand(builder: &ResponseBuilder, side: Side) -> Position {
    Position::Hand(ObjectPositionHand { owner: builder.to_player_name(side) })
}

pub fn deck(builder: &ResponseBuilder, side: Side) -> Position {
    Position::Deck(ObjectPositionDeck { owner: builder.to_player_name(side) })
}

pub fn deck_container(builder: &ResponseBuilder, side: Side) -> Position {
    Position::DeckContainer(ObjectPositionDeckContainer { owner: builder.to_player_name(side) })
}

pub fn discard(builder: &ResponseBuilder, side: Side) -> Position {
    Position::DiscardPile(ObjectPositionDiscardPile { owner: builder.to_player_name(side) })
}

pub fn discard_container(builder: &ResponseBuilder, side: Side) -> Position {
    Position::DiscardPileContainer(ObjectPositionDiscardPileContainer {
        owner: builder.to_player_name(side),
    })
}

pub fn identity(builder: &ResponseBuilder, side: Side) -> Position {
    Position::Identity(ObjectPositionIdentity { owner: builder.to_player_name(side) })
}

pub fn identity_container(builder: &ResponseBuilder, side: Side) -> Position {
    Position::IdentityContainer(ObjectPositionIdentityContainer {
        owner: builder.to_player_name(side),
    })
}

pub fn staging() -> Position {
    Position::Staging(ObjectPositionStaging {})
}

pub fn browser() -> Position {
    Position::Browser(ObjectPositionBrowser {})
}

pub fn revealed_cards(large: bool) -> Position {
    Position::Revealed(ObjectPositionRevealedCards {
        size: if large { RevealedCardsBrowserSize::Large } else { RevealedCardsBrowserSize::Small }
            as i32,
    })
}

pub fn raid() -> Position {
    Position::Raid(ObjectPositionRaid {})
}

pub fn parent_card(ability_id: AbilityId) -> Position {
    Position::IntoCard(ObjectPositionIntoCard {
        card_id: Some(adapters::card_identifier(ability_id.card_id)),
    })
}

pub fn convert(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
) -> Result<ObjectPosition> {
    Ok(if let Some(position_override) = position_override(builder, game, card)? {
        position_override
    } else {
        ObjectPosition {
            sorting_key: card.sorting_key,
            position: Some(adapt_position(builder, game, card.id, card.position())?),
            ..ObjectPosition::default()
        }
    })
}

fn adapt_position(
    builder: &ResponseBuilder,
    game: &GameState,
    card_id: CardId,
    position: CardPosition,
) -> Result<Position> {
    Ok(match position {
        CardPosition::Room(room_id, location) => room(room_id, location),
        CardPosition::ArenaItem(location) => item(location),
        CardPosition::Hand(side) => hand(builder, side),
        CardPosition::DeckTop(side) => deck(builder, side),
        CardPosition::DiscardPile(side) => discard(builder, side),
        CardPosition::Scored(side) | CardPosition::Identity(side) => identity(builder, side),
        CardPosition::Scoring => staging(),
        CardPosition::Played(side, target) => {
            card_release_position(builder, game, side, card_id, target)?
        }
        CardPosition::DeckUnknown(_) => fail!("Invalid card position"),
    })
}

/// Calculates the position of a card after it has been played.
///
/// For cards that are played by the opponent, we animate them to the staging
/// area. We also animate spell cards to staging while resolving their effects.
/// For other card types, we move them directly to their destination to make
/// playing a card feel more responsive.
pub fn card_release_position(
    builder: &ResponseBuilder,
    game: &GameState,
    side: Side,
    card_id: CardId,
    target: CardTarget,
) -> Result<Position> {
    if builder.user_side != side || rules::card_definition(game, card_id).card_type.is_spell() {
        Ok(staging())
    } else {
        adapt_position(
            builder,
            game,
            card_id,
            queries::played_position(game, side, card_id, target)?,
        )
    }
}

pub fn ability_card_position(
    builder: &ResponseBuilder,
    game: &GameState,
    ability_id: AbilityId,
) -> ObjectPosition {
    for_ability(
        game,
        ability_id,
        if utils::is_true(|| Some(game.ability_state.get(&ability_id)?.currently_resolving)) {
            staging()
        } else {
            hand(builder, ability_id.side())
        },
    )
}

pub fn game_object_positions(
    builder: &ResponseBuilder,
    game: &GameState,
) -> Result<GameObjectPositions> {
    let (side, opponent) = (builder.user_side, builder.user_side.opponent());
    Ok(GameObjectPositions {
        user_deck: Some(non_card(builder, game, GameObjectId::Deck(side))?),
        opponent_deck: Some(non_card(builder, game, GameObjectId::Deck(opponent))?),
        user_identity: Some(non_card(builder, game, GameObjectId::Identity(side))?),
        opponent_identity: Some(non_card(builder, game, GameObjectId::Identity(opponent))?),
        user_discard: Some(non_card(builder, game, GameObjectId::DiscardPile(side))?),
        opponent_discard: Some(non_card(builder, game, GameObjectId::DiscardPile(opponent))?),
    })
}

fn non_card(
    builder: &ResponseBuilder,
    game: &GameState,
    id: GameObjectId,
) -> Result<ObjectPosition> {
    Ok(if let Some(position_override) = raid_position_override(game, id)? {
        position_override
    } else {
        match id {
            GameObjectId::Deck(side) => for_sorting_key(0, deck_container(builder, side)),
            GameObjectId::DiscardPile(side) => for_sorting_key(0, discard_container(builder, side)),
            GameObjectId::Identity(side) => for_sorting_key(0, identity_container(builder, side)),
            _ => fail!("Unsupported ID type"),
        }
    })
}

fn position_override(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
) -> Result<Option<ObjectPosition>> {
    match &game.data.phase {
        GamePhase::ResolveMulligans(mulligans) => {
            Ok(opening_hand_position_override(builder, game, card, mulligans))
        }
        GamePhase::Play => raid_position_override(game, card.id.into()),
        _ => Ok(None),
    }
}

fn raid_position_override(game: &GameState, id: GameObjectId) -> Result<Option<ObjectPosition>> {
    Ok(if let Some(raid_data) = &game.data.raid {
        match raid_data.phase().display_state(game)? {
            RaidDisplayState::None => None,
            RaidDisplayState::Defenders(defenders) => {
                browser_position(id, raid(), raid_browser(game, raid_data, defenders))
            }
            RaidDisplayState::Access => {
                browser_position(id, browser(), raid_access_browser(game, raid_data))
            }
        }
    } else {
        None
    })
}

fn opening_hand_position_override(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
    data: &MulliganData,
) -> Option<ObjectPosition> {
    if data.decision(builder.user_side).is_none()
        && game.hand(builder.user_side).any(|c| c.id == card.id)
    {
        Some(for_card(card, revealed_cards(true)))
    } else {
        None
    }
}

fn browser_position(
    id: GameObjectId,
    position: Position,
    browser: Vec<GameObjectId>,
) -> Option<ObjectPosition> {
    browser.iter().position(|gid| *gid == id).map(|index| ObjectPosition {
        sorting_key: index as u32,
        sorting_subkey: 0,
        position: Some(position),
    })
}

fn raid_browser(game: &GameState, raid: &RaidData, defenders: Vec<CardId>) -> Vec<GameObjectId> {
    let mut result = Vec::new();

    match raid.target {
        RoomId::Vault => {
            result.push(GameObjectId::Deck(Side::Overlord));
        }
        RoomId::Sanctum => {
            result.push(GameObjectId::Identity(Side::Overlord));
        }
        RoomId::Crypts => {
            result.push(GameObjectId::DiscardPile(Side::Overlord));
        }
        _ => {}
    }

    result.extend(game.occupants(raid.target).map(|card| GameObjectId::CardId(card.id)));
    result.extend(defenders.iter().map(|card_id| GameObjectId::CardId(*card_id)));
    result.push(GameObjectId::Identity(Side::Champion));
    result
}

fn raid_access_browser(game: &GameState, raid: &RaidData) -> Vec<GameObjectId> {
    match raid.target {
        RoomId::Sanctum => {
            game.hand(Side::Overlord).map(|card| GameObjectId::CardId(card.id)).collect()
        }
        RoomId::Crypts => {
            game.discard_pile(Side::Overlord).map(|card| GameObjectId::CardId(card.id)).collect()
        }
        _ => raid.accessed.iter().map(|card_id| GameObjectId::CardId(*card_id)).collect(),
    }
}
