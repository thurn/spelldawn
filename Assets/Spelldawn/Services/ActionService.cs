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
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Text;
using Grpc.Core;
using Grpc.Net.Client;
using Grpc.Net.Client.Web;
using Grpc.Net.Compression;
using Spelldawn.Protos;
using Spelldawn.Tools;
using Spelldawn.Utils;
using UnityEngine;
using CompressionLevel = System.IO.Compression.CompressionLevel;
using Random = UnityEngine.Random;

namespace Spelldawn.Services
{
  public sealed class ActionService : MonoBehaviour
  {
    const string LocalServerAddress = "http://localhost";
    const string ProductionServerAddress = "http://localhost";

    readonly Lazy<Protos.Spelldawn.SpelldawnClient> _client = new(() => new Protos.Spelldawn.SpelldawnClient(
      GrpcChannel.ForAddress(
        UseProductionServer.ShouldUseProductionServer ? ProductionServerAddress : LocalServerAddress,
        new GrpcChannelOptions
        {
          HttpHandler = new GrpcWebHandler(new HttpClientHandler()),
          Credentials = ChannelCredentials.Insecure,
          CompressionProviders = new List<ICompressionProvider>
          {
            new GzipCompressionProvider(CompressionLevel.Optimal)
          }
        })));

    [SerializeField] Registry _registry = null!;
    [SerializeField] bool _currentlyHandlingAction;
    readonly Queue<ClientAction> _actionQueue = new();
    PlayerIdentifier? _playerIdentifier;
    bool _attemptReconnect;

    public bool Active => _currentlyHandlingAction || _actionQueue.Count > 0;

    void Start()
    {
      _attemptReconnect = false;
      StartCoroutine(AutoReconnect());
    }

    public void Connect(PlayerIdentifier playerIdentifier)
    {
      _playerIdentifier = playerIdentifier;
      AttemptConnection();
    }

    public void HandleAction(ClientAction action)
    {
      if (!_registry.CapabilityService.CanExecuteAction(action.ActionCase))
      {
        var message = new StringBuilder();
        message.Append($"Error: User cannot currently perform action {action}");
        throw new InvalidOperationException(message.ToString());
      }

      if (action.ActionCase == ClientAction.ActionOneofCase.StandardAction)
      {
        if (_currentlyHandlingAction)
        {
          // I would like to eventually handle multiple concurrent StandardActions, but it requires
          // more robust testing, for example to ensure that multiple optimistic interface updates
          // work with any sequence of mutations & responses. For now we simply ignore button clicks
          // while an RPC is pending.
          Debug.Log($"Silently dropping StandardAction");
        }
        else
        {
          ApplyImmediateResponse(action.StandardAction);
          _actionQueue.Enqueue(action);          
        }
      }
      else
      {
        _actionQueue.Enqueue(action);
      }
    }

    void Update()
    {
      if (_actionQueue.Count > 0 && !_currentlyHandlingAction)
      {
        _currentlyHandlingAction = true;
        StartCoroutine(HandleActionAsync(_actionQueue.Dequeue()));
      }

#if USE_UNITY_PLUGIN
      var pollCommands = Plugin.Poll();
      if (pollCommands != null)
      { 
        StartCoroutine(_registry.CommandService.HandleCommands(pollCommands));
      }
#endif
    }

    IEnumerator AutoReconnect()
    {
      while (true)
      {
        yield return new WaitForSeconds(1.0f);
        if (_attemptReconnect)
        {
          AttemptConnection();
        }
      }

      // ReSharper disable once IteratorNeverReturns
    }

    async void AttemptConnection()
    {
      _registry.DocumentService.Loading = true;

      var request = new ConnectRequest
      {
        PlayerId = Errors.CheckNotNull(_playerIdentifier),
      };

      // TODO: Android in particular seems to hang for multiple minutes when the server can't be reached?
      using var call = _client.Value.Connect(request);

      try
      {
        while (await call.ResponseStream.MoveNext())
        {
          if (this != null)
          {
            var commands = call.ResponseStream.Current;
            _attemptReconnect = false;
            StartCoroutine(_registry.CommandService.HandleCommands(commands,
              () => { _registry.DocumentService.Loading = false; }));
            _registry.DocumentService.FetchOpenPanelsOnConnect();
          }
        }
      }
      catch (RpcException e)
      {
        _registry.DocumentService.Loading = true;
        _attemptReconnect = true;
        if (!DoNotLogRpcErrors.ShouldSkipLoggingRpcErrors)
        {
          Debug.Log($"RpcException: {e.StatusCode} -- {e.Message}");
        }
      }
    }

    IEnumerator HandleActionAsync(ClientAction action)
    {
      StartCoroutine(ApplyOptimisticResponse(action));
      if (action.ActionCase == ClientAction.ActionOneofCase.StandardAction)
      {
        if (action.StandardAction.Payload.Length == 0)
        {
          // No need to send empty payload to server
          _currentlyHandlingAction = false;
          yield break;
        }

        _registry.DocumentService.AddRequestFields(action.StandardAction);
      }

      // Send to server
      var request = new GameRequest
      {
        Action = action,
        PlayerId = Errors.CheckNotNull(_playerIdentifier),
      };
      request.OpenPanels.AddRange(_registry.DocumentService.OpenPanels);

      float startTime = 0;
      if (LogRpcTime.ShouldLogRpcTime)
      {
        Debug.Log($"Sending {request.Action.ActionCase}");
        startTime = Time.time;
      }

      // Introduce simulated server delay
      if (IntroduceNetworkDelay.ShouldIntroduceLongNetworkDelay)
      {
        yield return new WaitForSeconds(5f);
      }
      else if (!NoNetworkDelay.ShouldRemoveNetworkDelay)
      {
        yield return new WaitForSeconds(Random.Range(0f, 0.5f));
      }

      var call = _client.Value.PerformActionAsync(request);
      var task = call.GetAwaiter();
      yield return new WaitUntil(() => task.IsCompleted);

      switch (call.GetStatus().StatusCode)
      {
        case StatusCode.OK:
          _registry.DocumentService.Loading = false;
          _attemptReconnect = false;
          if (LogRpcTime.ShouldLogRpcTime)
          {
            Debug.Log($"Got response in {(Time.time - startTime) * 1000} milliseconds");
          }

          yield return _registry.CommandService.HandleCommands(task.GetResult());
          break;
        default:
          _registry.DocumentService.Loading = true;
          _attemptReconnect = true;
          if (!DoNotLogRpcErrors.ShouldSkipLoggingRpcErrors)
          {
            Debug.Log($"Error connecting to {LocalServerAddress}: {call.GetStatus().Detail}");
          }

          break;
      }

      _currentlyHandlingAction = false;
    }

    /// <summary>
    /// Immediate action handling, without waiting for the queue. This is needed to avoid things that feel
    /// broken, like waiting for animations before closing a panel.
    /// </summary>
    void ApplyImmediateResponse(StandardAction action)
    {
      _registry.StaticAssets.PlayButtonSound();
      if (action.Update is { } update)
      {
        foreach (var command in update.Commands)
        {
          switch (command.CommandCase)
          {
            case GameCommand.CommandOneofCase.TogglePanel:
              _registry.DocumentService.TogglePanel(command.TogglePanel);
              break;
          }
        }
      }
    }

    IEnumerator ApplyOptimisticResponse(ClientAction action)
    {
      switch (action.ActionCase)
      {
        case ClientAction.ActionOneofCase.StandardAction:
          if (action.StandardAction.Update is { } update)
          {
            // Immediate commands skip the queue and are processed by ApplyImmediateResponse() above 
            var list = new CommandList();
            list.Commands.AddRange(
              update.Commands.Where(c =>
                c.CommandCase != GameCommand.CommandOneofCase.TogglePanel));
            yield return _registry.CommandService.HandleCommands(list);
          }

          break;
        case ClientAction.ActionOneofCase.DrawCard:
          _registry.StaticAssets.PlayDrawCardStartSound();
          _registry.ActionDisplayForPlayer(PlayerName.User).SpendActions(1);
          _registry.CardService.DrawOptimisticCard();
          break;
        case ClientAction.ActionOneofCase.PlayCard:
          yield return HandlePlayCard(action.PlayCard);
          break;
        case ClientAction.ActionOneofCase.GainMana:
          _registry.StaticAssets.PlayAddManaSound();
          _registry.ActionDisplayForPlayer(PlayerName.User).SpendActions(1);
          _registry.ManaDisplayForPlayer(PlayerName.User).GainMana(1);
          break;
        case ClientAction.ActionOneofCase.InitiateRaid:
          _registry.ActionDisplayForPlayer(PlayerName.User).SpendActions(1);
          yield return _registry.CommandService.HandleCommands(new GameCommand
          {
            VisitRoom = new VisitRoomCommand
            {
              RoomId = action.InitiateRaid.RoomId,
              Initiator = PlayerName.User,
              VisitType = RoomVisitType.InitiateRaid
            }
          });
          break;
        case ClientAction.ActionOneofCase.LevelUpRoom:
          _registry.ActionDisplayForPlayer(PlayerName.User).SpendActions(1);
          yield return _registry.CommandService.HandleCommands(new GameCommand
          {
            VisitRoom = new VisitRoomCommand
            {
              RoomId = action.LevelUpRoom.RoomId,
              Initiator = PlayerName.User,
              VisitType = RoomVisitType.LevelUpRoom
            }
          });
          break;
        default:
          yield break;
      }
    }

    IEnumerator HandlePlayCard(PlayCardAction action)
    {
      var card = _registry.CardService.FindCard(action.CardId);
      _registry.StaticAssets.PlayWhooshSound();
      if (card.ReleasePosition == null)
      {
        yield break;
      }

      var position = card.ReleasePosition;

      if (position.PositionCase == ObjectPosition.PositionOneofCase.Room)
      {
        var room = action.Target.RoomId;
        Errors.CheckArgument(room != RoomIdentifier.Unspecified, "No RoomId target provided!");
        // Move to targeted room
        var newPosition = new ObjectPosition();
        newPosition.MergeFrom(position);
        newPosition.Room.RoomId = room;
        position = newPosition;
      }

      yield return _registry.ObjectPositionService.MoveGameObject(card, position);
    }
  }

  /// <summary>
  /// You can use this type instead of 'GzipCompressionProvider' above to log the size of server payloads before
  /// decompression.
  /// </summary>
  // ReSharper disable once UnusedType.Global
  sealed class DebugGzipCompressionProvider : ICompressionProvider
  {
    readonly GzipCompressionProvider _wrappedProvider;

    public DebugGzipCompressionProvider(CompressionLevel defaultCompressionLevel)
    {
      _wrappedProvider = new GzipCompressionProvider(defaultCompressionLevel);
    }

    public Stream CreateCompressionStream(Stream stream, CompressionLevel? compressionLevel) =>
      _wrappedProvider.CreateCompressionStream(stream, compressionLevel);

    public Stream CreateDecompressionStream(Stream stream)
    {
      Debug.Log($">>> Decompressing: {stream.Length}");
      return _wrappedProvider.CreateDecompressionStream(stream);
    }

    public string EncodingName => _wrappedProvider.EncodingName;
  }
}