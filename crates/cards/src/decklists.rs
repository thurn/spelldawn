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

use actions;
use anyhow::Result;
use data::card_name::CardName;
use data::deck::Deck;
use data::game::{GameConfiguration, GameState, MulliganDecision};
use data::game_actions::{GameAction, PromptAction};
use data::player_name::{NamedPlayer, PlayerId};
use data::primitives::{DeckIndex, GameId, Side};
use maplit::hashmap;
use once_cell::sync::Lazy;
use rules::{dispatch, mutations};

/// Empty Overlord deck for use in tests
pub static EMPTY_OVERLORD: Lazy<Deck> = Lazy::new(|| Deck {
    index: DeckIndex { value: 0 },
    name: "Overlord Empty".to_string(),
    owner_id: PlayerId::Named(NamedPlayer::TestNoAction),
    side: Side::Overlord,
    identity: CardName::TestOverlordIdentity,
    cards: HashMap::new(),
});

/// Standard Overlord deck for use in tests
pub static CANONICAL_OVERLORD: Lazy<Deck> = Lazy::new(|| Deck {
    index: DeckIndex { value: 0 },
    name: "Overlord Starter".to_string(),
    owner_id: PlayerId::Named(NamedPlayer::TestNoAction),
    side: Side::Overlord,
    identity: CardName::TestOverlordIdentity,
    cards: hashmap! {
        CardName::GoldMine => 3,
        CardName::ActivateReinforcements => 2,
        CardName::ResearchProject => 2,
        CardName::Gemcarver => 2,
        CardName::Coinery => 2,
        CardName::SpikeTrap => 2,
        CardName::OverwhelmingPower => 2,
        CardName::GatheringDark => 3,
        CardName::ForcedMarch => 2,
        CardName::TimeGolem => 1,
        CardName::TemporalStalker => 2,
        CardName::ShadowLurker => 3,
        CardName::SphinxOfWintersBreath => 2,
        CardName::BridgeTroll => 2,
        CardName::Stormcaller => 2,
        CardName::FireGoblin => 2
    },
});

/// Empty Champion deck for use in tests
pub static EMPTY_CHAMPION: Lazy<Deck> = Lazy::new(|| Deck {
    index: DeckIndex { value: 0 },
    name: "Champion Empty".to_string(),
    owner_id: PlayerId::Named(NamedPlayer::TestNoAction),
    side: Side::Champion,
    identity: CardName::TestChampionIdentity,
    cards: HashMap::new(),
});

/// Standard Champion deck for use in tests
pub static CANONICAL_CHAMPION: Lazy<Deck> = Lazy::new(|| Deck {
    index: DeckIndex { value: 1 },
    name: "Champion Starter".to_string(),
    owner_id: PlayerId::Named(NamedPlayer::TestNoAction),
    side: Side::Champion,
    identity: CardName::TestChampionIdentity,
    cards: hashmap! {
        CardName::Meditation => 2,
        CardName::CoupDeGrace => 3,
        CardName::ChargedStrike => 2,
        CardName::ArcaneRecovery => 3,
        CardName::StealthMission => 2,
        CardName::Preparation => 2,
        CardName::InvisibilityRing => 1,
        CardName::Accumulator => 1,
        CardName::MageGloves => 1,
        CardName::SkysReach => 2,
        CardName::MagicalResonator => 2,
        CardName::DarkGrimoire => 1,
        CardName::MaraudersAxe => 2,
        CardName::KeenHalberd => 2,
        CardName::EtherealBlade => 2,
        CardName::BowOfTheAlliance => 2
    },
});

/// Returns a canonical deck associated with the given [PlayerId].
pub fn canonical_deck(player_id: PlayerId, side: Side) -> Deck {
    if side == Side::Champion {
        Deck { owner_id: player_id, ..CANONICAL_CHAMPION.clone() }
    } else {
        Deck { owner_id: player_id, ..CANONICAL_OVERLORD.clone() }
    }
}

/// Creates a new deterministic game using the canonical decklists, deals
/// opening hands and resolves mulligans.
pub fn canonical_game() -> Result<GameState> {
    let mut game = GameState::new(
        GameId::new(0),
        CANONICAL_OVERLORD.clone(),
        CANONICAL_CHAMPION.clone(),
        GameConfiguration { deterministic: true, simulation: true },
    );

    dispatch::populate_delegate_cache(&mut game);
    mutations::deal_opening_hands(&mut game)?;
    actions::handle_game_action(
        &mut game,
        Side::Overlord,
        GameAction::PromptAction(PromptAction::MulliganDecision(MulliganDecision::Keep)),
    )?;
    actions::handle_game_action(
        &mut game,
        Side::Champion,
        GameAction::PromptAction(PromptAction::MulliganDecision(MulliganDecision::Keep)),
    )?;

    Ok(game)
}

/// Looks up the [Deck] for a named player.
pub fn deck_for_player(player: NamedPlayer, side: Side) -> Deck {
    // (Eventually different named players will have different decks)
    let id = PlayerId::Named(player);
    canonical_deck(id, side)
}
