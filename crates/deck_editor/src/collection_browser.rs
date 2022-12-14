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

use core_ui::action_builder::ActionBuilder;
use core_ui::animations::{
    self, default_duration, AnimateToElement, CreateTargetAtIndex, DestroyElement,
    InterfaceAnimation,
};
use core_ui::conditional::Conditional;
use core_ui::draggable::Draggable;
use core_ui::drop_target::DropTarget;
use core_ui::prelude::*;
use data::card_name::CardName;
use data::deck::Deck;
use data::player_data::PlayerData;
use data::primitives::Side;
use data::user_actions::DeckEditorAction;
use deck_card::deck_card_slot::DeckCardSlot;
use deck_card::{CardHeight, DeckCard};
use element_names::{CurrentDraggable, ElementName, TargetName};
use panel_address::CollectionBrowserFilters;
use protos::spelldawn::{FlexAlign, FlexDirection, FlexJustify};

use crate::card_list;
use crate::card_list_card_name::CardListCardName;

// use crate::card_list;
// use crate::deck_editor_card::DeckEditorCard;
// use crate::empty_card::EmptyCard;

/// Returns an iterator over cards owned by 'player' which match a given
/// [CollectionBrowserFilters]
pub fn get_matching_cards(
    player: &PlayerData,
    _: CollectionBrowserFilters,
) -> impl Iterator<Item = (CardName, u32)> + '_ {
    // TODO: Use adventure collection
    player
        .collection
        .iter()
        .map(|(card_name, count)| (*card_name, *count))
        .filter(|(name, _)| rules::get(*name).side == Side::Champion)
}

pub struct CollectionBrowser<'a> {
    pub player: &'a PlayerData,
    pub deck: &'a Deck,
    pub filters: CollectionBrowserFilters,
}

impl<'a> CollectionBrowser<'a> {
    fn card_row(&self, cards: Vec<&(CardName, u32)>) -> impl Component {
        let empty_slots = if cards.len() < 4 { 4 - cards.len() } else { 0 };
        Row::new("CardRow")
            .style(
                Style::new()
                    .flex_grow(1.0)
                    .align_items(FlexAlign::Center)
                    .justify_content(FlexJustify::Center),
            )
            .children(cards.into_iter().map(|(n, quantity)| {
                let card_name = *n;
                let quantity_element = ElementName::new("Quantity");
                DeckCardSlot::new(CardHeight::vh(36.0))
                    .layout(Layout::new().margin(Edge::All, 16.px()))
                    .card(Some(
                        DeckCard::new(card_name)
                            .quantity(*quantity)
                            .quantity_element_name(quantity_element)
                            .draggable(
                                Draggable::new(card_name.to_string())
                                    .drop_target(element_names::CARD_LIST)
                                    .over_target_indicator(move || {
                                        CardListCardName::new(card_name).build()
                                    })
                                    .on_drop(Some(self.drop_action(card_name)))
                                    .hide_indicator_children(vec![quantity_element]),
                            ),
                    ))
            }))
            .children((0..empty_slots).map(|_| {
                DeckCardSlot::new(CardHeight::vh(36.0))
                    .layout(Layout::new().margin(Edge::All, 4.px()))
            }))
    }

    fn drop_action(&self, name: CardName) -> ActionBuilder {
        let element_name = element_names::card_list_card_name(name);
        let target_name = TargetName(element_name);
        ActionBuilder::new().action(DeckEditorAction::AddToDeck(name)).update(
            Conditional::if_exists(element_name)
                .then(
                    InterfaceAnimation::new()
                        .start(CurrentDraggable, AnimateToElement::new(element_name))
                        .insert(animations::default_duration(), CurrentDraggable, DestroyElement),
                )
                .or_else(
                    InterfaceAnimation::new()
                        .start(
                            CurrentDraggable,
                            CreateTargetAtIndex::parent(element_names::CARD_LIST)
                                .index(card_list::position_for_card(self.deck, name) as u32)
                                .name(target_name),
                        )
                        .start(
                            CurrentDraggable,
                            // We need to offset this animation because the
                            // target is moving *to* its size while the card is
                            // moving to the target.
                            AnimateToElement::new(target_name).disable_height_half_offset(true),
                        )
                        .insert(default_duration(), CurrentDraggable, DestroyElement),
                ),
        )
    }
}

fn sort_cards(cards: &mut [(CardName, u32)]) {
    cards.sort_by_key(|(name, _)| {
        let definition = rules::get(*name);
        let cost = definition.cost.mana.unwrap_or_default();
        (definition.side, definition.school, definition.card_type, cost, name.displayed_name())
    });
}

impl<'a> Component for CollectionBrowser<'a> {
    fn build(self) -> Option<Node> {
        let mut cards = get_matching_cards(self.player, self.filters).collect::<Vec<_>>();
        sort_cards(&mut cards);
        let row_one = cards.iter().skip(self.filters.offset).take(4).collect::<Vec<_>>();
        let row_two = cards.iter().skip(self.filters.offset + 4).take(4).collect::<Vec<_>>();
        DropTarget::new(element_names::COLLECTION_BROWSER)
            .style(
                Style::new()
                    .flex_direction(FlexDirection::Column)
                    .flex_grow(1.0)
                    .align_items(FlexAlign::Center)
                    .justify_content(FlexJustify::Center),
            )
            .child(self.card_row(row_one))
            .child(self.card_row(row_two))
            .build()
    }
}
