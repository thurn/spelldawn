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

using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class LinearCardDisplay : CardDisplay
  {
    [SerializeField] float _width;
    [SerializeField] float _initialSpacing;
    [SerializeField] float _cardSize;

    protected override SortingOrder.Type SortingType => SortingOrder.Type.Arena;

    protected override Vector3 CalculateCardPosition(int index, int count) =>
      transform.position + new Vector3(CalculateXOffset(_width, _initialSpacing, _cardSize, index, count), 0, 0);

    protected override Vector3? CalculateCardRotation(int index, int count) =>
      new Vector3(x: 270, y: 0, 0);

    public static float CalculateXOffset(float width, float initialSpacing, float cardWidth, int index, int count)
    {
      var availableWidth = Mathf.Min(width, (cardWidth + initialSpacing) * count);
      var offset = (availableWidth / 2f - cardWidth / 2f);

      return count switch
      {
        0 or 1 => 0,
        _ => Mathf.Lerp(-offset, offset, index / (count - 1f))
      };
    }

    void OnDrawGizmosSelected()
    {
      Gizmos.color = Color.blue;
      Gizmos.DrawSphere(transform.position + new Vector3(_width / 2f, 0, 0), radius: 1);
      Gizmos.DrawSphere(transform.position, radius: 1);
      Gizmos.DrawSphere(transform.position + new Vector3(_width / -2f, 0, 0), radius: 1);
    }
  }
}