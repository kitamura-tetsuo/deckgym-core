import pyspiel
from deckgym import PyGameState, GameOutcome

class DeckGymState(pyspiel.State):
    def __init__(self, game, deckgym_game_state: PyGameState, chance_probabilities=None, pending_action_id=None):
        super().__init__(game)
        self._game_state = deckgym_game_state
        self._chance_probabilities = chance_probabilities
        self._pending_action_id = pending_action_id

        # Check initial terminal status
        state = self._game_state.get_state()
        self._game_over = state.is_game_over()
        self._winner = state.winner.winner if state.winner else None
        self._is_tie = state.winner.is_tie if state.winner else False

    def current_player(self):
        if self._game_over:
            return pyspiel.PlayerId.TERMINAL
        if self._chance_probabilities is not None:
            return pyspiel.PlayerId.CHANCE
        return self._game_state.get_state().current_player

    def legal_actions(self, player=None):
        if self._game_over:
            return []

        # Handle chance player
        if self._chance_probabilities is not None:
            if player == pyspiel.PlayerId.CHANCE or player is None:
                return list(range(len(self._chance_probabilities)))
            return []

        # Handle regular players
        current_player = self.current_player()
        if player is not None and player != current_player:
            return []

        return self._game_state.legal_actions()

    def chance_outcomes(self):
        """Returns the possible chance outcomes and their probabilities."""
        if self._chance_probabilities is None:
            return []
        return list(enumerate(self._chance_probabilities))

    def apply_action(self, action):
        if self._chance_probabilities is not None:
            # Resolving chance node
            outcome_idx = action
            if not (0 <= outcome_idx < len(self._chance_probabilities)):
                raise ValueError(f"Invalid chance outcome index: {outcome_idx}")

            # Apply the specific outcome
            done, won = self._game_state.apply_action_outcome(self._pending_action_id, outcome_idx)

            # Clear chance state
            self._chance_probabilities = None
            self._pending_action_id = None

            # Update terminal status
            self._game_over = done
            state = self._game_state.get_state()
            self._winner = state.winner.winner if state.winner else None
            self._is_tie = state.winner.is_tie if state.winner else False

        else:
            # Regular player action
            # Predict outcomes first
            probs = self._game_state.get_action_probabilities(action)

            if len(probs) > 1:
                # Transition to chance node
                self._chance_probabilities = probs
                self._pending_action_id = action
            else:
                # Deterministic action (or single outcome stochastic action that resolves immediately)
                # Apply outcome 0
                done, won = self._game_state.apply_action_outcome(action, 0)

                # Update terminal status
                self._game_over = done
                state = self._game_state.get_state()
                self._winner = state.winner.winner if state.winner else None
                self._is_tie = state.winner.is_tie if state.winner else False

    def action_to_string(self, player, action):
        if player == pyspiel.PlayerId.CHANCE:
            return f"Outcome {action}"
        return PyGameState.action_name(action)

    def is_terminal(self):
        return self._game_over

    def returns(self):
        if not self._game_over:
            return [0.0, 0.0]

        if self._is_tie:
            return [0.0, 0.0]

        if self._winner == 0:
            return [1.0, -1.0]
        elif self._winner == 1:
            return [-1.0, 1.0]
        else:
            return [0.0, 0.0]

    def __str__(self):
        return self._game_state.get_state().debug_string()

    def clone(self):
         return DeckGymState(
             self.get_game(),
             self._game_state.clone(),
             self._chance_probabilities[:] if self._chance_probabilities else None,
             self._pending_action_id
         )
