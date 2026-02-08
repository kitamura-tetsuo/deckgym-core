import pytest
import numpy as np
import pyspiel
from deckgym import PyGameState
from deckgym_openspiel.game import DeckGymGame
from deckgym_openspiel.state import DeckGymState

CARD_IDS = [
    "A1 001", "A1 001",
    "A1 002", "A1 002",
    "A1 003", "A1 003",
    "A1 004", "A1 004",
    "A1 005", "A1 005",
    "A1 006", "A1 006",
    "A1 007", "A1 007",
    "A1 008", "A1 008",
    "A1 009", "A1 009",
    "A1 010", "A1 010",
]

@pytest.fixture
def game():
    return DeckGymGame(CARD_IDS, CARD_IDS, seed=42)

def test_tensor_shapes(game):
    state = game.new_initial_state()
    obs_shape = game.observation_tensor_shape()
    info_shape = game.information_state_tensor_shape()

    assert obs_shape == info_shape

    obs_tensor = state.observation_tensor(0)
    info_tensor = state.information_state_tensor(0)

    assert len(obs_tensor) == obs_shape[0]
    assert len(info_tensor) == info_shape[0]

def test_tensor_structure(game):
    state = game.new_initial_state()
    player = state.current_player()
    tensor = state.information_state_tensor(player)

    # Validate tensor size and infer Card Count C
    # Size = 6 (Globals) + 8 * (16 + C) + C (MyHand) + C (OppHand) + 2 (DeckCounts) + C (MyDisc) + C (OppDisc)
    #      = 8 + 128 + 8C + 4C = 136 + 12C

    tensor_len = len(tensor)
    assert (tensor_len - 136) % 12 == 0
    C = (tensor_len - 136) // 12
    print(f" inferred Card Count: {C}")

    # Section Offsets
    # 1. Globals: 0-6
    # 2. My Active: 6 - (6 + 16 + C)
    pokemon_slot_size = 16 + C

    # HP is at offset 0 of pokemon slot
    hp_idx = 6
    hp_val = tensor[hp_idx]

    # Initially no active pokemon
    assert hp_val == 0.0

    # Check Hand Counts (Indices 4, 5)
    # Start of game, hands should have cards (5 each)
    # Indices: 0=Turn, 1=PointsMe, 2=PointsOpp, 3=CurrentPlayerOneHot
    # 4=MyHandCount, 5=OppHandCount
    p0_hand_count = tensor[4]
    p1_hand_count = tensor[5]
    assert p0_hand_count == 5.0
    assert p1_hand_count == 5.0

    # Check Hand Masking
    obs_tensor = state.observation_tensor(player)
    info_tensor = state.information_state_tensor(player)

    # My Hand starts after 8 pokemon slots
    # 6 (Globals) + 2 (HandCounts) -> Actually Globals includes HandCounts in my description above?
    # In encoding.rs:
    # 1. Turn Info (4 floats)
    # 2. Hand Counts (2 floats) -> Total 6 floats before pokemon.
    # So Pokemon start at index 6. Correct.

    my_hand_start = 6 + 8 * pokemon_slot_size
    my_hand_end = my_hand_start + C

    my_hand_info = info_tensor[my_hand_start:my_hand_end]
    my_hand_obs = obs_tensor[my_hand_start:my_hand_end]

    # Info tensor should have cards (hand not empty at start)
    assert sum(my_hand_info) > 0, "My Hand should not be empty in info state"

    # Obs tensor should be all zeros (masked)
    assert sum(my_hand_obs) == 0, "My Hand should be masked in observation"

    # Opponent Hand (next section)
    opp_hand_start = my_hand_end
    opp_hand_end = opp_hand_start + C

    opp_hand_info = info_tensor[opp_hand_start:opp_hand_end]
    opp_hand_obs = obs_tensor[opp_hand_start:opp_hand_end]

    # Should be zero in BOTH (always masked)
    assert sum(opp_hand_info) == 0
    assert sum(opp_hand_obs) == 0

    # Test taking action to place pokemon (to verify HP update)
    # Find a Place action
    legal_actions = state.legal_actions(player)
    place_action = None
    for action in legal_actions:
        action_str = state.action_to_string(player, action)
        if "Place" in action_str and ", 0)" in action_str: # Place at slot 0 (Active)
            place_action = action
            break

    if place_action is not None:
        state.apply_action(place_action)
        # Now check HP
        tensor = state.information_state_tensor(player)
        hp_val = tensor[hp_idx]
        assert hp_val > 0.0, "HP should be > 0 after placing pokemon"
    else:
        print("Could not find Place action to test HP update")
