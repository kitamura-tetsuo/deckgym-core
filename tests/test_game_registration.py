import pyspiel
import sys
import os
import pytest

# Ensure python/ is in sys.path so we can import deckgym_openspiel
sys.path.append(os.path.join(os.path.dirname(__file__), "../python"))

import deckgym_openspiel

def test_game_registration():
    """Tests that the game is registered correctly with OpenSpiel."""
    game_name = "deckgym_ptcgp"

    # Verify game is in registered names
    registered_names = pyspiel.registered_names()
    assert game_name in registered_names, f"{game_name} not found in registered games."

    # Load game without parameters
    game = pyspiel.load_game(game_name)
    assert game.get_type().short_name == game_name
    assert game.num_players() == 2

    # Verify game type properties
    game_type = game.get_type()
    assert game_type.dynamics == pyspiel.GameType.Dynamics.SEQUENTIAL
    assert game_type.chance_mode == pyspiel.GameType.ChanceMode.EXPLICIT_STOCHASTIC
    assert game_type.information == pyspiel.GameType.Information.IMPERFECT_INFORMATION
    assert game_type.utility == pyspiel.GameType.Utility.ZERO_SUM
    assert game_type.reward_model == pyspiel.GameType.RewardModel.TERMINAL

    # Load game with parameters
    params = {"deck_id_1": "test_deck_1", "deck_id_2": "test_deck_2"}
    game_with_params = pyspiel.load_game(game_name, params)

    # Check parameters
    game_params = game_with_params.get_parameters()
    assert game_params["deck_id_1"] == "test_deck_1"
    assert game_params["deck_id_2"] == "test_deck_2"

    # Verify new_initial_state raises NotImplementedError
    with pytest.raises(NotImplementedError):
        game.new_initial_state()

if __name__ == "__main__":
    sys.exit(pytest.main(["-v", __file__]))
