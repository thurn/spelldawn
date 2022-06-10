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
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using UnityEditor.SceneManagement;
using UnityEngine;
using UnityEngine.SceneManagement;
using UnityEngine.TestTools;
using UnityEngine.TestTools.Graphics;
using Object = UnityEngine.Object;

public class EndToEndTests
{
    Registry Registry
    {
        get
        {
            var registries = Object.FindObjectsOfType<Registry>();
            Debug.Assert(registries.Length == 1);
            return registries[0];            
        }
    }

    bool _sceneLoaded;
    
    [UnityTest]
    public IEnumerator RunTestGame()
    {
        SceneManager.sceneLoaded += OnSceneLoaded;
        PlayerPrefs.DeleteAll();
        PlayerPrefs.SetInt(Preferences.OfflineMode, 1);
        
        yield return WaitUntilSceneLoaded(() =>
        {
            EditorSceneManager.LoadSceneAsyncInPlayMode(
                "Assets/Scenes/Labyrinth.unity",
                new LoadSceneParameters(LoadSceneMode.Additive));
        });
        
        Registry.GameService.CurrentGameId = null;
        Registry.GameService.PlayerId = new PlayerIdentifier { Value = 1 };
        
        yield return WaitUntilSceneLoaded(() =>
        {
            Registry.ActionService.HandleAction(new GameAction
            {
                CreateNewGame = new CreateNewGameAction
                {
                    Side = PlayerSide.Overlord,
                    OpponentId = new PlayerIdentifier
                    {
                        Value = 2
                    },
                    Deterministic = true,
                    UseDebugId = true
                }
            });
        });

        yield return new WaitForSeconds(1f);
        Debug.Assert(Registry.CardBrowser.AllObjects.Count == 5); 
        yield return new WaitForSeconds(1f);
    }

    IEnumerator WaitUntilSceneLoaded(Action action)
    {
        _sceneLoaded = false;
        action();
        return new WaitUntil(() => _sceneLoaded);
    }

    void OnSceneLoaded(Scene scene, LoadSceneMode mode)
    {
        Debug.Log($"OnSceneLoaded: {scene.name}");
        _sceneLoaded = true;
    }
}
