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
using Spelldawn.Services;
using Spelldawn.Utils;
using TMPro;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class ManaDisplay : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] bool _clickable;
    [SerializeField] GameObject _pressEffect = null!;
    [SerializeField] TextMeshPro _manaText = null!;
    [SerializeField] int _currentMana = 5;

    public int CurrentMana => _currentMana;

    public void Increment()
    {
      SetMana(_currentMana + 1);
    }

    public void SetMana(int currentMana)
    {
      Errors.CheckNonNegative(currentMana);
      _currentMana = currentMana;
      _manaText.text = "" + _currentMana;
    }

    void OnMouseDown()
    {
      if (_clickable)
      {
        transform.localScale = 0.95f * Vector3.one;
      }
    }

    void OnMouseUp()
    {
      if (_clickable)
      {
        transform.localScale = Vector3.one;
      }
    }

    void OnMouseUpAsButton()
    {
      if (_clickable)
      {
        _pressEffect.SetActive(false);
        _pressEffect.SetActive(true);
        _registry.ActionService.HandleAction(new GameAction
        {
          GainMana = new GainManaAction()
        });
      }
    }
  }
}