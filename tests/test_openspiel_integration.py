import sys
import os
import pyspiel
import random
import numpy as np
import pytest

# Add python directory to path so we can import deckgym_openspiel
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../python")))

try:
    from deckgym_openspiel.game import DeckGymGame
except ImportError:
    pass

def test_integration():
    # Use existing deck file relative to repo root
    deck_path = os.path.join(os.path.dirname(__file__), "../example_decks/mewtwoex.txt")
    if not os.path.exists(deck_path):
             pytest.skip(f"Deck file not found at {os.path.abspath(deck_path)}")

    game = pyspiel.load_game("deckgym_ptcgp", {
        "deck_id_1": deck_path,
        "deck_id_2": deck_path,
        "seed": 42
    })

    state = game.new_initial_state()
    assert state.current_player() in [0, 1, pyspiel.PlayerId.CHANCE]

    # Test cloning
    clone_state = state.clone()
    assert str(state) == str(clone_state)

    steps = 0
    while not state.is_terminal():
        if state.is_chance_node():
            outcomes = state.chance_outcomes()
            # Pick first outcome for determinism/simplicity
            action, _ = outcomes[0]
            state.apply_action(action)
        else:
            legal_actions = state.legal_actions()
            assert len(legal_actions) > 0
            # Pick random legal action
            action = random.choice(legal_actions)
            state.apply_action(action)
        steps += 1

        if steps > 200: # Limit steps to avoid infinite loop if logic is broken
            break

    # Verify game progresses
    assert steps > 0
