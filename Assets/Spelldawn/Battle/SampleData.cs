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

using Spelldawn.Protos;
using static Spelldawn.Masonry.MasonUtil;

#nullable enable

namespace Spelldawn.Battle
{
  public static class SampleData
  {
    static readonly string Text1 = @"★, 2❋: Destroy this token. Add another line of text to this card.
<b>↯Play</b>: Give the Overlord the <u>Shatter</u> ability";

    static readonly string Text2 = @"If another item would be destroyed, this item is destroyed instead
";

    static readonly string Text3 = @"<b>↯Play:</b> <b>Store</b> 12❋
★: <b>Take</b> 2❋
";

    static readonly string Text4 =
      @"Search your deck for a weapon. If you made a successful raid this turn you may play it <i>(paying its costs)</i>, otherwise put it into your hand
";

    static readonly string Text5 =
      @"Choose a minion. That minion gains the <b>Infernal</b>, <b>Human</b>, and <b>Abyssal</b> subtypes until end of turn.
";

    static readonly string Text6 = @"<b>Choose One:</b>
• Gain 2❋
• Reveal one card
";

    static readonly string Text7 = @"1❋ <b>Recurring</b> <i>(Refill up to 1❋ each turn)</i>
Use this ❋ to pay for using Weapons or equipping Silver items 
";

    static readonly string Text8 = @"<b>Attack:</b> 2❋ → 1 damage
<b>Strike</b> 2 <i>(deal 2 damage at the start of combat)</i>
<b>↯Play:</b> Pick a minion in play. Whenever you pay that minion’s shield cost, kill it
";

    static readonly string Text9 = @"<b>Attack:</b> 1❋ → 1 damage
★: Place a ◈ on this item
When you use this item, remove a ◈ or sacrifice it
";

    static readonly string Text10 = @"<b>Attack:</b> 1❋ → 1 damage
<b>Strike</b> 2 <i>(deal 2 damage at the start of combat)</i>
<b>Area</b> <i>(this item’s damage persists for the duration of the raid)</i>
";

    public static GameView SampleView() =>
      new()
      {
        User = new PlayerView
        {
          Hand = new HandView
          {
            Cards =
            {
              RevealedCard(1, "Meteor Shower", Text1),
              RevealedCard(2, "The Maker's Eye", Text2),
              RevealedCard(3, "Gordian Blade", Text3),
              RevealedCard(4, "Personal Touch", Text4),
              RevealedCard(5, "Secret Key", Text5),
              RevealedCard(6, "Femme Fatale", Text6),
            }
          }
        },
        Opponent = new PlayerView(),
        Arena = new ArenaView()
      };

    static CardView RevealedCard(int cardId, string title, string text) => new()
    {
      CardId = cardId,
      RevealedCard = new RevealedCardView
      {
        CardBack = Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Backs/Back_Steampunk_Style_Color_1"),
        CardFrame = Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Cards/Card_Steampunk_Style_Color_1"),
        Webbing = Sprite(
          "LittleSweetDaemon/TCG_Card_Design/Magic_Card/Magic_Card_Face_Tape"),
        Jewel = Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Jewels/Jewel_Steampunk_Color_01"),
        Image = Sprite($"Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_2{cardId}"),
        Title = new CardTitle
        {
          Text = title
        },
        RulesText = new RulesText
        {
          Text = text
        },
        CanPlay = false
      }
    };
  }
}