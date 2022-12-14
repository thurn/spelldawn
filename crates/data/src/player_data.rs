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

use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use with_error::WithError;

use crate::adventure::AdventureState;
use crate::card_name::CardName;
use crate::deck::Deck;
use crate::player_name::PlayerId;
use crate::primitives::{DeckId, DeckIndex, GameId};
use crate::tutorial::TutorialData;

/// Data for a player's request to create a new game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewGameRequest {
    pub deck_id: DeckIndex,
}

/// Represents the state of a game the player is participating in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerState {
    /// The player has initiated a request to create a game
    RequestedGame(NewGameRequest),
    /// The player is currently playing in the [GameId] game.
    Playing(GameId),
}

/// Represents a player's stored data.
///
/// For a player's state *within a given game* see `PlayerState`.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    /// Unique identifier for this player
    pub id: PlayerId,
    /// Identifies the game this player is currently participating in, if any.
    pub state: Option<PlayerState>,
    /// This player's saved decks.
    pub decks: Vec<Deck>,
    /// State for an ongoing adventure, if any
    pub adventure: Option<AdventureState>,
    /// Cards owned by this player
    #[serde_as(as = "Vec<(_, _)>")]
    pub collection: HashMap<CardName, u32>,
    /// Data related to this player's tutorial progress
    pub tutorial: TutorialData,
}

impl PlayerData {
    pub fn new(id: PlayerId) -> Self {
        Self {
            id,
            state: None,
            decks: vec![],
            adventure: None,
            collection: HashMap::default(),
            tutorial: TutorialData::default(),
        }
    }

    /// Returns the active [AdventureState] when one is expected to exist
    pub fn adventure(&self) -> Result<&AdventureState> {
        self.adventure.as_ref().with_error(|| "Expected active adventure")
    }

    /// Mutable equivalent of [Self::adventure]
    pub fn adventure_mut(&mut self) -> Result<&mut AdventureState> {
        self.adventure.as_mut().with_error(|| "Expected active adventure")
    }

    /// Returns the [DeckIndex] this player requested to use for a new game.
    pub fn requested_deck_id(&self) -> Option<DeckIndex> {
        match &self.state {
            Some(PlayerState::RequestedGame(request)) => Some(request.deck_id),
            _ => None,
        }
    }

    /// Retrieves one of a player's decks based on its [DeckIndex].
    pub fn deck(&self, deck_id: DeckIndex) -> Result<&Deck> {
        self.decks.get(deck_id.value as usize).with_error(|| "Deck not found")
    }

    /// Retrieves one of a player's decks based on its [DeckIndex].
    pub fn find_deck(&self, deck_id: DeckId) -> Result<&Deck> {
        Ok(match deck_id {
            DeckId::Adventure => &self.adventure()?.deck,
        })
    }

    pub fn deck_mut(&mut self, deck_id: DeckIndex) -> Result<&mut Deck> {
        self.decks.get_mut(deck_id.value as usize).with_error(|| "Deck not found")
    }
}

/// Returns the [GameId] an optional [PlayerData] is currently playing in, if
/// any.
pub fn current_game_id(data: Option<PlayerData>) -> Option<GameId> {
    match data.as_ref().and_then(|player| player.state.as_ref()) {
        Some(PlayerState::Playing(id)) => Some(*id),
        _ => None,
    }
}
