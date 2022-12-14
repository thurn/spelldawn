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

using System.Collections;
using System.Collections.Generic;
using System.Linq;
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using TMPro;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class IdentityCard : StackObjectDisplay, ArrowService.IArrowDelegate
  {
    [Header("Identity")] [SerializeField] Registry _registry = null!;
    [SerializeField] SpriteRenderer _image = null!;
    [SerializeField] SpriteRenderer _frame = null!;
    [SerializeField] TextMeshPro _scoreText = null!;
    [SerializeField] GameObject _raidSymbol = null!;
    [SerializeField] PlayerName _owner;
    ISet<RoomIdentifier>? _validRoomsToVisit;

    public PlayerSide Side { get; set; }

    protected override Registry Registry => _registry;

    protected override GameContext DefaultGameContext() => GameContext.Identity;

    protected override float? CalculateObjectScale(int index, int count) => 0f;

    public override float DefaultScale => 2.0f;

    public override bool IsContainer() => false;

    public void OnRaidStateChanged(bool raidActive)
    {
      if (Side == PlayerSide.Champion)
      {
        _raidSymbol.SetActive(raidActive);
      }
    }

    public void DisableAnimation()
    {
      var text = ComponentUtils.GetComponent<TextMeshPro>(_raidSymbol);
      text.fontMaterial = new Material(Shader.Find("TextMeshPro/Distance Field"));
      text.color = new Color(1.0f, 0.435f, 0.0f);
    }

    public void RenderPlayerInfo(PlayerInfo playerInfo)
    {
      _validRoomsToVisit = playerInfo.ValidRoomsToVisit.ToHashSet();
      _registry.AssetService.AssignSprite(_image, playerInfo.Portrait);
      _registry.AssetService.AssignSprite(_frame, playerInfo.PortraitFrame);
    }

    public void RenderScore(ScoreView scoreView)
    {
      _scoreText.text = scoreView.Score.ToString();
    }

    public override bool CanHandleMouseDown() => true;

    public override void MouseDown()
    {
      base.MouseDown();

      if (_owner == PlayerName.User && _registry.CapabilityService.CanInitiateAction())
      {
        _registry.StaticAssets.PlayCardSound();
        switch (Side)
        {
          case PlayerSide.Champion:
            _registry.ArrowService.ShowArrow(ArrowService.Type.Red, transform, this);
            break;
          case PlayerSide.Overlord:
            _registry.ArrowService.ShowArrow(ArrowService.Type.Green, transform, this);
            break;
        }
      }
    }

    protected override void LongPress()
    {
      StartCoroutine(_registry.LongPressCardBrowser.BrowseCards(this));
    }

    public void OnArrowMoved(Vector3 position)
    {
      _registry.ArenaService.ShowRoomSelectorForMousePosition(Errors.CheckNotNull(_validRoomsToVisit));
    }

    public void OnArrowReleased(Vector3 position)
    {
      if (_registry.ArenaService.CurrentRoomSelector is { } selectedRoom)
      {
        switch (Side)
        {
          case PlayerSide.Champion:
            _registry.ActionService.HandleAction(new ClientAction
            {
              InitiateRaid = new InitiateRaidAction
              {
                RoomId = selectedRoom.RoomId
              }
            });
            break;
          case PlayerSide.Overlord:
            _registry.ActionService.HandleAction(new ClientAction
            {
              LevelUpRoom = new LevelUpRoomAction
              {
                RoomId = selectedRoom.RoomId
              }
            });
            break;
        }
      }
      else
      {
        _registry.StaticAssets.PlayCardSound();
      }

      _registry.ArenaService.HideRoomSelector();
    }

    protected override void OnSetGameContext(GameContext _, GameContext gameContext)
    {
    }
  }
}