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

using System;
using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using Spelldawn.Masonry;
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using TMPro;
using UnityEngine;
using UnityEngine.Serialization;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class Card : Displayable, ArrowService.IArrowDelegate
  {
    public const float CardScale = 1.5f;

    [Header("Card")] [SerializeField] SpriteRenderer _cardBack = null!;
    [SerializeField] GameObject _arenaCardBack = null!;
    [SerializeField] Transform _cardFront = null!;
    [SerializeField] GameObject _cardShadow = null!;
    [SerializeField] Transform _arenaCard = null!;
    [SerializeField] SpriteRenderer _image = null!;
    [SerializeField] SpriteRenderer _frame = null!;
    [SerializeField] SpriteRenderer _titleBackground = null!;
    [SerializeField] MeshRenderer _outline = null!;
    [SerializeField] TextMeshPro _title = null!;
    [SerializeField] WarpText _warpText = null!;
    [SerializeField] TextMeshPro _rulesText = null!;
    [SerializeField] SpriteRenderer? _jewel;
    [SerializeField] SpriteRenderer _arenaFrame = null!;
    [SerializeField] SpriteRenderer _faceDownArenaFrame = null!;
    [SerializeField] GameObject _arenaShadow = null!;
    [SerializeField] Transform _topLeftAnchor = null!;
    [SerializeField] Transform _topRightAnchor = null!;
    [SerializeField] Transform _bottomLeftAnchor = null!;
    [SerializeField] Transform _bottomRightAnchor = null!;
    [SerializeField] Icon _topLeftIcon = null!;
    [SerializeField] Icon _topRightIcon = null!;
    [SerializeField] Icon _bottomRightIcon = null!;
    [SerializeField] Icon _bottomLeftIcon = null!;
    [SerializeField] Icon _arenaIcon = null!;
    [SerializeField] bool _isRevealed;
    [SerializeField] float _dragStartScreenZ;
    [SerializeField] Vector3 _dragStartPosition;
    [SerializeField] Vector3 _dragOffset;
    [SerializeField] Quaternion _initialDragRotation;
    [SerializeField] ObjectDisplay? _previousParent;

    [SerializeField] ObjectDisplay? _containedObjectsDisplay;

    // Minor hack: we want to shift the image down to be centered within the card in the arena, so we store
    // the image position here to restore it later.
    [SerializeField] float _arenaCardYOffset;

    // Desired size for the card image in px. Images in token cards are slightly smaller, so we customize this
    // for each prefab.
    [FormerlySerializedAs("_referenceWidth")] [SerializeField]
    float _referenceImageWidth;

    CardIdentifier? _cardId;
    bool? _serverCanPlay;
    bool? _serverRevealedInArena;
    ISet<RoomIdentifier>? _validRoomTargets;
    ObjectPosition? _releasePosition;
    Node? _supplementalInfo;
    ArrowService.Type? _arrowOnDrag;

    [Serializable]
    public sealed class Icon
    {
      [SerializeField] SpriteRenderer _background = null!;
      public SpriteRenderer Background => _background;
      [SerializeField] TextMeshPro _text = null!;
      public TextMeshPro Text => _text;
    }

    public Registry Registry { get; set; } = null!;

    public bool IsRevealed => _isRevealed;

    bool InHand() => HasGameContext && GameContext == GameContext.Hand;

    public override float DefaultScale => CardScale;

    public ObjectPosition? ReleasePosition => _releasePosition;

    public ObjectPosition? DestroyPosition { get; private set; }

    public Transform TopLeftAnchor => _topLeftAnchor;

    public Transform TopRightAnchor => _topRightAnchor;

    public Transform BottomLeftAnchor => _bottomLeftAnchor;

    public Transform BottomRightAnchor => _bottomRightAnchor;

    public Node? SupplementalInfo => _supplementalInfo;

    public ObjectDisplay ContainedObjects => Errors.CheckNotNull(_containedObjectsDisplay);

    public Sequence? Render(
      CardView cardView,
      GameContext? gameContext = null,
      bool animate = true)
    {
      _cardId = cardView.CardId;

      if (gameContext is { } gc)
      {
        SetGameContext(gc);
      }

      if (cardView.OwningPlayer != PlayerName.Unspecified)
      {
        Registry.AssetService.AssignSprite(_cardBack, Registry.CardService.GetCardBack(cardView.OwningPlayer));
      }

      _outline.sortingOrder = -1;
      Registry.AssetService.AssignSprite(_arenaFrame, cardView.ArenaFrame);
      Registry.AssetService.AssignSprite(_faceDownArenaFrame, cardView.FaceDownArenaFrame);
      DestroyPosition = cardView.DestroyPosition;

      if (cardView.RevealedToViewer)
      {
        if (_isRevealed)
        {
          RenderCardView(cardView);
          return null;
        }
        else
        {
          return Flip(_cardFront, _cardBack, () => RenderCardView(cardView), animate);
        }
      }
      else
      {
        if (_isRevealed)
        {
          return Flip(_cardBack, _cardFront, () => RenderCardView(cardView), animate);
        }
        else
        {
          RenderCardView(cardView);
          return null;
        }
      }
    }

    void Update()
    {
      if (Registry.GlobalGameMode == GlobalGameMode.Default)
      {
        // Outline is too fiery for screenshot tests
        _outline.gameObject.SetActive(CanPlay());
      }
    }

    bool CanPlay() => _serverCanPlay == true && InHand() && Registry.CapabilityService.CanInitiateAction() &&
                      _isRevealed;

    public Card Clone()
    {
      var result = ComponentUtils.GetComponent<Card>(Instantiate(gameObject));
      result._cardId = _cardId;
      result._outline.enabled = false;
      result._serverCanPlay = _serverCanPlay;
      result._serverRevealedInArena = _serverRevealedInArena;
      result._validRoomTargets = _validRoomTargets;
      result._releasePosition = _releasePosition;
      result._supplementalInfo = _supplementalInfo;
      result.Registry = Registry;
      return result;
    }

    public Sequence? TurnFaceDown(bool animate)
      => Flip(_cardBack, _cardFront, () => { }, animate);

    protected override void OnSetGameContext(GameContext oldContext, GameContext newContext)
    {
      if (newContext.RenderArenaCard())
      {
        _arenaCardBack.SetActive(!_isRevealed);
        _cardBack.maskInteraction = SpriteMaskInteraction.VisibleInsideMask;
        _frame.gameObject.SetActive(false);
        _titleBackground.gameObject.SetActive(false);
        _title.gameObject.SetActive(false);
        _rulesText.gameObject.SetActive(false);
        if (_jewel)
        {
          _jewel!.gameObject.SetActive(false);
        }

        _arenaFrame.gameObject.SetActive(true);
        _cardShadow.SetActive(false);
        _arenaShadow.SetActive(true);
        // In Arena mode, we want the image content to be centered within the card, so we shift
        // it around.
        _arenaCard.position = transform.position;
      }
      else
      {
        _arenaCardBack.SetActive(false);
        _cardBack.maskInteraction = SpriteMaskInteraction.None;
        _frame.gameObject.SetActive(true);
        _titleBackground.gameObject.SetActive(true);
        _title.gameObject.SetActive(true);
        _rulesText.gameObject.SetActive(true);
        if (_jewel)
        {
          _jewel!.gameObject.SetActive(true);
        }

        _arenaFrame.gameObject.SetActive(false);
        _cardShadow.SetActive(true);
        _arenaShadow.SetActive(false);
        _arenaCard.localPosition = new Vector3(0, _arenaCardYOffset, 0);
      }

      UpdateIcons(null, GameContext.RenderArenaCard());
      UpdateRevealedToOpponent(GameContext.RenderArenaCard());
    }

    public override bool CanHandleMouseDown()
    {
      if (Registry.CapabilityService.CanInfoZoom(this, GameContext) && _isRevealed)
      {
        return true;
      }

      return InHand() && CanPlay();
    }

    // I originally did all of this using Unity's OnMouseDown events, but they were not reliable
    // enough for me in testing and the UI sometimes get stuck. Some person on reddit told me it
    // always works for them and my code is probably wrong... cool.
    public override void MouseDown()
    {
      if (Registry.CapabilityService.CanInfoZoom(this, GameContext) && _isRevealed)
      {
        Registry.StaticAssets.PlayCardSound();
        Registry.CardService.DisplayInfoZoom(
          WorldMousePosition(Registry, Registry.MainCamera.WorldToScreenPoint(gameObject.transform.position).z),
          this);
      }

      if (InHand() && CanPlay())
      {
        Registry.CardService.CurrentlyDragging = true;
        SetGameContext(GameContext.Dragging);
        _previousParent = Parent;
        _previousParent!.RemoveObject(this);
        _outline.gameObject.SetActive(false);
        _initialDragRotation = transform.rotation;
        _dragStartScreenZ = Registry.MainCamera.WorldToScreenPoint(gameObject.transform.position).z;
        _dragStartPosition = WorldMousePosition(Registry, _dragStartScreenZ);
        _dragOffset = gameObject.transform.position - _dragStartPosition;
      }
    }

    public override void MouseDrag()
    {
      if (!Registry.CardService.CurrentlyDragging)
      {
        return;
      }

      var mousePosition = WorldMousePosition(Registry, _dragStartScreenZ);
      var distanceDragged = Vector2.Distance(mousePosition, _dragStartPosition);
      var t = Mathf.Clamp01(distanceDragged / 5);
      transform.position = _dragOffset + mousePosition;
      var rotation = Quaternion.Slerp(_initialDragRotation, Quaternion.Euler(280, 0, 0), t);
      transform.rotation = rotation;

      if (distanceDragged > 0.25f)
      {
        Registry.CardService.ClearInfoZoom();
      }

      if (Registry.CardService.IsMouseOverPlayCardArea())
      {
        if (_validRoomTargets != null)
        {
          Registry.ArenaService.ShowRoomSelectorForMousePosition(_validRoomTargets);
        }

        if (_arrowOnDrag is { } arrow)
        {
          gameObject.SetActive(false);
          Registry.ArrowService.ShowArrow(arrow, Registry.IdentityCardForPlayer(PlayerName.User).transform, this);
        }
      }
      else
      {
        Registry.ArenaService.HideRoomSelector();
      }
    }

    public override void MouseUp()
    {
      if (_arrowOnDrag != null)
      {
        // GameObject will be disabled when arrow is being shown 
        gameObject.SetActive(true);
        Registry.ArrowService.HideArrows();
      }

      if (!Registry.CardService.CurrentlyDragging)
      {
        Registry.StaticAssets.PlayCardSound();
        return;
      }

      Registry.CardService.CurrentlyDragging = false;

      if (ShouldReturnToHandOnRelease())
      {
        Registry.StaticAssets.PlayCardSound();
        StartCoroutine(_previousParent!.AddObject(this, animate: true));
        Registry.ArenaService.HideRoomSelector();
        return;
      }

      var action = new PlayCardAction
      {
        CardId = Errors.CheckNotNull(_cardId)
      };

      if (_validRoomTargets != null)
      {
        var roomId = Errors.CheckNotDefault(Errors.CheckNotNull(Registry.ArenaService.CurrentRoomSelector).RoomId);
        Errors.CheckState(_validRoomTargets.Contains(roomId), "Invalid Room selected");
        action.Target = new CardTarget
        {
          RoomId = roomId
        };
      }

      Registry.ArenaService.HideRoomSelector();

      Registry.ActionService.HandleAction(new ClientAction
      {
        PlayCard = action
      });
    }

    static Vector3 WorldMousePosition(Registry registry, float dragStartScreenZ) =>
      registry.MainCamera.ScreenToWorldPoint(
        new Vector3(Input.mousePosition.x, Input.mousePosition.y, dragStartScreenZ));

    bool ShouldReturnToHandOnRelease()
    {
      if (!Registry.CapabilityService.CanExecuteAction(ClientAction.ActionOneofCase.PlayCard))
      {
        return true;
      }

      if (_validRoomTargets == null)
      {
        return !Registry.CardService.IsMouseOverPlayCardArea();
      }
      else
      {
        return !Registry.ArenaService.CurrentRoomSelector;
      }
    }

    static Sequence? Flip(Component faceUp, Component faceDown, Action onFlipped, bool animate)
    {
      if (animate)
      {
        const float duration = TweenUtils.FlipAnimationDurationSeconds / 2f;
        return TweenUtils.Sequence($"{faceUp.transform.parent.gameObject.name} Flip")
          .Insert(atPosition: 0, faceDown.transform.DOLocalRotate(new Vector3(x: 0, y: 90, z: 0), duration))
          .InsertCallback(atPosition: duration, () =>
          {
            faceUp.gameObject.SetActive(value: true);
            faceUp.transform.localRotation = Quaternion.Euler(x: 0, y: -90, z: 0);
            faceDown.gameObject.SetActive(value: false);
            onFlipped();
          })
          .Insert(atPosition: duration, faceUp.transform.DOLocalRotate(Vector3.zero, duration));
      }
      else
      {
        faceUp.gameObject.SetActive(value: true);
        faceDown.gameObject.SetActive(value: false);
        onFlipped();
        return null;
      }
    }

    void RenderCardView(CardView card)
    {
      _serverRevealedInArena = card.RevealedToViewer && card.IsFaceUp;

      if (card.RevealedToViewer && card.RevealedCard != null)
      {
        RenderRevealedCard(card.RevealedCard);
      }
      else if (!card.RevealedToViewer)
      {
        RenderHiddenCard();
      }

      UpdateIcons(card.CardIcons, GameContext.RenderArenaCard());
      UpdateRevealedToOpponent(GameContext.RenderArenaCard());
    }

    void RenderRevealedCard(RevealedCardView revealed)
    {
      _isRevealed = true;
      _arenaCardBack.SetActive(false);
      gameObject.name = revealed.Title.Text + IdString();
      _validRoomTargets = null;
      _serverCanPlay = false;
      _arrowOnDrag = null;

      switch (revealed.Targeting?.TargetingCase)
      {
        case CardTargeting.TargetingOneofCase.NoTargeting:
          _validRoomTargets = null;
          _serverCanPlay = revealed.Targeting.NoTargeting.CanPlay;
          break;
        case CardTargeting.TargetingOneofCase.PlayInRoom:
          _validRoomTargets = revealed.Targeting.PlayInRoom.ValidRooms.ToHashSet();
          _serverCanPlay = _validRoomTargets.Count > 0;
          break;
        case CardTargeting.TargetingOneofCase.ArrowTargetRoom:
          _validRoomTargets = revealed.Targeting.ArrowTargetRoom.ValidRooms.ToHashSet();
          _serverCanPlay = _validRoomTargets.Count > 0;
          _arrowOnDrag = revealed.Targeting.ArrowTargetRoom.Arrow switch
          {
            TargetingArrow.Red => ArrowService.Type.Red,
            TargetingArrow.Blue => ArrowService.Type.Blue,
            TargetingArrow.Green => ArrowService.Type.Green,
            _ => null
          };
          break;
        default:
          _validRoomTargets = null;
          _serverCanPlay = false;
          break;
      }

      if (revealed.OnReleasePosition is { } position)
      {
        _releasePosition = position;
      }

      if (revealed.SupplementalInfo is { } info)
      {
        _supplementalInfo = info;
      }

      _cardBack.gameObject.SetActive(value: false);
      _cardFront.gameObject.SetActive(value: true);
      Registry.AssetService.AssignSprite(_image, revealed.Image, referenceWidth: _referenceImageWidth);
      _image.gameObject.SetActive(true);
      Registry.AssetService.AssignSprite(_frame, revealed.CardFrame);
      Registry.AssetService.AssignSprite(_titleBackground, revealed.TitleBackground);
      SetTitle(revealed.Title.Text);
      
      _title.color = Mason.ToUnityColor(revealed.Title.TextColor);
      if (revealed.RulesText?.Text != null)
      {
        _rulesText.text = revealed.RulesText.Text;
      }

      if (_jewel)
      {
        Registry.AssetService.AssignSprite(_jewel!, revealed.Jewel);
      }
    }

    void SetTitle(string title)
    {
      if (_title.text != title)
      {
        _title.text = title;
        StartCoroutine(_warpText.Warp());            
      }
    }

    void RenderHiddenCard()
    {
      _isRevealed = false;
      gameObject.name = "Hidden Card" + IdString();
      _cardBack.gameObject.SetActive(value: true);
      _cardFront.gameObject.SetActive(value: false);
    }

    void UpdateIcons(CardIcons? cardIcons, bool inArena)
    {
      SetCardIcon(_topLeftIcon, cardIcons?.TopLeftIcon, !inArena);
      SetCardIcon(_topRightIcon, cardIcons?.TopRightIcon, !inArena);
      SetCardIcon(_bottomRightIcon, cardIcons?.BottomRightIcon, !inArena);
      SetCardIcon(_bottomLeftIcon, cardIcons?.BottomLeftIcon, !inArena);
      SetCardIcon(_arenaIcon, cardIcons?.ArenaIcon, inArena);
    }

    void UpdateRevealedToOpponent(bool inArena)
    {
      if (inArena && _serverRevealedInArena != true)
      {
        _image.color = Color.gray;
        _arenaFrame.color = Color.gray;
      }
      else
      {
        _image.color = Color.white;
        _arenaFrame.color = Color.white;
      }
    }

    void SetCardIcon(Icon icon, CardIcon? cardIcon, bool show)
    {
      var iconContainer = icon.Background.transform.parent;
      if (cardIcon != null)
      {
        Registry.AssetService.AssignSprite(icon.Background, cardIcon.Background);
        icon.Background.transform.localScale = (cardIcon.BackgroundScale ?? 1.0f) * Vector3.one;
        icon.Text.text = cardIcon.Text;
      }
      
      iconContainer.gameObject.SetActive(show);
    }

    public void OnArrowMoved(Vector3 position)
    {
      if (!Registry.CardService.IsMouseOverPlayCardArea())
      {
        Registry.ArrowService.HideArrows();
        gameObject.SetActive(true);
      }
    }

    public void OnArrowReleased(Vector3 position)
    {
    }

    string IdString()
    {
      var side = _cardId?.Side switch
      {
        PlayerSide.Overlord => "O", PlayerSide.Champion => "O", _ => "??"
      };

      if (_cardId?.AbilityId != null)
      {
        return $" {side}{_cardId?.Index}[{_cardId?.AbilityId}]";
      }
      else
      {
        return $" {side}{_cardId?.Index}";
      }
    }
  }
}