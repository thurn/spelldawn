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

#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using Spelldawn.Masonry;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEngine;
using UnityEngine.UIElements;
using TimeValue = Spelldawn.Protos.TimeValue;

namespace Spelldawn.Services
{
  public sealed class UpdateInterfaceService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    public HashSet<VisualElement> HiddenForAnimation { get; } = new();

    public IEnumerator HandleUpdateInterface(UpdateInterfaceElementCommand command) =>
      command.InterfaceUpdateCase switch
      {
        UpdateInterfaceElementCommand.InterfaceUpdateOneofCase.AnimateToChildIndex => HandleAnimateToChildIndex(
          command.ElementName, command.AnimateToChildIndex),
        UpdateInterfaceElementCommand.InterfaceUpdateOneofCase.AnimateToElementPosition =>
          HandleAnimateToElementPosition(command.ElementName, command.AnimateToElementPosition),
        UpdateInterfaceElementCommand.InterfaceUpdateOneofCase.Destroy => HandleDestroy(command.ElementName),
        UpdateInterfaceElementCommand.InterfaceUpdateOneofCase.UpdateText => HandleUpdateText(command.ElementName,
          command.UpdateText),
        _ => throw new ArgumentOutOfRangeException()
      };

    IEnumerator HandleUpdateText(string elementName, UpdateText command)
    {
      Debug.Log($"HandleUpdateText");
      yield break;
    }

    IEnumerator HandleDestroy(string elementName)
    {
      var element = FindElement<VisualElement>(elementName);
      element.RemoveFromHierarchy();
      yield break;
    }

    IEnumerator HandleAnimateToElementPosition(string elementName, AnimateToElementPositionAndDestroy command)
    {
      var element = FindElement<VisualElement>(elementName);
      var target = FindElementOptional<VisualElement>(command.TargetElementName) ?? 
                   FindElement<VisualElement>(command.FallbackTargetElementName);
      yield return AnimateToPositionAndDestroy(element, target.worldBound, command.Duration).WaitForCompletion();
    }

    IEnumerator HandleAnimateToChildIndex(string elementName, AnimateDraggableToChildIndex command)
    {
      var element = FindElement<Draggable>(elementName);
      var parent = FindElement<VisualElement>(command.ParentElementName);

      var newElement = Mason.Render(_registry, element.Node);
      if (parent.childCount <= command.Index)
      {
        parent.Add(newElement);
      }
      else
      {
        parent.Insert((int)command.Index, newElement);
      }

      var targetHeight = newElement.style.height;
      newElement.style.opacity = 0f;
      newElement.style.height = 0f;
      newElement.style.transitionProperty = new StyleList<StylePropertyName>(
        new List<StylePropertyName> { "height" });
      newElement.style.transitionDuration = new StyleList<UnityEngine.UIElements.TimeValue>(
        new List<UnityEngine.UIElements.TimeValue> { new(command.Duration.Milliseconds, TimeUnit.Millisecond) });
      HiddenForAnimation.Add(newElement);

      yield return new WaitForEndOfFrame();

      newElement.style.height = targetHeight;
      yield return AnimateToPositionAndDestroy(element, newElement.worldBound, command.Duration, () =>
      {
        newElement.style.opacity = 1.0f;
        HiddenForAnimation.Remove(newElement);
      }).WaitForCompletion();
    }

    public static Sequence AnimateToPositionAndDestroy(
      VisualElement element,
      Rect worldBound,
      TimeValue duration,
      Action? onComplete = null)
    {
      var targetPosition = worldBound.position -
                           new Vector2(element.style.marginLeft.value.value, element.style.marginTop.value.value);
      return TweenUtils.Sequence("AnimateToPositionAndDestroy").Insert(0,
          DOTween.To(() => element.style.left.value.value,
            x => element.style.left = x,
            endValue: targetPosition.x, duration.Milliseconds / 1000f))
        .Insert(0,
          DOTween.To(() => element.style.top.value.value,
            y => element.style.top = y,
            endValue: targetPosition.y, duration.Milliseconds / 1000f))
        .AppendCallback(() =>
        {
          element.RemoveFromHierarchy();
          onComplete?.Invoke();
        });
    }

    public static Sequence AnimateToZeroHeightAndDestroy(VisualElement element, TimeValue duration)
    {
      return TweenUtils.Sequence("AnimateToZeroHeightAndDestroy").Append(
        DOTween.To(() => element.style.height.value.value,
          height => element.style.height = height,
          endValue: 0,
          duration.Milliseconds / 1000f))
        .AppendCallback(element.RemoveFromHierarchy);
    }

    T FindElement<T>(string elementName) where T : VisualElement
    {
      var results = _registry.DocumentService.RootVisualElement.Query<T>(elementName).Build();
      Errors.CheckState(results.Count() == 1, $"Expected exactly 1 {elementName} but got {results.Count()}");
      return results.First();
    }
    
    T? FindElementOptional<T>(string elementName) where T : VisualElement
    {
      var results = _registry.DocumentService.RootVisualElement.Query<T>(elementName).Build();
      switch (results.Count())
      {
        case 0:
          return null;
        case 1:
          return results.First();
        default:
          throw new InvalidOperationException($"Found more than one element named {elementName}");
      }
    }    
  }
}