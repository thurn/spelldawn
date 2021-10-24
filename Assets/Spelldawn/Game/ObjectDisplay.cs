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
using DG.Tweening;
using Spelldawn.Utils;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public abstract class ObjectDisplay : Displayable
  {
    [Header("Object Display")] [SerializeField]
    List<Displayable> _objects = new();

    [SerializeField] bool _updateRequired;

    [SerializeField] bool _animateNextUpdate;

    [SerializeField] bool _animationRunning;

    public List<Displayable> AllObjects => new(_objects);

    void Update()
    {
      if (_updateRequired && !_animationRunning)
      {
        MoveObjectsToPosition(_animateNextUpdate);
        _updateRequired = false;
      }
    }

    public IEnumerator AddObject(Displayable displayable, bool animate = true, int? index = null)
    {
      MarkUpdateRequired(animate);
      Insert(displayable, index);
      yield return new WaitUntil(() => !_animationRunning && !_updateRequired);
    }

    public IEnumerator AddObjects(IEnumerable<Displayable> objects, bool animate = true)
    {
      MarkUpdateRequired(animate);
      foreach (var displayable in objects)
      {
        Insert(displayable, null);
      }

      yield return new WaitUntil(() => !_animationRunning && !_updateRequired);
    }

    public int RemoveObject(Displayable displayable, bool animate = true)
    {
      MarkUpdateRequired(animate);
      var index = _objects.FindIndex(c => c == displayable);
      Errors.CheckNonNegative(index);
      _objects.RemoveAt(index);
      return index;
    }

    public void RemoveObjectIfPresent(Displayable displayable, bool animate = true)
    {
      MarkUpdateRequired(animate);
      _objects.Remove(displayable);
    }

    public void DebugUpdate()
    {
      MarkUpdateRequired(true);
    }

    protected override void OnSetGameContext(GameContext oldContext, GameContext newContext, int? index = null)
    {
      MarkUpdateRequired(true);
    }

    protected abstract override GameContext DefaultGameContext();

    protected virtual float AnimationDuration => 0.3f;

    protected abstract Vector3 CalculateObjectPosition(int index, int count);

    protected virtual Vector3? CalculateObjectRotation(int index, int count) => null;

    void MarkUpdateRequired(bool animate)
    {
      _updateRequired = true;
      _animateNextUpdate |= animate;
    }

    void Insert(Displayable displayable, int? index)
    {
      displayable.Parent = this;
      if (!_objects.Contains(displayable))
      {
        if (index is { } i)
        {
          _objects.Insert(i, displayable);
        }
        else
        {
          _objects.Add(displayable);
        }
      }
    }

    void MoveObjectsToPosition(bool animate)
    {
      if (animate)
      {
        _animationRunning = true;
      }

      Sequence? sequence = null;
      if (animate)
      {
        sequence = TweenUtils.Sequence($"{gameObject.name} MoveObjectsToPosition");
      }

      for (var i = 0; i < _objects.Count; ++i)
      {
        var displayable = _objects[i];
        var position = CalculateObjectPosition(i, _objects.Count);
        var rotation = CalculateObjectRotation(i, _objects.Count);

        if (animate)
        {
          sequence.Insert(atPosition: 0, displayable.transform.DOMove(position, duration: AnimationDuration));
        }
        else
        {
          displayable.transform.position = position;
        }

        if (rotation is { } vector)
        {
          if (animate)
          {
            sequence.Insert(atPosition: 0,
              displayable.transform.DOLocalRotate(vector, duration: AnimationDuration));
          }
          else
          {
            displayable.transform.localEulerAngles = vector;
          }
        }

        displayable.SetGameContext(GameContext, 10 + i);
      }

      if (animate)
      {
        sequence.OnComplete(() =>
        {
          _animationRunning = false;
          _animateNextUpdate = false;
        });
      }
    }
  }
}