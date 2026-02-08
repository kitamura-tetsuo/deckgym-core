import gymnasium as gym
from gymnasium.utils.env_checker import check_env
import deckgym
import os
import pytest

def test_ptcgp_env_compliance():
    deck_path = "example_decks/fire.txt"
    if not os.path.exists(deck_path):
        pytest.skip(f"Deck file {deck_path} not found.")

    try:
        env = gym.make("deckgym:PTCGP-v0", deck_a=deck_path, deck_b=deck_path, seed=42)
    except gym.error.Error as e:
        env = gym.make("PTCGP-v0", deck_a=deck_path, deck_b=deck_path, seed=42)

    print("Checking environment compliance...")
    check_env(env)
    print("Environment is compliant with Gymnasium API.")

    # Run a few steps manually
    obs, info = env.reset(seed=42)
    assert obs.shape == (33352,), "Initial observation shape mismatch"

    valid_actions = info.get("valid_actions", [])

    for i in range(5):
        if valid_actions:
            action = valid_actions[0]
        else:
            action = env.action_space.sample()

        obs, reward, terminated, truncated, info = env.step(action)

        if terminated or truncated:
            obs, info = env.reset()
            valid_actions = info.get("valid_actions", [])
        else:
            valid_actions = info.get("valid_actions", [])

if __name__ == "__main__":
    test_ptcgp_env_compliance()
