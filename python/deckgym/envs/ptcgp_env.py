import gymnasium as gym
import numpy as np
from gymnasium import spaces
import deckgym

class PTCGPEnv(gym.Env):
    """
    Gymnasium environment for Pokemon TCG Pocket.
    """
    metadata = {"render_modes": ["human", "ansi"], "render_fps": 4}

    def __init__(self, deck_a, deck_b, seed=None, render_mode=None):
        self.deck_a = deck_a
        self.deck_b = deck_b
        self.render_mode = render_mode
        self._seed = seed

        # Initialize the game to get space sizes
        self.game = deckgym.PyGameState(deck_a, deck_b, seed)

        # Action space
        action_size = deckgym.PyGameState.get_action_space_size()
        self.action_space = spaces.Discrete(action_size)

        # Observation space
        initial_obs = self.game.encode_observation()
        obs_size = len(initial_obs)
        self.observation_space = spaces.Box(
            low=0,
            high=float('inf'),
            shape=(obs_size,),
            dtype=np.float32
        )

    def reset(self, seed=None, options=None):
        super().reset(seed=seed)

        if seed is not None:
            self._seed = seed

        # Reset the game state
        self.game.reset()

        observation = self._get_obs()
        info = self._get_info()

        if self.render_mode == "human":
            self.render()

        return observation, info

    def step(self, action):
        try:
            # Apply action using global ID
            done, won = self.game.step_with_id(int(action))

            observation = self._get_obs()
            reward = 1.0 if won else 0.0
            terminated = done
            truncated = False
            info = self._get_info()

        except ValueError:
            # Illegal action
            observation = self._get_obs()
            reward = -1.0
            terminated = True # Terminate on illegal action to prevent infinite loops
            truncated = False
            info = self._get_info()
            info["error"] = "Illegal action"

        if self.render_mode == "human":
            self.render()

        return observation, reward, terminated, truncated, info

    def render(self):
        state = self.game.get_state()
        if self.render_mode == "ansi":
            return state.debug_string()
        elif self.render_mode == "human":
            print(state.debug_string())

    def _get_obs(self):
        return np.array(self.game.encode_observation(), dtype=np.float32)

    def _get_info(self):
        state = self.game.get_state()
        return {
            "turn": state.turn_count,
            "current_player": state.current_player,
            "points": state.points,
            "valid_actions": self.game.legal_actions()
        }

    def close(self):
        pass
