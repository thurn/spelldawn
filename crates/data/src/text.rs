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

#![allow(clippy::use_self)] // Required to use EnumKind

use std::fmt;
use std::fmt::{Debug, Formatter};

use enum_kinds::EnumKind;

use crate::card_definition::{CardDefinition, Cost};
use crate::card_name::CardName;
use crate::card_state::{CardData, CardState};
use crate::game::GameState;
use crate::primitives::{ActionCount, BreachValue, CardId, ManaValue};

/// Provides the context in which rules text is being evaluated, i.e. during an
/// active game or in a deck editor.
pub enum RulesTextContext<'a> {
    Default(&'a CardDefinition),
    Game(&'a GameState, &'a CardState),
}

impl<'a> RulesTextContext<'a> {
    pub fn card_name(&self) -> CardName {
        match self {
            RulesTextContext::Default(definition) => definition.name,
            RulesTextContext::Game(_, card) => card.name,
        }
    }

    pub fn card_data(&self) -> Option<&CardData> {
        match self {
            RulesTextContext::Default(_) => None,
            RulesTextContext::Game(_, card) => Some(&card.data),
        }
    }

    /// Invokes the provided `game` function to product a value in the active
    /// game context, otherwise returns some `default`.
    pub fn query_or<T>(&self, default: T, game: impl Fn(&GameState, CardId) -> T) -> T {
        match self {
            RulesTextContext::Default(_) => default,
            RulesTextContext::Game(state, card) => game(state, card.id),
        }
    }
}

/// A function which produces rules text
pub type TextFn = fn(&RulesTextContext) -> Vec<TextToken>;

/// Text describing what an ability does. Can be a function (if text is dynamic)
/// or a vector of [TextToken]s.
#[derive(Clone)]
pub enum AbilityText {
    Text(Vec<TextToken>),
    TextFn(TextFn),
}

impl Debug for AbilityText {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AbilityText::Text(tokens) => write!(f, "{:?}", tokens),
            AbilityText::TextFn(_) => write!(f, "<TextFn>"),
        }
    }
}

/// Different types of text which can appear in rules text
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum TextToken {
    Literal(String),
    Number(NumericOperator, u32),
    Mana(ManaValue),
    Actions(ActionCount),
    Keyword(Keyword),
    Reminder(String),
    Cost(Vec<Self>),
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
/// Location of a keyword within a sentence, used to determine capitalization
pub enum Sentence {
    Start,
    Internal,
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum DamageWord {
    DealStart,
    DealInternal,
    TakeStart,
    TakeInternal,
}

/// Identifies a keyword or concept which appears in rules text
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, EnumKind)]
#[enum_kind(KeywordKind, derive(Ord, PartialOrd))]
pub enum Keyword {
    Play,
    Dawn,
    Dusk,
    Score,
    Combat,
    Encounter,
    Unveil,
    SuccessfulRaid,
    Store(Sentence, u32),
    Take(Sentence, u32),
    DealDamage(DamageWord, u32),
    InnerRoom(Sentence),
    Breach(BreachValue),
    LevelUp,
    Trap,
    Construct,
}

impl Keyword {
    pub fn kind(&self) -> KeywordKind {
        self.into()
    }
}

/// A symbol applied to a number which appears in rules text
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum NumericOperator {
    None,
    Add,
}

impl From<&str> for TextToken {
    fn from(s: &str) -> Self {
        Self::Literal(s.to_owned())
    }
}

impl From<u32> for TextToken {
    fn from(v: u32) -> Self {
        Self::Number(NumericOperator::None, v)
    }
}

impl From<Keyword> for TextToken {
    fn from(k: Keyword) -> Self {
        Self::Keyword(k)
    }
}

impl<T> From<Cost<T>> for TextToken {
    fn from(cost: Cost<T>) -> Self {
        let mut result = vec![];
        if let Some(mana) = cost.mana {
            result.push(Self::Mana(mana))
        }

        if cost.actions > 1 {
            result.push(Self::Actions(cost.actions));
        }

        Self::Cost(result)
    }
}
