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

//! A fake game client. Records server responses about a game and stores them in
//! [TestGame].

use std::cmp::Ordering;
use std::collections::HashMap;

use anyhow::{Context, Result};
use data::card_name::CardName;
use data::card_state::{CardData, CardPosition, CardState};
use data::game::GameState;
use data::primitives::{
    ActionCount, CardId, CardType, GameId, ManaValue, PointsValue, RoomId, Side, UserId,
};
use display::full_sync;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{
    card_target, game_object_identifier, CardIdentifier, CardTarget, CardView, ClientRoomLocation,
    CommandList, CreateOrUpdateCardCommand, GameAction, GameIdentifier, GameRequest,
    ObjectPosition, ObjectPositionDiscardPile, ObjectPositionHand, ObjectPositionRoom,
    PlayCardAction, PlayerName, PlayerView, RevealedCardView,
};
use server::database::Database;
use server::GameResponse;

/// A fake game for use in testing.
///
/// This struct keeps track of server responses related to an ongoing game and
/// converts them into a useful format for writing tests. This enables our
/// 'black box' testing strategy, where the game is almost exclusively tested
/// via the public client-facing API.
///
/// There are actually two perspectives on an ongoing game: each player has
/// their own view of the state of the game, which differs due to hidden
/// information. This struct has two different [TestClient]s which get updated
/// based on server responses, representing what the two players are seeing.
#[derive(Debug, Clone)]
pub struct TestGame {
    /// This is the perspective of the player identified by the `user_id`
    /// parameter to [Self::new].
    pub user: TestClient,
    /// This is the perspective of the player identified by the `opponent_id`
    /// parameter to [Self::new].
    pub opponent: TestClient,
    game: GameState,
}

impl TestGame {
    /// Creates a new game, starting in the provided [GameState].
    ///
    /// It is usually better to create a blank new game and then update its
    /// state via the action methods on this struct instead of putting a bunch
    /// of information into the [GameState] here, because this helps avoid
    /// coupling tests to the specific implementation details of [GameState].
    pub fn new(game: GameState, user_id: UserId, opponent_id: UserId) -> Self {
        Self { user: TestClient::new(user_id), opponent: TestClient::new(opponent_id), game }
    }

    pub fn game_id(&self) -> GameId {
        self.game.id
    }

    pub fn user_id(&self) -> UserId {
        self.user.id
    }

    pub fn opponent_id(&self) -> UserId {
        self.opponent.id
    }

    /// Returns the user player state for the user client, (i.e. the user's
    /// state from *their own* perspective).
    pub fn user(&self) -> &ClientPlayer {
        &self.user.this_player
    }

    /// Simulates a client connecting to the server, either creating a new game
    /// or connecting to an existing game. Returns the commands which would
    /// be sent to the client when connected. If a new game is created, its
    /// ID will be 0.
    pub fn connect(&mut self, user_id: UserId, game_id: Option<GameId>) -> Result<CommandList> {
        let result = server::handle_connect(self, user_id, game_id, true /* test mode */)?;
        let to_update = match () {
            _ if user_id == self.user.id => &mut self.user,
            _ if user_id == self.opponent.id => &mut self.opponent,
            _ => panic!("Unknown user id: {:?}", user_id),
        };

        // Clear all previous state
        *to_update = TestClient::new(user_id);

        for command in result.commands.iter() {
            let c = command.command.as_ref().with_context(|| "Command not received")?;
            to_update.handle_command(c);
        }

        Ok(result)
    }

    /// Execute a simulated client request for this game as a specific user,
    /// updating the client state as appropriate based on the responses.
    /// Returns the [GameResponse] for this action or an error if the server
    /// request failed.
    pub fn perform_action(&mut self, action: Action, user_id: UserId) -> Result<GameResponse> {
        let response = server::handle_request(
            self,
            &GameRequest {
                action: Some(GameAction { action: Some(action) }),
                game_id: Some(GameIdentifier { value: self.game.id.value }),
                user_id: user_id.value,
            },
        )?;

        let (opponent_id, local, remote) = match () {
            _ if user_id == self.user.id => (self.opponent.id, &mut self.user, &mut self.opponent),
            _ if user_id == self.opponent.id => (self.user.id, &mut self.opponent, &mut self.user),
            _ => panic!("Unknown user id: {:?}", user_id),
        };

        for command in &response.command_list.commands {
            local.handle_command(command.command.as_ref().with_context(|| "Command not received")?);
        }

        if let Some((channel_user_id, list)) = &response.channel_response {
            assert_eq!(*channel_user_id, opponent_id);
            for command in &list.commands {
                remote.handle_command(
                    command.command.as_ref().with_context(|| "Command not received")?,
                );
            }
        }

        Ok(response)
    }

    /// Adds a named card to its owner's hand.
    ///
    /// This function operates by locating a test card in the owner's deck and
    /// overwriting it with the provided `card_name`. This card is then
    /// moved to the user's hand via [GameState::move_card].
    /// CreateOrUpdateCard commands are sent to the attached test clients.
    ///
    /// This function will *not* check the legality of drawing a card, invoke
    /// any game events, or append a game update. It will correctly update
    /// the card's sorting key, however.
    ///
    /// Returns the client [CardIdentifier] for the drawn card. Panics if no
    /// test cards remain in the user's deck.
    pub fn add_to_hand(&mut self, card_name: CardName) -> CardIdentifier {
        let side = side_for_card_name(card_name);
        let card_id = self
            .game
            .cards_in_position(side, CardPosition::DeckUnknown(side))
            .find(|c| c.name.is_test_card())
            .expect("No test cards remaining in deck")
            .id;
        overwrite_card(&mut self.game, card_id, card_name);
        self.game.move_card(card_id, CardPosition::Hand(side));

        self.connect(self.user.id, Some(self.game.id)).expect("User connection error");
        self.connect(self.opponent.id, Some(self.game.id)).expect("Opponent connection error");

        full_sync::adapt_card_id(card_id)
    }

    /// Creates and then plays a named card.
    ///
    /// This function first adds a copy of the requested card to the user's hand
    /// via [Self::add_to_hand]. The card is then played via the standard
    /// [PlayCardAction].
    ///
    /// If the card is a minion, project, scheme, or upgrade card, it is played
    /// into the [crate::ROOM_ID] room. The [GameResponse] produced by
    /// playing the card is returned.
    ///
    /// Panics if the server returns an error for playing this card.
    pub fn play_from_hand(&mut self, card_name: CardName) -> GameResponse {
        let card_id = self.add_to_hand(card_name);

        let target = match rules::get(card_name).card_type {
            CardType::Minion | CardType::Project | CardType::Scheme | CardType::Upgrade => {
                Some(CardTarget {
                    card_target: Some(card_target::CardTarget::RoomId(
                        full_sync::adapt_room_id(crate::ROOM_ID).into(),
                    )),
                })
            }
            _ => None,
        };

        self.perform_action(
            Action::PlayCard(PlayCardAction { card_id: Some(card_id), target }),
            self.user.id,
        )
        .expect("Server error playing card")
    }
}

/// Overwrites the card with ID `card_id` in `game` to be a new card with the
/// provided `card_name`.
pub fn overwrite_card(game: &mut GameState, card_id: CardId, card_name: CardName) {
    *game.card_mut(card_id) = CardState {
        id: card_id,
        name: card_name,
        side: card_id.side,
        position: game.card(card_id).position,
        sorting_key: 0,
        data: CardData::default(),
    };
}

pub fn side_for_card_name(name: CardName) -> Side {
    rules::get(name).side
}

impl Database for TestGame {
    fn generate_game_id(&self) -> Result<GameId> {
        panic!("Attempted to generate new ID for test game!")
    }

    fn game(&self, _id: GameId) -> Result<GameState> {
        Ok(self.game.clone())
    }

    fn write_game(&mut self, game: &GameState) -> Result<()> {
        self.game = game.clone();
        Ok(())
    }
}

/// Represents a user client connected to a test game
#[derive(Debug, Clone)]
pub struct TestClient {
    pub id: UserId,
    pub data: ClientGameData,
    /// A player's view of *their own* state.
    pub this_player: ClientPlayer,
    /// A player's view of *their opponent's* state.
    pub other_player: ClientPlayer,
    pub cards: ClientCards,
}

impl TestClient {
    fn new(id: UserId) -> Self {
        Self {
            id,
            data: ClientGameData::default(),
            this_player: ClientPlayer::new(PlayerName::User),
            other_player: ClientPlayer::new(PlayerName::Opponent),
            cards: ClientCards::default(),
        }
    }

    fn handle_command(&mut self, command: &Command) {
        self.data.update(command.clone());
        self.this_player.update(command.clone());
        self.other_player.update(command.clone());
        self.cards.update(command.clone());
    }
}

/// Simulated game state in an ongoing [TestGame]
#[derive(Debug, Clone, Default)]
pub struct ClientGameData {
    priority: Option<PlayerName>,
}

impl ClientGameData {
    pub fn priority(&self) -> PlayerName {
        self.priority.unwrap()
    }

    fn update(&mut self, command: Command) {
        if let Command::UpdateGameView(update_game) = command {
            self.priority =
                PlayerName::from_i32(update_game.game.as_ref().unwrap().current_priority)
        }
    }
}

/// Simulated player state in an ongoing [TestGame]
#[derive(Debug, Clone)]
pub struct ClientPlayer {
    name: PlayerName,
    mana: Option<ManaValue>,
    actions: Option<ActionCount>,
    score: Option<PointsValue>,
}

impl ClientPlayer {
    fn new(name: PlayerName) -> Self {
        Self { name, mana: None, actions: None, score: None }
    }

    pub fn mana(&self) -> ManaValue {
        self.mana.expect("Mana")
    }

    pub fn actions(&self) -> ActionCount {
        self.actions.expect("Actions")
    }

    pub fn score(&self) -> PointsValue {
        self.score.expect("Points")
    }

    fn update(&mut self, command: Command) {
        if let Command::UpdateGameView(update) = command {
            self.update_with_player(if self.name == PlayerName::User {
                update.game.unwrap().user
            } else {
                update.game.unwrap().opponent
            });
        }
    }

    fn update_with_player(&mut self, player: Option<PlayerView>) {
        if let Some(p) = player {
            write_if_present(&mut self.mana, p.mana, |v| v.amount);
            write_if_present(&mut self.actions, p.action_tracker, |v| v.available_action_count);
            write_if_present(&mut self.score, p.score, |v| v.score);
        }
    }
}
/// Simulated card state in an ongoing [TestGame]
#[derive(Debug, Clone, Default)]
pub struct ClientCards {
    cards: HashMap<CardId, ClientCard>,
}

impl ClientCards {
    /// Returns a vec containing the titles of all of the cards in the provided
    /// player's hand from the perspective of the this client, or
    /// [crate::HIDDEN_CARD] if the card's title is unknown. Titles will be
    /// ordered by their sorting key.
    pub fn hand(&self, player: PlayerName) -> Vec<String> {
        self.names_in_position(Position::Hand(ObjectPositionHand { owner: player.into() }))
    }

    /// Returns a player's discard pile in the same manner as [Self::hand]
    pub fn discard_pile(&self, player: PlayerName) -> Vec<String> {
        self.names_in_position(Position::DiscardPile(ObjectPositionDiscardPile {
            owner: player.into(),
        }))
    }

    /// Returns a vector containing the card titles in the provided `location`
    /// of a given room, Titles are structured in the same manner described
    /// in [Self::hand].
    pub fn room_cards(&self, room_id: RoomId, location: ClientRoomLocation) -> Vec<String> {
        self.names_in_position(Position::Room(ObjectPositionRoom {
            room_id: full_sync::adapt_room_id(room_id).into(),
            room_location: location.into(),
        }))
    }

    /// Returns an iterator over the cards in a given [Position] in an arbitrary
    /// order.
    pub fn in_position(&self, position: Position) -> impl Iterator<Item = &ClientCard> {
        self.cards.values().filter(move |c| c.position() == position)
    }

    /// Returns a list of the titles of cards in the provided `position`, or the
    /// string [crate::HIDDEN_CARD] if no title is available. Cards are
    /// sorted in position order based on their `sorting_key` with ties being
    /// broken arbitrarily.
    pub fn names_in_position(&self, position: Position) -> Vec<String> {
        let mut result = self
            .in_position(position)
            .map(|c| c.title_option().unwrap_or_else(|| crate::HIDDEN_CARD.to_string()))
            .collect::<Vec<_>>();
        result.sort();
        result
    }

    fn update(&mut self, command: Command) {
        match command {
            Command::CreateOrUpdateCard(create_or_update) => {
                let card_view = create_or_update.clone().card.expect("CardView");
                let card_id = server::to_server_card_id(&card_view.card_id).expect("CardId");
                self.cards
                    .entry(card_id)
                    .and_modify(|c| c.update(&card_view))
                    .or_insert_with(|| ClientCard::new(&create_or_update));
            }
            Command::MoveGameObjects(move_objects) => {
                let position = move_objects.clone().position.expect("ObjectPosition");
                for id in move_objects.ids {
                    if let game_object_identifier::Id::CardId(identifier) = id.id.expect("ID") {
                        let card_id = server::to_server_card_id(&Some(identifier)).expect("CardId");
                        assert!(
                            self.cards.contains_key(&card_id),
                            "Expected a CreateOrUpdate command before a Move command for card {:?}",
                            card_id
                        );
                        let mut card = self.cards.get_mut(&card_id).unwrap();
                        card.position = Some(position.clone());
                    }
                }
            }
            Command::DestroyCard(destroy_card) => {
                let card_id = server::to_server_card_id(&destroy_card.card_id).expect("CardId");
                self.cards.remove(&card_id);
            }
            _ => {}
        }
    }
}

/// Simulated state of a specific card
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClientCard {
    title: Option<String>,
    position: Option<ObjectPosition>,
}

impl ClientCard {
    /// Returns the game object position for this card
    pub fn position(&self) -> Position {
        self.position.clone().expect("CardPosition").position.expect("Position")
    }

    /// Returns the user-visible title for this card. Panics if no title is
    /// available.
    pub fn title(&self) -> String {
        self.title_option().expect("No card title found")
    }

    /// Returns a copy of the user-visible title for this card, if one is
    /// available
    pub fn title_option(&self) -> Option<String> {
        self.title.clone()
    }

    fn new(command: &CreateOrUpdateCardCommand) -> Self {
        let mut result = Self { position: command.create_position.clone(), ..Self::default() };
        result.update(command.card.as_ref().expect("No CardView found"));
        result
    }

    fn update(&mut self, view: &CardView) {
        if let Some(revealed) = &view.revealed_card {
            self.update_revealed_card(revealed);
        }
    }

    fn update_revealed_card(&mut self, revealed: &RevealedCardView) {
        if let Some(title) = revealed.clone().title.map(|title| title.text) {
            self.title = Some(title);
        }
    }
}

impl PartialOrd for ClientCard {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.position.as_ref()?.sorting_key.partial_cmp(&other.position.as_ref()?.sorting_key)
    }
}

fn write_if_present<T, U>(value: &mut Option<T>, option: Option<U>, map: impl Fn(U) -> T) {
    if let Some(v) = option {
        *value = Some(map(v));
    }
}