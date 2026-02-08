import pyspiel
import deckgym
from deckgym_openspiel.state import DeckGymState

class DeckGymGame(pyspiel.Game):
    def __init__(self, deck_a: str | list[str], deck_b: str | list[str], seed=None):
        self._deck_a = deck_a
        self._deck_b = deck_b
        self._seed = seed

        # Cache tensor shape
        try:
            dummy_state = deckgym.PyGameState(self._deck_a, self._deck_b, self._seed)
            self._obs_shape = [len(dummy_state.encode_observation())]
        except Exception:
             # Fallback or error if decks are invalid
             self._obs_shape = [0]

        game_type = pyspiel.GameType(
            short_name="deckgym",
            long_name="DeckGym Pokémon TCG Pocket",
            dynamics=pyspiel.GameType.Dynamics.SEQUENTIAL,
            chance_mode=pyspiel.GameType.ChanceMode.EXPLICIT_STOCHASTIC,
            information=pyspiel.GameType.Information.IMPERFECT_INFORMATION,
            utility=pyspiel.GameType.Utility.ZERO_SUM,
            reward_model=pyspiel.GameType.RewardModel.TERMINAL,
            max_num_players=2,
            min_num_players=2,
            provides_information_state_string=True,
            provides_information_state_tensor=True,
            provides_observation_string=True,
            provides_observation_tensor=True,
            parameter_specification={}
        )

        game_info = pyspiel.GameInfo(
            deckgym.PyGameState.get_action_space_size(),
            100,
            2,
            -1.0,
            1.0,
            0.0,
            1000
        )
        super().__init__(game_type, game_info, {})

    def new_initial_state(self):
        game_state = deckgym.PyGameState(self._deck_a, self._deck_b, self._seed)
        return DeckGymState(self, game_state)

    def num_distinct_actions(self):
        return deckgym.PyGameState.get_action_space_size()

    def max_chance_outcomes(self):
        return 100

    def num_players(self):
        return 2

    def min_utility(self):
        return -1.0

    def max_utility(self):
        return 1.0

    def utility_sum(self):
        return 0.0

    def get_type(self):
        return pyspiel.GameType(
            short_name="deckgym",
            long_name="DeckGym Pokémon TCG Pocket",
            dynamics=pyspiel.GameType.Dynamics.SEQUENTIAL,
            chance_mode=pyspiel.GameType.ChanceMode.EXPLICIT_STOCHASTIC,
            information=pyspiel.GameType.Information.IMPERFECT_INFORMATION,
            utility=pyspiel.GameType.Utility.ZERO_SUM,
            reward_model=pyspiel.GameType.RewardModel.TERMINAL,
            max_num_players=2,
            min_num_players=2,
            provides_information_state_string=True,
            provides_information_state_tensor=True,
            provides_observation_string=True,
            provides_observation_tensor=True,
            parameter_specification={}
        )

    def information_state_tensor_shape(self):
        return self._obs_shape

    def observation_tensor_shape(self):
        return self._obs_shape
