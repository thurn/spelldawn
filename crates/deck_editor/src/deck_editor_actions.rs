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
use data::player_data::PlayerData;
use data::tutorial::TutorialMessageKey;
use data::user_actions::DeckEditorAction;
use with_error::{fail, WithError};

pub fn handle(player: &mut PlayerData, action: DeckEditorAction) -> Result<()> {
    match action {
        DeckEditorAction::ViewedPrompt => {
            player.tutorial.seen.insert(TutorialMessageKey::DeckEditor);
        }
        DeckEditorAction::AddToDeck(card_name) => {
            player
                .adventure_mut()?
                .deck
                .cards
                .entry(card_name)
                .and_modify(|e| *e += 1)
                .or_insert(1);
        }
        DeckEditorAction::RemoveFromDeck(card_name) => {
            let deck = &mut player.adventure_mut()?.deck;
            let count = *deck.cards.get(&card_name).with_error(|| "Card not present")?;
            match count {
                0 => fail!("Card count is zero"),
                1 => {
                    deck.cards.remove(&card_name);
                }
                _ => {
                    deck.cards.insert(card_name, count - 1);
                }
            }
        }
    }
    Ok(())
}
