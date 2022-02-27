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

//! Updates for the current phase of a raid.

use std::iter;

use anyhow::Result;
use data::actions::{
    ContinueAction, EncounterAction, Prompt, PromptAction, PromptContext, RoomActivationAction,
};
use data::card_state::CardPosition;
use data::game::{GameState, RaidPhase};
use data::primitives::{CardId, CardType, RoomId, Side};
use data::with_error::WithError;
use rand::seq::IteratorRandom;
use rand::thread_rng;

use crate::{flags, mutations, queries};

/// Updates the [RaidPhase] for the ongoing raid in the provided `game` and
/// invokes callbacks as appropriate.
pub fn set_raid_phase(game: &mut GameState, phase: RaidPhase) -> Result<()> {
    game.raid_mut()?.phase = phase;
    on_enter_raid_phase(game)
}

/// Function to apply updates for the current [RaidPhase] of the provided
/// `game`.
fn on_enter_raid_phase(game: &mut GameState) -> Result<()> {
    match game.raid()?.phase {
        RaidPhase::Activation => {}
        RaidPhase::Encounter(defender_index) => {
            if can_reveal_defender(game, defender_index) {
                let defender_id = find_defender(game, game.raid()?.target, defender_index)?;
                let cost = queries::mana_cost(game, defender_id).with_error(|| "Expected cost")?;
                mutations::spend_mana(game, Side::Overlord, cost);
                mutations::set_revealed_to(game, defender_id, Side::Champion, true);
            }
        }
        RaidPhase::Continue(_) => {}
        RaidPhase::Access => {
            game.raid_mut()?.accessed = accessed_cards(game)?;
        }
    }

    set_raid_prompt(game)
}

/// Returns true if the raid defender at `defender_index` is not *currently*
/// revealed to the Champion player, but can be revealed automatically by paying
/// its mana cost.
///
/// Panics if there is no active raid or if this is an invalid defender index.
pub fn can_reveal_defender(game: &GameState, defender_index: usize) -> bool {
    let raid = game.raid().expect("Active Raid");
    let defender_id = find_defender(game, raid.target, defender_index).expect("Defender");
    raid.room_active
        && !game.card(defender_id).is_revealed_to(Side::Champion)
        && matches!(queries::mana_cost(game, defender_id),
            Some(cost)
            if cost <= game.player(Side::Overlord).mana
        )
}

/// Returns a vector of the cards accessed for the current raid target, mutating
/// the [GameState] to store the results of random zone selections.
fn accessed_cards(game: &mut GameState) -> Result<Vec<CardId>> {
    let target = game.raid()?.target;

    let accessed = match target {
        RoomId::Vault => mutations::realize_top_of_deck(
            game,
            Side::Overlord,
            queries::vault_access_count(game),
            true, /* push_updates */
        ),
        RoomId::Sanctum => {
            let count = queries::sanctum_access_count(game);
            if game.data.config.deterministic {
                game.hand(Side::Overlord).map(|c| c.id).take(count).collect()
            } else {
                game.hand(Side::Overlord).map(|c| c.id).choose_multiple(&mut thread_rng(), count)
            }
        }
        RoomId::Crypts => game
            .card_list_for_position(Side::Overlord, CardPosition::DiscardPile(Side::Overlord))
            .iter()
            .filter(|c| !c.is_revealed_to(Side::Champion))
            .map(|c| c.id)
            .collect(),
        _ => game.occupants(target).map(|c| c.id).collect(),
    };

    for card_id in &accessed {
        mutations::set_revealed_to(game, *card_id, Side::Champion, true);
    }

    Ok(accessed)
}

/// Sets a UI [Prompt] for the current raid state of the provided `game`.
///
/// Only one player at a time receives a prompt, while their opponent sees a
/// 'waiting' indicator.
pub fn set_raid_prompt(game: &mut GameState) -> Result<()> {
    let (active_player, prompt) = match game.raid()?.phase {
        RaidPhase::Activation => (Side::Overlord, build_activation_prompt()),
        RaidPhase::Encounter(defender) => (Side::Champion, build_encounter_prompt(game, defender)?),
        RaidPhase::Continue(_) => (Side::Champion, build_continue_prompt()),
        RaidPhase::Access => (Side::Champion, build_access_prompt(game)?),
    };

    mutations::set_prompt(game, active_player, prompt);
    Ok(())
}

fn build_activation_prompt() -> Prompt {
    Prompt {
        context: Some(PromptContext::ActivateRoom),
        responses: vec![
            PromptAction::ActivateRoomAction(RoomActivationAction::Activate),
            PromptAction::ActivateRoomAction(RoomActivationAction::Pass),
        ],
    }
}

fn build_encounter_prompt(game: &GameState, defender: usize) -> Result<Prompt> {
    let defender_id = find_defender(game, game.raid()?.target, defender)?;
    Ok(Prompt {
        context: None,
        responses: game
            .weapons()
            .filter(|weapon| flags::can_defeat_target(game, weapon.id, defender_id))
            .map(|weapon| {
                PromptAction::EncounterAction(EncounterAction::UseWeaponAbility(
                    weapon.id,
                    defender_id,
                ))
            })
            .chain(iter::once(PromptAction::EncounterAction(EncounterAction::NoWeapon)))
            .collect(),
    })
}

fn build_continue_prompt() -> Prompt {
    Prompt {
        context: Some(PromptContext::RaidAdvance),
        responses: vec![
            PromptAction::ContinueAction(ContinueAction::Advance),
            PromptAction::ContinueAction(ContinueAction::Retreat),
        ],
    }
}

fn build_access_prompt(game: &GameState) -> Result<Prompt> {
    Ok(Prompt {
        context: None,
        responses: game
            .raid()?
            .accessed
            .iter()
            .filter_map(|card_id| access_prompt_for_card(game, *card_id))
            .chain(iter::once(PromptAction::EndRaid))
            .collect(),
    })
}

/// Returns a [PromptAction] for the Champion to access the provided `card_id`,
/// if any action can be taken.
fn access_prompt_for_card(game: &GameState, card_id: CardId) -> Option<PromptAction> {
    let definition = crate::card_definition(game, card_id);
    match definition.card_type {
        CardType::Scheme if flags::can_score_card(game, Side::Champion, card_id) => {
            Some(PromptAction::RaidScoreCard(card_id))
        }
        CardType::Project | CardType::Upgrade
            if flags::can_destroy_accessed_card(game, card_id) =>
        {
            Some(PromptAction::RaidDestroyCard(card_id))
        }
        _ => None,
    }
}

/// Finds the defending [CardId] at the given `index` position in the indicated
/// `room_id`.
pub fn find_defender(game: &GameState, room_id: RoomId, index: usize) -> Result<CardId> {
    Ok(game.defender_list(room_id).get(index).with_error(|| "Defender Not Found")?.id)
}