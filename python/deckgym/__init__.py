from .deckgym import (
    PyEnergyType as EnergyType,
    PyAttack as Attack,
    PyAbility as Ability,
    PyCard as Card,
    PyPlayedCard as PlayedCard,
    PyDeck as Deck,
    PyGame as Game,
    PyGameState,
    PyState as State,
    PyGameOutcome as GameOutcome,
    PySimulationResults as SimulationResults,
    py_simulate as simulate,
    get_player_types,
    PyBatchedSimulator,
)
from gymnasium.envs.registration import register
from .envs.ptcgp_env import PTCGPEnv

__all__ = [
    "EnergyType",
    "Attack",
    "Ability",
    "Card",
    "PlayedCard",
    "Deck",
    "Game",
    "PyGameState",
    "State",
    "GameOutcome",
    "SimulationResults",
    "simulate",
    "get_player_types",
    "PyBatchedSimulator",
    "PTCGPEnv",
]

register(
    id="PTCGP-v0",
    entry_point="deckgym.envs.ptcgp_env:PTCGPEnv",
)
