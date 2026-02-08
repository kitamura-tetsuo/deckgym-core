import pyspiel
import deckgym
import deckgym_openspiel
import pytest

def test_game_registration():
    # Load the game
    game = pyspiel.load_game("deckgym_ptcgp")

    # Check short name
    assert game.get_type().short_name == "deckgym_ptcgp"

    # Check max players
    assert game.num_players() == 2
    assert game.get_type().max_num_players == 2
    assert game.get_type().min_num_players == 2

    # Check that we can create a game with specific parameters
    params = {"deck_id_1": "deck1", "deck_id_2": "deck2"}
    game_with_params = pyspiel.load_game("deckgym_ptcgp", params)
    assert game_with_params.get_type().short_name == "deckgym_ptcgp"
