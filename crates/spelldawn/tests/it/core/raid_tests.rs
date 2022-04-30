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

use data::card_name::CardName;
use data::primitives::{RoomId, Side};
use insta::assert_snapshot;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::game_object_identifier::Id;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{
    ClientRoomLocation, GainManaAction, InitiateRaidAction, ObjectPositionBrowser,
    ObjectPositionIdentity, ObjectPositionIdentityContainer, ObjectPositionRaid,
    ObjectPositionRoom, PlayerName,
};
use test_utils::client::HasText;
use test_utils::summarize::Summary;
use test_utils::{test_games, *};

#[test]
fn initiate_raid() {
    let (mut g, ids) = test_games::simple_game(
        Side::Champion,
        CardName::TestScheme31,
        CardName::TestMinionEndRaid,
        CardName::TestWeapon3Attack12Boost3Cost,
    );
    let response = g.perform_action(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    assert_eq!(1, g.me().actions());
    assert!(g.user.other_player.can_take_action());
    assert!(g.opponent.this_player.can_take_action());
    assert!(g.user.data.raid_active());
    assert!(g.opponent.data.raid_active());

    assert_eq!(
        g.user.data.object_index_position(Id::CardId(ids.scheme_id)),
        (0, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.opponent.data.object_index_position(Id::CardId(ids.scheme_id)),
        (0, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.user.data.object_index_position(Id::CardId(ids.minion_id)),
        (1, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.opponent.data.object_index_position(Id::CardId(ids.minion_id)),
        (1, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.user.data.object_index_position(Id::Identity(PlayerName::User.into())),
        (2, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.opponent.data.object_index_position(Id::Identity(PlayerName::Opponent.into())),
        (2, Position::Raid(ObjectPositionRaid {}))
    );

    assert!(g.user.interface.controls().has_text("Waiting"));
    assert!(g.opponent.interface.controls().has_text("Activate"));
    assert!(g.opponent.interface.controls().has_text("Pass"));

    assert_snapshot!(Summary::run(&response));
}

#[test]
fn activate_room() {
    let (mut g, ids) = test_games::simple_game(
        Side::Champion,
        CardName::TestScheme31,
        CardName::TestMinionEndRaid,
        CardName::TestWeapon3Attack12Boost3Cost,
    );
    g.perform(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    assert_eq!(g.opponent.this_player.mana(), 999);
    assert!(!g.user.cards.get(ids.minion_id).revealed_to_me());
    let response = g.click_on(g.opponent_id(), "Activate");
    assert_eq!(g.opponent.this_player.mana(), 996); // Minion costs 3 to summon
    assert!(g.user.cards.get(ids.minion_id).revealed_to_me());
    assert!(g.opponent.cards.get(ids.minion_id).revealed_to_me());
    assert!(g.user.this_player.can_take_action());
    assert!(g.opponent.other_player.can_take_action());
    assert!(g.opponent.interface.controls().has_text("Waiting"));
    assert!(g.user.interface.controls().has_text("Test Weapon"));
    assert!(g.user.interface.controls().has_text("1\u{f06d}"));
    assert!(g.user.interface.controls().has_text("Continue"));
    assert_eq!(
        g.user.data.object_index_position(Id::CardId(ids.scheme_id)),
        (0, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.user.data.object_index_position(Id::CardId(ids.minion_id)),
        (1, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.user.data.object_index_position(Id::Identity(PlayerName::User.into())),
        (2, Position::Raid(ObjectPositionRaid {}))
    );

    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn activate_room_weapon_2() {
    let (mut g, _) = test_games::simple_game(
        Side::Champion,
        CardName::TestScheme31,
        CardName::TestMinionEndRaid,
        CardName::TestWeapon2Attack,
    );
    g.perform(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    g.click_on(g.opponent_id(), "Activate");
    assert!(g.opponent.interface.controls().has_text("Waiting"));
    assert!(!g.user.interface.controls().has_text("Test Weapon"));
    assert!(g.user.interface.controls().has_text("Continue"));
}

#[test]
fn activate_room_weapon_2_12() {
    let (mut g, _) = test_games::simple_game(
        Side::Champion,
        CardName::TestScheme31,
        CardName::TestMinionEndRaid,
        CardName::TestWeapon2Attack12Boost,
    );
    g.perform(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    g.click_on(g.opponent_id(), "Activate");
    assert!(g.opponent.interface.controls().has_text("Waiting"));
    assert!(g.user.interface.controls().has_text("Test Weapon"));
    assert!(g.user.interface.controls().has_text("2\u{f06d}"));
    assert!(g.user.interface.controls().has_text("Continue"));
}

#[test]
fn activate_room_weapon_4_12() {
    let (mut g, _) = test_games::simple_game(
        Side::Champion,
        CardName::TestScheme31,
        CardName::TestMinionEndRaid,
        CardName::TestWeapon4Attack12Boost,
    );
    g.perform(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    g.click_on(g.opponent_id(), "Activate");
    assert!(g.opponent.interface.controls().has_text("Waiting"));
    assert!(g.user.interface.controls().has_text("Test Weapon"));
    assert!(g.user.interface.controls().has_text("1\u{f06d}"));
    assert!(g.user.interface.controls().has_text("Continue"));
}

#[test]
fn activate_room_weapon_5() {
    let (mut g, _) = test_games::simple_game(
        Side::Champion,
        CardName::TestScheme31,
        CardName::TestMinionEndRaid,
        CardName::TestWeapon5Attack,
    );
    g.perform(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    g.click_on(g.opponent_id(), "Activate");
    assert!(g.opponent.interface.controls().has_text("Waiting"));
    assert!(g.user.interface.controls().has_text("Test Weapon"));
    assert!(!g.user.interface.controls().has_text("\u{f06d}"));
    assert!(g.user.interface.controls().has_text("Continue"));
}

#[test]
fn use_weapon() {
    let (mut g, ids) = test_games::simple_game(
        Side::Champion,
        CardName::TestScheme31,
        CardName::TestMinionEndRaid,
        CardName::TestWeapon3Attack12Boost3Cost,
    );
    g.perform(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    g.click_on(g.opponent_id(), "Activate");
    assert_eq!(g.user.this_player.mana(), 996); // Minion costs 3 to summon
    let response = g.click_on(g.user_id(), "Test Weapon");
    assert_eq!(g.user.this_player.mana(), 995); // Weapon costs 1 to use
    assert_eq!(g.opponent.other_player.mana(), 995); // Weapon costs 1 to use
    assert!(g.user.cards.get(ids.scheme_id).revealed_to_me());
    assert!(g.opponent.cards.get(ids.scheme_id).revealed_to_me());
    assert!(g.user.this_player.can_take_action());
    assert!(g.opponent.other_player.can_take_action());
    assert!(g.opponent.interface.controls().has_text("Waiting"));
    assert!(g.user.interface.card_anchor_nodes().has_text("Score!"));
    assert!(g.user.interface.controls().has_text("End Raid"));

    assert_eq!(
        g.user.data.object_index_position(Id::CardId(ids.scheme_id)),
        (0, Position::Browser(ObjectPositionBrowser {}))
    );
    assert_eq!(
        g.user.data.object_position(Id::CardId(ids.minion_id)),
        Position::Room(ObjectPositionRoom {
            room_id: CLIENT_ROOM_ID.into(),
            room_location: ClientRoomLocation::Front.into()
        })
    );
    assert_eq!(
        g.user.data.object_position(Id::Identity(PlayerName::User.into())),
        Position::IdentityContainer(ObjectPositionIdentityContainer {
            owner: PlayerName::User.into()
        })
    );

    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn fire_combat_ability() {
    let (mut g, ids) = test_games::simple_game(
        Side::Champion,
        CardName::TestScheme31,
        CardName::TestMinionEndRaid,
        CardName::TestWeapon3Attack12Boost3Cost,
    );
    g.perform(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    g.click_on(g.opponent_id(), "Activate");
    assert_eq!(g.user.this_player.mana(), 996); // Minion costs 3 to summon
    let response = g.click_on(g.user_id(), "Continue");
    assert_eq!(g.user.this_player.mana(), 996); // Mana is unchanged
    assert_eq!(g.opponent.other_player.mana(), 996);
    assert!(!g.user.cards.get(ids.scheme_id).revealed_to_me()); // Scheme is not revealed

    // Still Champion turn
    assert!(g.user.this_player.can_take_action());
    assert!(g.opponent.other_player.can_take_action());

    assert!(!g.user.data.raid_active()); // No raid active due to End Raid ability
    assert!(!g.opponent.data.raid_active());

    assert_eq!(
        g.user.data.object_position(Id::CardId(ids.minion_id)),
        Position::Room(ObjectPositionRoom {
            room_id: CLIENT_ROOM_ID.into(),
            room_location: ClientRoomLocation::Front.into()
        })
    );
    assert_eq!(
        g.user.data.object_position(Id::CardId(ids.scheme_id)),
        Position::Room(ObjectPositionRoom {
            room_id: CLIENT_ROOM_ID.into(),
            room_location: ClientRoomLocation::Back.into()
        })
    );
    assert_eq!(
        g.user.data.object_position(Id::Identity(PlayerName::User.into())),
        Position::IdentityContainer(ObjectPositionIdentityContainer {
            owner: PlayerName::User.into()
        })
    );

    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn score_scheme_card() {
    let (mut g, ids) = test_games::simple_game(
        Side::Champion,
        CardName::TestScheme31,
        CardName::TestMinionEndRaid,
        CardName::TestWeapon3Attack12Boost3Cost,
    );
    g.perform(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    g.click_on(g.opponent_id(), "Activate");
    g.click_on(g.user_id(), "Test Weapon");
    let response = g.click_on(g.user_id(), "Score");

    assert_eq!(g.user.this_player.score(), 1);
    assert_eq!(g.opponent.other_player.score(), 1);
    assert!(g.user.this_player.can_take_action());
    assert!(g.opponent.other_player.can_take_action());
    assert!(g.user.data.raid_active()); // Raid still active
    assert!(g.opponent.data.raid_active());
    assert!(g.opponent.interface.controls().has_text("Waiting"));
    assert!(g.user.interface.controls().has_text("End Raid"));

    assert_eq!(
        g.user.data.object_position(Id::CardId(ids.scheme_id)),
        Position::Identity(ObjectPositionIdentity { owner: PlayerName::User.into() })
    );
    assert_eq!(
        g.user.data.object_position(Id::Identity(PlayerName::User.into())),
        Position::IdentityContainer(ObjectPositionIdentityContainer {
            owner: PlayerName::User.into()
        })
    );

    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn complete_raid() {
    let (mut g, ids) = test_games::simple_game(
        Side::Champion,
        CardName::TestScheme31,
        CardName::TestMinionEndRaid,
        CardName::TestWeapon3Attack12Boost3Cost,
    );
    // Gain mana to spend an action point. Should be Overlord turn after this raid.
    g.perform(Action::GainMana(GainManaAction {}), g.user_id());
    g.perform(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    g.click_on(g.opponent_id(), "Activate");
    g.click_on(g.user_id(), "Test Weapon");
    g.click_on(g.user_id(), "Score");
    let response = g.click_on(g.user_id(), "End Raid");

    assert_eq!(g.user.this_player.score(), 1);
    assert_eq!(g.opponent.other_player.score(), 1);
    assert!(g.user.other_player.can_take_action());
    assert!(g.opponent.this_player.can_take_action());
    assert_eq!(g.opponent.interface.main_controls_option(), None);
    assert_eq!(g.user.interface.main_controls_option(), None);
    assert!(!g.user.data.raid_active()); // Raid no longer active
    assert!(!g.opponent.data.raid_active());

    assert_eq!(
        g.user.data.object_position(Id::CardId(ids.scheme_id)),
        Position::Identity(ObjectPositionIdentity { owner: PlayerName::User.into() })
    );
    assert_eq!(
        g.user.data.object_position(Id::Identity(PlayerName::User.into())),
        Position::IdentityContainer(ObjectPositionIdentityContainer {
            owner: PlayerName::User.into()
        })
    );

    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn no_activate() {
    let mut g = new_game(
        Side::Champion,
        Args { turn: Some(Side::Overlord), actions: 2, ..Args::default() },
    );

    g.play_from_hand(CardName::TestScheme31);
    g.play_from_hand(CardName::TestMinionEndRaid);

    g.play_from_hand(CardName::TestWeapon3Attack12Boost3Cost);
    g.initiate_raid(ROOM_ID);
    let response = g.click_on(g.opponent_id(), "Pass");

    assert!(g.user.interface.controls().has_text("Score"));
    assert!(g.user.interface.controls().has_text("End Raid"));
    assert!(g.opponent.interface.controls().has_text("Waiting"));
    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn raid_vault() {
    let mut g = new_game(
        Side::Champion,
        Args {
            turn: Some(Side::Overlord),
            actions: 1,
            opponent_deck_top: Some(CardName::TestScheme31),
            ..Args::default()
        },
    );

    g.play_in_room(CardName::TestMinionEndRaid, RoomId::Vault);
    g.play_from_hand(CardName::TestWeapon3Attack12Boost3Cost);
    g.initiate_raid(RoomId::Vault);
    g.click_on(g.opponent_id(), "Activate");

    let response = g.click_on(g.user_id(), "Test Weapon");
    assert!(g.user.interface.controls().has_text("Score"));
    assert!(g.opponent.interface.controls().has_text("Waiting"));
    // TODO: Deck top should not be revealed to overlord
    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn raid_sanctum() {
    let mut g = new_game(
        Side::Champion,
        Args { turn: Some(Side::Overlord), actions: 1, ..Args::default() },
    );

    g.add_to_hand(CardName::TestScheme31);
    g.play_in_room(CardName::TestMinionEndRaid, RoomId::Sanctum);
    g.play_from_hand(CardName::TestWeapon3Attack12Boost3Cost);
    g.initiate_raid(RoomId::Sanctum);
    g.click_on(g.opponent_id(), "Activate");

    let response = g.click_on(g.user_id(), "Test Weapon");
    assert!(g.user.interface.controls().has_text("Score"));
    assert!(g.opponent.interface.controls().has_text("Waiting"));
    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn raid_crypts() {
    let mut g = new_game(
        Side::Champion,
        Args {
            turn: Some(Side::Overlord),
            actions: 1,
            opponent_discard: Some(CardName::TestScheme31),
            ..Args::default()
        },
    );

    g.add_to_hand(CardName::TestScheme31);
    g.play_in_room(CardName::TestMinionEndRaid, RoomId::Crypts);
    g.play_from_hand(CardName::TestWeapon3Attack12Boost3Cost);
    g.initiate_raid(RoomId::Crypts);
    g.click_on(g.opponent_id(), "Activate");

    let response = g.click_on(g.user_id(), "Test Weapon");
    assert!(g.user.interface.controls().has_text("Score"));
    assert!(g.opponent.interface.controls().has_text("Waiting"));
    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn raid_vault_twice() {
    let mut g = new_game(
        Side::Champion,
        Args {
            turn: Some(Side::Overlord),
            actions: 1,
            opponent_deck_top: Some(CardName::TestScheme31),
            ..Args::default()
        },
    );

    g.play_in_room(CardName::TestMinionEndRaid, RoomId::Vault);
    g.play_from_hand(CardName::TestWeapon3Attack12Boost3Cost);
    g.initiate_raid(RoomId::Vault);
    g.click_on(g.opponent_id(), "Activate");
    g.click_on(g.user_id(), "Test Weapon");
    g.click_on(g.user_id(), "Score");
    g.click_on(g.user_id(), "End Raid");

    g.initiate_raid(RoomId::Vault);
    // Should not need to activate when already revealed
    assert!(!g.opponent.interface.controls().has_text("Activate"));

    // Champion spent mana on playing + activating weapon, overlord on summoning
    // minion
    assert_eq!(g.me().mana(), STARTING_MANA - 4);
    assert_eq!(g.you().mana(), STARTING_MANA - 3);

    // Should skip Activation phase:
    assert!(g.opponent.interface.controls().has_text("Waiting"));
    assert!(g.user.interface.controls().has_text("Test Weapon"));
    g.click_on(g.user_id(), "Test Weapon");

    // Champion spends mana again to use weapon, Overlord mana is unchanged.
    assert_eq!(g.me().mana(), STARTING_MANA - 5);
    assert_eq!(g.you().mana(), STARTING_MANA - 3);

    // Scheme should not longer be on top for second raid
    assert!(g.opponent.interface.controls().has_text("Waiting"));
    assert!(g.user.interface.controls().has_text("End Raid"));
    assert!(!g.user.interface.controls().has_text("Score"));
}

#[test]
fn raid_no_defenders() {
    let mut g = new_game(
        Side::Champion,
        Args { turn: Some(Side::Overlord), actions: 1, ..Args::default() },
    );

    g.play_from_hand(CardName::TestScheme31);
    let response = g.initiate_raid(ROOM_ID);
    // Should immediately jump to the Score action
    assert!(g.user.interface.controls().has_text("Score"));
    assert!(g.user.interface.controls().has_text("End Raid"));
    assert!(g.opponent.interface.controls().has_text("Waiting"));
    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn raid_vault_no_defenders() {
    let mut g = new_game(
        Side::Champion,
        Args { opponent_deck_top: Some(CardName::TestScheme31), ..Args::default() },
    );

    g.initiate_raid(RoomId::Vault);
    // Should immediately jump to the Score action
    assert!(g.user.interface.controls().has_text("Score"));
    assert!(g.user.interface.controls().has_text("End Raid"));
    assert!(g.opponent.interface.controls().has_text("Waiting"));
}

#[test]
fn raid_no_occupants() {
    let mut g = new_game(
        Side::Champion,
        Args { turn: Some(Side::Overlord), actions: 1, ..Args::default() },
    );

    g.play_from_hand(CardName::TestMinionEndRaid);
    g.play_from_hand(CardName::TestWeapon3Attack12Boost3Cost);
    let result = g.perform_action(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    assert_error(result);
}

#[test]
fn raid_no_occupants_or_defenders() {
    let mut g = new_game(Side::Champion, Args::default());

    let response = g.perform_action(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );

    assert_error(response);
}

#[test]
fn raid_two_defenders() {
    let mut g = new_game(
        Side::Champion,
        Args {
            turn: Some(Side::Overlord),
            actions: 2,
            opponent_deck_top: Some(CardName::TestScheme31),
            ..Args::default()
        },
    );

    g.play_in_room(CardName::TestMinionEndRaid, RoomId::Vault);
    g.play_in_room(CardName::TestMinionDealDamage, RoomId::Vault);
    g.play_from_hand(CardName::TestWeapon3Attack12Boost3Cost);
    g.initiate_raid(RoomId::Vault);
    g.click_on(g.opponent_id(), "Activate");
    let response = g.click_on(g.user_id(), "Test Weapon");

    assert!(g.user.interface.controls().has_text("Advance"));
    assert!(g.user.interface.controls().has_text("Retreat"));
    assert!(g.opponent.interface.controls().has_text("Waiting"));
    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn raid_two_defenders_advance() {
    let mut g = new_game(
        Side::Champion,
        Args {
            turn: Some(Side::Overlord),
            actions: 2,
            opponent_deck_top: Some(CardName::TestScheme31),
            ..Args::default()
        },
    );

    g.play_in_room(CardName::TestMinionEndRaid, RoomId::Vault);
    g.play_in_room(CardName::TestMinionDealDamage, RoomId::Vault);
    g.play_from_hand(CardName::TestWeapon3Attack12Boost3Cost);
    g.initiate_raid(RoomId::Vault);
    g.click_on(g.opponent_id(), "Activate");
    g.click_on(g.user_id(), "Test Weapon");
    let response = g.click_on(g.user_id(), "Advance");
    assert!(g.user.interface.controls().has_text("Test Weapon"));
    assert!(g.opponent.interface.controls().has_text("Waiting"));

    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn raid_two_defenders_retreat() {
    let mut g = new_game(
        Side::Champion,
        Args {
            turn: Some(Side::Overlord),
            actions: 2,
            opponent_deck_top: Some(CardName::TestScheme31),
            ..Args::default()
        },
    );

    g.play_in_room(CardName::TestMinionEndRaid, RoomId::Vault);
    g.play_in_room(CardName::TestMinionDealDamage, RoomId::Vault);
    g.play_from_hand(CardName::TestWeapon3Attack12Boost3Cost);
    g.initiate_raid(RoomId::Vault);
    g.click_on(g.opponent_id(), "Activate");
    g.click_on(g.user_id(), "Test Weapon");
    let response = g.click_on(g.user_id(), "Retreat");
    assert!(!g.user.data.raid_active());
    assert!(!g.opponent.data.raid_active());
    assert_eq!(g.opponent.interface.main_controls_option(), None);
    assert_eq!(g.user.interface.main_controls_option(), None);
    assert_eq!(
        g.user.data.object_position(Id::Identity(PlayerName::User.into())),
        Position::IdentityContainer(ObjectPositionIdentityContainer {
            owner: PlayerName::User.into()
        })
    );

    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn raid_two_defenders_full_raid() {
    let mut g = new_game(
        Side::Champion,
        Args {
            turn: Some(Side::Overlord),
            actions: 2,
            opponent_deck_top: Some(CardName::TestScheme31),
            ..Args::default()
        },
    );

    g.play_in_room(CardName::TestMinionEndRaid, RoomId::Vault);
    g.play_in_room(CardName::TestMinionDealDamage, RoomId::Vault);
    g.play_from_hand(CardName::TestWeapon3Attack12Boost3Cost);
    g.initiate_raid(RoomId::Vault);
    g.click_on(g.opponent_id(), "Activate");
    g.click_on(g.user_id(), "Test Weapon");
    g.click_on(g.user_id(), "Advance");
    g.click_on(g.user_id(), "Test Weapon");
    let response = g.click_on(g.user_id(), "Score");
    assert_eq!(g.me().mana(), STARTING_MANA - 5);
    assert_eq!(g.you().mana(), STARTING_MANA - 4);
    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn raid_deal_damage_game_over() {
    let mut g = new_game(Side::Overlord, Args { ..Args::default() });
    // Two 'deal 1 damage' defenders are needed because the Champion draws a card
    // for turn
    g.play_in_room(CardName::TestMinionDealDamage, RoomId::Vault);
    g.play_in_room(CardName::TestMinionDealDamage, RoomId::Vault);
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    assert!(g.dawn());

    g.initiate_raid(RoomId::Vault);
    g.click_on(g.user_id(), "Activate");
    g.click_on(g.opponent_id(), "Continue");
    g.click_on(g.opponent_id(), "Advance");
    g.click_on(g.opponent_id(), "Continue");

    assert!(g.is_victory_for_player(Side::Overlord));
}

#[test]
fn raid_two_defenders_cannot_afford_second() {
    let mut g = new_game(
        Side::Champion,
        Args {
            turn: Some(Side::Overlord),
            actions: 2,
            opponent_deck_top: Some(CardName::TestScheme31),
            opponent_mana: 1,
            ..Args::default()
        },
    );

    g.play_in_room(CardName::TestMinionDealDamage, RoomId::Vault);
    g.play_in_room(CardName::TestMinionEndRaid, RoomId::Vault);
    g.play_from_hand(CardName::TestWeapon3Attack12Boost3Cost);
    g.initiate_raid(RoomId::Vault);
    g.click_on(g.opponent_id(), "Activate");
    g.click_on(g.user_id(), "Test Weapon");
    let response = g.click_on(g.user_id(), "Score");
    assert_eq!(g.me().mana(), STARTING_MANA - 4);
    assert_eq!(g.you().mana(), 0);
    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn raid_add_defender() {
    let mut g = new_game(
        Side::Champion,
        Args { turn: Some(Side::Overlord), actions: 2, ..Args::default() },
    );

    g.play_from_hand(CardName::TestMinionEndRaid);
    g.play_from_hand(CardName::TestScheme31);
    assert!(g.dawn());

    // Raid 1
    g.initiate_raid(ROOM_ID);
    g.click_on(g.opponent_id(), "Activate");
    g.click_on(g.user_id(), "Continue");
    assert!(!g.user.data.raid_active());

    g.play_from_hand(CardName::TestWeapon3Attack12Boost3Cost);

    // Raid 2, no activate
    g.initiate_raid(ROOM_ID);
    g.click_on(g.user_id(), "Test Weapon");
    g.click_on(g.user_id(), "Score");
    g.click_on(g.user_id(), "End Raid");
    assert!(!g.user.data.raid_active());

    // Opponent Turn
    assert!(g.dusk());
    g.play_from_hand(CardName::TestMinionDealDamage);
    g.play_from_hand(CardName::TestScheme31);
    g.perform(Action::GainMana(GainManaAction {}), g.opponent_id());

    // User Turn, Raid 3
    assert!(g.dawn());
    g.initiate_raid(ROOM_ID);
    g.click_on(g.opponent_id(), "Activate");
    g.click_on(g.user_id(), "Test Weapon");
    g.click_on(g.user_id(), "Advance");
    let response = g.click_on(g.user_id(), "Test Weapon");
    assert_snapshot!(Summary::summarize(&response));
}