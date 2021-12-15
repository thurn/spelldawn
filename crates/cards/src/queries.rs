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

use crate::dispatch;
use model::card_definition::CardStats;
use model::delegates;
use model::game::GameState;
use model::primitives::{AttackValue, BoostCount, CardId, HealthValue};

/// Obtain the [CardStats] for a given card
pub fn stats(game: &GameState, card_id: impl Into<CardId>) -> &CardStats {
    &crate::get(game.card(card_id).name).config.stats
}

pub fn attack(game: &GameState, card_id: impl Into<CardId> + Copy) -> AttackValue {
    dispatch::perform_query(
        game,
        delegates::get_attack_value,
        card_id.into(),
        stats(game, card_id).base_attack.unwrap_or(0),
    )
}

pub fn health(game: &GameState, card_id: impl Into<CardId> + Copy) -> HealthValue {
    dispatch::perform_query(
        game,
        delegates::get_health_value,
        card_id.into(),
        stats(game, card_id).health.unwrap_or(0),
    )
}

pub fn boost_count(game: &GameState, card_id: impl Into<CardId> + Copy) -> BoostCount {
    dispatch::perform_query(
        game,
        delegates::get_boost_count,
        card_id.into(),
        game.card(card_id).boost_count,
    )
}