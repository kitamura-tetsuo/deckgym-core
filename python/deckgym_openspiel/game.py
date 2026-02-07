import pyspiel

_GAME_TYPE = pyspiel.GameType(
    short_name="deckgym_ptcgp",
    long_name="DeckGym Pokemon TCG Pocket",
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
    parameter_specification={
        "deck_id_1": "deck_1",
        "deck_id_2": "deck_2"
    }
)

_GAME_INFO = pyspiel.GameInfo(
    num_distinct_actions=100, # Placeholder
    max_chance_outcomes=10, # Placeholder
    num_players=2,
    min_utility=-1.0,
    max_utility=1.0,
    utility_sum=0.0,
    max_game_length=1000 # Placeholder
)

class DeckGymGame(pyspiel.Game):
    def __init__(self, params=None):
        super().__init__(_GAME_TYPE, _GAME_INFO, params or {})

    def new_initial_state(self):
        raise NotImplementedError

pyspiel.register_game(_GAME_TYPE, DeckGymGame)
