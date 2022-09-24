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

//! Addresses for user interface panels

use data::primitives::{DeckId, Side};
use protos::spelldawn::{interface_panel_address, InterfacePanelAddress};
use serde::{Deserialize, Serialize};
use serde_json::ser;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PanelAddress {
    SetPlayerName(Side),
    DeckEditor(DeckEditorData),
    CreateDeck,
}

impl From<PanelAddress> for InterfacePanelAddress {
    fn from(address: PanelAddress) -> Self {
        Self {
            address_type: Some(interface_panel_address::AddressType::Serialized(
                ser::to_vec(&address).expect("Serialization failed"),
            )),
        }
    }
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct DeckEditorData {
    /// Deck currently being viewed
    pub deck: Option<DeckId>,
}