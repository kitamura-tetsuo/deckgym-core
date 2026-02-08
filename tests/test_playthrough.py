import pytest
import pyspiel
import random
import sys
import os

# Add python/ directory to sys.path to allow importing deckgym_openspiel
sys.path.append(os.path.join(os.getcwd(), "python"))

import deckgym
from deckgym_openspiel.game import DeckGymGame
from deckgym_openspiel.state import DeckGymState

def test_random_playthrough():
    # Setup simple decks using valid card IDs
    card_ids = [
        "A1 001", "A1 001", # Bulbasaur
        "A1 002", "A1 002", # Ivysaur
        "A1 003", "A1 003", # Venusaur
        "A1 004", "A1 004", # Charmander
        "A1 005", "A1 005", # Charmeleon
        "A1 006", "A1 006", # Charizard
        "A1 007", "A1 007", # Squirtle
        "A1 008", "A1 008", # Wartortle
        "A1 009", "A1 009", # Blastoise
        "A1 010", "A1 010", # Caterpie
    ]

    try:
        game = DeckGymGame(deck_a=card_ids, deck_b=card_ids, seed=42)
    except Exception as e:
        pytest.fail(f"Failed to initialize DeckGymGame: {e}")

    state = game.new_initial_state()

    step_count = 0
    max_steps = 1000

    while not state.is_terminal():
        legal_actions = state.legal_actions()

        if not legal_actions:
            # Should not happen unless state thinks it's not terminal but has no actions
            break

        action = random.choice(legal_actions)
        state.apply_action(action)
        step_count += 1

        if step_count > max_steps:
            break

    assert state.is_terminal(), f"Game did not terminate within {max_steps} steps."

    returns = state.returns()
    assert len(returns) == 2

    if returns[0] == 0.0 and returns[1] == 0.0:
        pass # Tie
    else:
        assert returns[0] in [1.0, -1.0]
        assert returns[1] in [1.0, -1.0]
        assert returns[0] + returns[1] == 0.0
