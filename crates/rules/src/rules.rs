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

//! All primary game rules, responses to user actions, and associated helpers

use std::collections::HashMap;

use dashmap::DashSet;
use data::card_definition::{Ability, CardDefinition};
use data::card_name::CardName;
use data::game::GameState;
use data::primitives::{AbilityId, CardId};
use once_cell::sync::Lazy;

pub mod card_prompt;
pub mod constants;
pub mod dispatch;
pub mod flags;
pub mod mana;
pub mod mutations;
pub mod queries;

pub static DEFINITIONS: Lazy<DashSet<fn() -> CardDefinition>> = Lazy::new(DashSet::new);

/// Contains [CardDefinition]s for all known cards, keyed by [CardName]
static CARDS: Lazy<HashMap<CardName, CardDefinition>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for card_fn in DEFINITIONS.iter() {
        let card = card_fn();
        map.insert(card.name, card);
    }
    map
});

/// Returns an iterator over all known card definitions in an undefined order
pub fn all_cards() -> impl Iterator<Item = &'static CardDefinition> {
    assert!(CARDS.len() > 0, "Must call initialize() first!");
    CARDS.values()
}

/// Looks up the definition for a [CardName]. Panics if no such card is defined.
/// If this panics, you are probably not calling initialize::run();
pub fn get(name: CardName) -> &'static CardDefinition {
    CARDS.get(&name).unwrap_or_else(|| panic!("Must call initialize() first!"))
}

pub fn card_definition(game: &GameState, card_id: CardId) -> &'static CardDefinition {
    get(game.card(card_id).name)
}

pub fn ability_definition(game: &GameState, ability_id: AbilityId) -> &'static Ability {
    card_definition(game, ability_id.card_id).ability(ability_id.index)
}
