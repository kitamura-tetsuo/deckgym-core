import deckgym
import pytest

def test_env_observation_and_actions():
    print("Testing PyGameState...")
    # Create dummy decks
    # Using 20 Bulbasaurs (A1 001)
    card_ids = ["A1 001"] * 20

    # Initialize GameState
    game = deckgym.PyGameState(card_ids, card_ids, seed=42)

    # Reset
    game.reset()

    # Encode Observation
    obs = game.encode_observation()
    print(f"Observation shape: {len(obs)}")
    # Check shape roughly matches expectations (size ~10000)
    assert len(obs) > 1000

    # Legal Actions
    actions = game.legal_actions()
    print(f"Legal actions: {actions}")
    # At start of game, we should have some actions (e.g. place active)
    assert len(actions) > 0

    # Action Names
    for action_id in actions:
        name = game.action_name(action_id)
        print(f"Action {action_id}: {name}")
        assert isinstance(name, str)
        assert len(name) > 0

    # Check mapping consistency (basic sanity check)
    # EndTurn is 0
    assert game.action_name(0) == "EndTurn"

    # Action Space Size
    size = game.get_action_space_size()
    print(f"Action space size: {size}")
    assert size > 1000

if __name__ == "__main__":
    test_env_observation_and_actions()
