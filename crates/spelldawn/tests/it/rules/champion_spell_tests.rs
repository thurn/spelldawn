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
use data::primitives::Side;
use test_utils::*;

#[test]
fn arcane_recovery() {
    let mut g = new_game(Side::Champion, Args { mana: 5, ..Args::default() });
    g.play_from_hand(CardName::ArcaneRecovery);
    assert_eq!(9, g.user().mana());
    assert_eq!(9, g.opponent.other_player.mana())
}