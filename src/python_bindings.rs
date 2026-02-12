use pyo3::exceptions::{PyIOError, PyIndexError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyList, PyModule};
use pyo3::wrap_pyfunction;
use rand::distributions::{Distribution, WeightedIndex};
use rand::{RngCore, SeedableRng};
use rayon::prelude::*;
use std::collections::HashMap;

use crate::{
    card_ids::CardId,
    deck::Deck, encoding, game::Game, generate_possible_actions,
    models::{Ability, Attack, Card, EnergyType, PlayedCard},
    players::{create_players, fill_code_array, parse_player_code, PlayerCode, RandomPlayer},
    state::{GameOutcome, State},
    actions::{
        Action, SimpleAction, EFFECT_MECHANIC_MAP, trainer_mechanic::TrainerMechanic,
        get_attack_mechanic, get_enhanced_ability_mechanic, get_simulator_ability_mechanic,
    },
    actions::attacks::Mechanic,
};

use numpy::PyArrayMethods;
use pyo3::Py;

/// Python wrapper for EnergyType
#[pyclass]
#[derive(Clone, Copy)]
pub struct PyEnergyType {
    energy_type: EnergyType,
}

#[pymethods]
impl PyEnergyType {
    fn __repr__(&self) -> String {
        format!("{:?}", self.energy_type)
    }

    fn __str__(&self) -> String {
        format!("{}", self.energy_type)
    }

    #[getter]
    fn name(&self) -> String {
        format!("{:?}", self.energy_type)
    }
}

impl From<EnergyType> for PyEnergyType {
    fn from(energy_type: EnergyType) -> Self {
        PyEnergyType { energy_type }
    }
}

/// Python wrapper for Attack
#[pyclass]
#[derive(Clone)]
pub struct PyAttack {
    attack: Attack,
}

#[pymethods]
impl PyAttack {
    #[getter]
    fn title(&self) -> String {
        self.attack.title.clone()
    }

    #[getter]
    fn fixed_damage(&self) -> u32 {
        self.attack.fixed_damage
    }

    #[getter]
    fn effect(&self) -> Option<String> {
        self.attack.effect.clone()
    }

    #[getter]
    fn energy_required(&self) -> Vec<PyEnergyType> {
        self.attack
            .energy_required
            .iter()
            .map(|&e| e.into())
            .collect()
    }

    #[getter]
    fn cost(&self) -> Vec<PyEnergyType> {
        self.energy_required()
    }

    #[getter]
    fn mechanic_info(&self, py: Python) -> Option<PyObject> {
        let effect_text = self.attack.effect.as_deref()?;
        let mechanic = EFFECT_MECHANIC_MAP.get(effect_text)?;
        let json_str = serde_json::to_string(mechanic).ok()?;
        let json_module = py.import_bound("json").ok()?;
        json_module
            .call_method1("loads", (json_str,))
            .ok()
            .map(|v| v.to_object(py))
    }

    fn __repr__(&self) -> String {
        format!(
            "Attack(title='{}', damage={}, effect={:?})",
            self.attack.title, self.attack.fixed_damage, self.attack.effect
        )
    }
}

impl From<Attack> for PyAttack {
    fn from(attack: Attack) -> Self {
        PyAttack { attack }
    }
}

/// Python wrapper for Ability
#[pyclass]
#[derive(Clone)]
pub struct PyAbility {
    #[pyo3(get)]
    pub title: String,
    #[pyo3(get)]
    pub effect: String,
}

#[pymethods]
impl PyAbility {
    fn __repr__(&self) -> String {
        format!("Ability(title='{}', effect='{}')", self.title, self.effect)
    }
}

impl From<Ability> for PyAbility {
    fn from(ability: Ability) -> Self {
        PyAbility {
            title: ability.title,
            effect: ability.effect,
        }
    }
}

/// Python wrapper for Card
#[pyclass]
#[derive(Clone)]
pub struct PyCard {
    card: Card,
}

#[pymethods]
impl PyCard {
    #[getter]
    fn id(&self) -> String {
        self.card.get_id()
    }

    #[getter]
    fn name(&self) -> String {
        self.card.get_name()
    }

    #[getter]
    fn is_pokemon(&self) -> bool {
        matches!(self.card, Card::Pokemon(_))
    }

    #[getter]
    fn is_trainer(&self) -> bool {
        matches!(self.card, Card::Trainer(_))
    }

    #[getter]
    fn is_basic(&self) -> bool {
        self.card.is_basic()
    }

    #[getter]
    fn is_ex(&self) -> bool {
        self.card.is_ex()
    }

    #[getter]
    fn hp(&self) -> u32 {
        match &self.card {
            Card::Pokemon(pokemon_card) => pokemon_card.hp,
            _ => 0,
        }
    }

    #[getter]
    fn stage(&self) -> u8 {
        match &self.card {
            Card::Pokemon(pokemon_card) => pokemon_card.stage,
            _ => 0,
        }
    }

    #[getter]
    fn energy_type(&self) -> Option<PyEnergyType> {
        self.card.get_type().map(|t| t.into())
    }

    #[getter]
    fn attacks(&self) -> Vec<PyAttack> {
        match &self.card {
            Card::Pokemon(_) => self
                .card
                .get_attacks()
                .iter()
                .map(|a| a.clone().into())
                .collect(),
            _ => Vec::new(),
        }
    }

    #[getter]
    fn ability(&self) -> Option<PyAbility> {
        self.card.get_ability().map(|a| a.into())
    }

    #[getter]
    fn weakness(&self) -> Option<PyEnergyType> {
        match &self.card {
            Card::Pokemon(pokemon_card) => pokemon_card.weakness.map(|w| w.into()),
            _ => None,
        }
    }

    #[getter]
    fn trainer_mechanic_info(&self, py: Python) -> Option<PyObject> {
        let card_id = self.card.get_card_id();
        let mechanic = card_id.get_trainer_mechanic()?;
        let json_str = serde_json::to_string(&mechanic).ok()?;
        let json_module = py.import_bound("json").ok()?;
        json_module
            .call_method1("loads", (json_str,))
            .ok()
            .map(|v| v.to_object(py))
    }

    #[getter]
    fn attack_mechanic_info(&self, py: Python) -> Vec<Option<PyObject>> {
        let mut infos = Vec::new();
        let attacks = self.card.get_attacks();
        for (i, atk) in attacks.iter().enumerate() {
            let mechanic = get_attack_mechanic(&self.card, i).or_else(|| {
                atk.effect.as_deref().and_then(|text| EFFECT_MECHANIC_MAP.get(text)).cloned()
            });

            if let Some(mechanic) = mechanic {
                let json_str = serde_json::to_string(&mechanic).unwrap_or_default();
                let json_module = py.import_bound("json").unwrap();
                let info = json_module.call_method1("loads", (json_str,)).unwrap().to_object(py);
                infos.push(Some(info));
            } else {
                infos.push(None);
            }
        }
        infos
    }

    #[getter]
    fn ability_mechanic_info(&self, py: Python) -> Option<PyObject> {
        let mechanic = get_enhanced_ability_mechanic(&self.card).or_else(|| {
            get_simulator_ability_mechanic(&self.card).cloned()
        })?;
        let json_str = serde_json::to_string(&mechanic).ok()?;
        let json_module = py.import_bound("json").ok()?;
        json_module
            .call_method1("loads", (json_str,))
            .ok()
            .map(|v| v.to_object(py))
    }

    #[getter]
    fn retreat_cost(&self) -> usize {
        match &self.card {
            Card::Pokemon(pokemon_card) => pokemon_card.retreat_cost.len(),
            _ => 0,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Card(id='{}', name='{}')",
            self.card.get_id(),
            self.card.get_name()
        )
    }
}

impl From<Card> for PyCard {
    fn from(card: Card) -> Self {
        PyCard { card }
    }
}

/// Python wrapper for PlayedCard
#[pyclass]
#[derive(Clone)]
pub struct PyPlayedCard {
    played_card: PlayedCard,
}

#[pymethods]
impl PyPlayedCard {
    #[getter]
    fn card(&self) -> PyCard {
        self.played_card.card.clone().into()
    }

    #[getter]
    fn remaining_hp(&self) -> u32 {
        self.played_card.remaining_hp
    }

    #[getter]
    fn total_hp(&self) -> u32 {
        self.played_card.total_hp
    }

    #[getter]
    fn attached_energy(&self) -> Vec<PyEnergyType> {
        self.played_card
            .attached_energy
            .iter()
            .map(|&e| e.into())
            .collect()
    }

    #[getter]
    fn played_this_turn(&self) -> bool {
        self.played_card.played_this_turn
    }

    #[getter]
    fn ability_used(&self) -> bool {
        self.played_card.ability_used
    }

    #[getter]
    fn poisoned(&self) -> bool {
        self.played_card.poisoned
    }

    #[getter]
    fn paralyzed(&self) -> bool {
        self.played_card.paralyzed
    }

    #[getter]
    fn asleep(&self) -> bool {
        self.played_card.asleep
    }

    #[getter]
    fn is_damaged(&self) -> bool {
        self.played_card.is_damaged()
    }

    #[getter]
    fn has_tool_attached(&self) -> bool {
        self.played_card.has_tool_attached()
    }

    #[getter]
    fn name(&self) -> String {
        self.played_card.get_name()
    }

    #[getter]
    fn energy_type(&self) -> Option<PyEnergyType> {
        self.played_card.get_energy_type().map(|t| t.into())
    }

    #[getter]
    fn attacks(&self) -> Vec<PyAttack> {
        self.played_card
            .get_attacks()
            .iter()
            .map(|a| a.clone().into())
            .collect()
    }

    #[getter]
    fn ability(&self) -> Option<PyAbility> {
        self.played_card.card.get_ability().map(|a| a.into())
    }

    fn __repr__(&self) -> String {
        format!(
            "PlayedCard(name='{}', hp={}/{}, energy={})",
            self.played_card.get_name(),
            self.played_card.remaining_hp,
            self.played_card.total_hp,
            self.played_card.attached_energy.len()
        )
    }
}

impl From<PlayedCard> for PyPlayedCard {
    fn from(played_card: PlayedCard) -> Self {
        PyPlayedCard { played_card }
    }
}

/// Python wrapper for GameOutcome
#[pyclass]
#[derive(Clone)]
pub struct PyGameOutcome {
    #[pyo3(get)]
    pub winner: Option<usize>,
    #[pyo3(get)]
    pub is_tie: bool,
}

impl From<GameOutcome> for PyGameOutcome {
    fn from(outcome: GameOutcome) -> Self {
        match outcome {
            GameOutcome::Win(player) => PyGameOutcome {
                winner: Some(player),
                is_tie: false,
            },
            GameOutcome::Tie => PyGameOutcome {
                winner: None,
                is_tie: true,
            },
        }
    }
}

#[pymethods]
impl PyGameOutcome {
    fn __repr__(&self) -> String {
        if self.is_tie {
            "GameOutcome::Tie".to_string()
        } else if let Some(winner) = self.winner {
            format!("GameOutcome::Win({})", winner)
        } else {
            panic!("Invalid state: PyGameOutcome has neither a winner nor a tie.");
        }
    }
}

/// Python wrapper for Deck
#[pyclass]
pub struct PyDeck {
    deck: Deck,
}

#[pymethods]
impl PyDeck {
    #[new]
    pub fn new(deck_path: &str) -> PyResult<Self> {
        let deck = Deck::from_file(deck_path).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to load deck: {}", e))
        })?;
        Ok(PyDeck { deck })
    }

    fn __repr__(&self) -> String {
        format!("PyDeck(cards={})", self.deck.cards.len())
    }

    #[getter]
    fn card_count(&self) -> usize {
        self.deck.cards.len()
    }
}

/// Python wrapper for State
#[pyclass]
pub struct PyState {
    state: State,
}

#[pymethods]
impl PyState {
    #[getter]
    fn turn_count(&self) -> u8 {
        self.state.turn_count
    }

    #[getter]
    fn current_player(&self) -> usize {
        self.state.current_player
    }

    #[getter]
    fn points(&self) -> [u8; 2] {
        self.state.points
    }

    #[getter]
    fn winner(&self) -> Option<PyGameOutcome> {
        self.state.winner.map(|outcome| outcome.into())
    }

    #[getter]
    fn current_energy(&self) -> Option<PyEnergyType> {
        self.state.current_energy.map(|e| e.into())
    }

    #[getter]
    fn has_played_support(&self) -> bool {
        self.state.has_played_support
    }

    #[getter]
    fn has_retreated(&self) -> bool {
        self.state.has_retreated
    }

    fn is_game_over(&self) -> bool {
        self.state.is_game_over()
    }

    /// Get the hand of a specific player (0 or 1)
    fn get_hand(&self, player: usize) -> PyResult<Vec<PyCard>> {
        if player > 1 {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "Player index must be 0 or 1",
            ));
        }
        Ok(self.state.hands[player]
            .iter()
            .map(|card| card.clone().into())
            .collect())
    }

    /// Get the number of cards remaining in a player's deck
    fn get_deck_size(&self, player: usize) -> PyResult<usize> {
        if player > 1 {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "Player index must be 0 or 1",
            ));
        }
        Ok(self.state.decks[player].cards.len())
    }

    /// Get the discard pile of a specific player
    fn get_discard_pile(&self, player: usize) -> PyResult<Vec<PyCard>> {
        if player > 1 {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "Player index must be 0 or 1",
            ));
        }
        Ok(self.state.discard_piles[player]
            .iter()
            .map(|card| card.clone().into())
            .collect())
    }

    /// Get all in-play pokemon for a specific player
    fn get_in_play_pokemon(&self, player: usize) -> PyResult<Vec<Option<PyPlayedCard>>> {
        if player > 1 {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "Player index must be 0 or 1",
            ));
        }
        Ok(self.state.in_play_pokemon[player]
            .iter()
            .map(|opt_card| opt_card.as_ref().map(|card| card.clone().into()))
            .collect())
    }

    /// Get the active pokemon for a specific player (index 0)
    fn get_active_pokemon(&self, player: usize) -> PyResult<Option<PyPlayedCard>> {
        if player > 1 {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "Player index must be 0 or 1",
            ));
        }
        Ok(self.state.in_play_pokemon[player][0]
            .as_ref()
            .map(|card| card.clone().into()))
    }

    /// Get the bench pokemon for a specific player (indices 1-3)
    fn get_bench_pokemon(&self, player: usize) -> PyResult<Vec<Option<PyPlayedCard>>> {
        if player > 1 {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "Player index must be 0 or 1",
            ));
        }
        Ok(self.state.in_play_pokemon[player][1..4]
            .iter()
            .map(|opt_card| opt_card.as_ref().map(|card| card.clone().into()))
            .collect())
    }

    /// Get a specific pokemon by player and position
    fn get_pokemon_at_position(
        &self,
        player: usize,
        position: usize,
    ) -> PyResult<Option<PyPlayedCard>> {
        if player > 1 {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "Player index must be 0 or 1",
            ));
        }
        if position > 3 {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "Position index must be 0-3 (0=active, 1-3=bench)",
            ));
        }
        Ok(self.state.in_play_pokemon[player][position]
            .as_ref()
            .map(|card| card.clone().into()))
    }

    /// Get the remaining HP of a pokemon at a specific position
    fn get_remaining_hp(&self, player: usize, position: usize) -> PyResult<u32> {
        if player > 1 {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "Player index must be 0 or 1",
            ));
        }
        if position > 3 {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "Position index must be 0-3 (0=active, 1-3=bench)",
            ));
        }
        if let Some(pokemon) = &self.state.in_play_pokemon[player][position] {
            Ok(pokemon.remaining_hp)
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "No pokemon at this position",
            ))
        }
    }

    /// Count the number of in-play pokemon of a specific energy type for a player
    fn count_in_play_of_type(&self, player: usize, energy_type: PyEnergyType) -> PyResult<usize> {
        if player > 1 {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "Player index must be 0 or 1",
            ));
        }
        Ok(self
            .state
            .num_in_play_of_type(player, energy_type.energy_type))
    }

    /// Get a list of (position, pokemon) tuples for all in-play pokemon of a player
    fn enumerate_in_play_pokemon(&self, player: usize) -> PyResult<Vec<(usize, PyPlayedCard)>> {
        if player > 1 {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "Player index must be 0 or 1",
            ));
        }
        Ok(self
            .state
            .enumerate_in_play_pokemon(player)
            .map(|(pos, card)| (pos, card.clone().into()))
            .collect())
    }

    /// Get a list of (position, pokemon) tuples for all bench pokemon of a player
    fn enumerate_bench_pokemon(&self, player: usize) -> PyResult<Vec<(usize, PyPlayedCard)>> {
        if player > 1 {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "Player index must be 0 or 1",
            ));
        }
        Ok(self
            .state
            .enumerate_bench_pokemon(player)
            .map(|(pos, card)| (pos, card.clone().into()))
            .collect())
    }

    /// Get the debug string representation of the state
    fn debug_string(&self) -> String {
        self.state.debug_string()
    }

    /// Get hand size for a player
    fn get_hand_size(&self, player: usize) -> PyResult<usize> {
        if player > 1 {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "Player index must be 0 or 1",
            ));
        }
        Ok(self.state.hands[player].len())
    }

    /// Get discard pile size for a player
    fn get_discard_pile_size(&self, player: usize) -> PyResult<usize> {
        if player > 1 {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "Player index must be 0 or 1",
            ));
        }
        Ok(self.state.discard_piles[player].len())
    }

    /// Count the number of pokemon in play for a player
    fn count_in_play_pokemon(&self, player: usize) -> PyResult<usize> {
        if player > 1 {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "Player index must be 0 or 1",
            ));
        }
        Ok(self.state.in_play_pokemon[player]
            .iter()
            .filter(|pokemon| pokemon.is_some())
            .count())
    }

    /// Check if a player has an active pokemon
    fn has_active_pokemon(&self, player: usize) -> PyResult<bool> {
        if player > 1 {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "Player index must be 0 or 1",
            ));
        }
        Ok(self.state.in_play_pokemon[player][0].is_some())
    }

    /// Count the number of bench pokemon for a player
    fn count_bench_pokemon(&self, player: usize) -> PyResult<usize> {
        if player > 1 {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "Player index must be 0 or 1",
            ));
        }
        Ok(self.state.in_play_pokemon[player][1..4]
            .iter()
            .filter(|pokemon| pokemon.is_some())
            .count())
    }

    fn __repr__(&self) -> String {
        format!(
            "PyState(turn={}, player={}, points={:?}, game_over={})",
            self.state.turn_count,
            self.state.current_player,
            self.state.points,
            self.state.is_game_over()
        )
    }
}

/// Python wrapper for Game
#[pyclass(unsendable)]
pub struct PyGame {
    game: Game<'static>,
}

#[pymethods]
impl PyGame {
    #[new]
    #[pyo3(signature = (deck_a_path, deck_b_path, players=None, seed=None))]
    pub fn new(
        deck_a_path: &str,
        deck_b_path: &str,
        players: Option<Vec<String>>,
        seed: Option<u64>,
    ) -> PyResult<Self> {
        let deck_a = Deck::from_file(deck_a_path).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to load deck A: {}", e))
        })?;
        let deck_b = Deck::from_file(deck_b_path).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to load deck B: {}", e))
        })?;

        let player_codes = if let Some(player_strs) = players {
            let mut codes = Vec::new();
            for player_str in player_strs {
                let code = parse_player_code(&player_str)
                    .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
                codes.push(code);
            }
            Some(codes)
        } else {
            None
        };

        let cli_players = fill_code_array(player_codes);
        let rust_players = create_players(deck_a, deck_b, cli_players);
        let game_seed = seed.unwrap_or_else(rand::random::<u64>);
        let game = Game::new(rust_players, game_seed);

        Ok(PyGame { game })
    }

    fn play(&mut self) -> Option<PyGameOutcome> {
        self.game.play().map(|outcome| outcome.into())
    }

    fn get_state(&self) -> PyState {
        PyState {
            state: self.game.get_state_clone(),
        }
    }

    fn play_tick(&mut self) -> String {
        let action = self.game.play_tick();
        format!("{:?}", action.action)
    }

    fn __repr__(&self) -> String {
        let state = self.game.get_state_clone();
        format!(
            "PyGame(turn={}, current_player={}, game_over={})",
            state.turn_count,
            state.current_player,
            state.is_game_over()
        )
    }
}

/// Python wrapper for GameState used in RL
#[pyclass(unsendable)]
pub struct PyGameState {
    game: Game<'static>,
    deck_a_source: PyObject,
    deck_b_source: PyObject,
    seed: Option<u64>,
}

#[pymethods]
impl PyGameState {
    #[new]
    #[pyo3(signature = (deck_a, deck_b, seed=None))]
    pub fn new(
        py: Python,
        deck_a: PyObject,
        deck_b: PyObject,
        seed: Option<u64>,
    ) -> PyResult<Self> {
        let deck_a_deck = parse_deck(py, &deck_a)?;
        let deck_b_deck = parse_deck(py, &deck_b)?;

        let game_seed = seed.unwrap_or_else(rand::random::<u64>);
        let game = create_game_from_decks(deck_a_deck, deck_b_deck, game_seed);

        Ok(PyGameState {
            game,
            deck_a_source: deck_a,
            deck_b_source: deck_b,
            seed,
        })
    }

    pub fn step(&mut self, action_idx: usize) -> PyResult<(bool, bool)> {
        let (_actor, actions) = generate_possible_actions(self.game.state());

        if action_idx >= actions.len() {
            return Err(PyIndexError::new_err(format!(
                "Action index {} out of bounds. Valid range: 0-{}",
                action_idx,
                actions.len().saturating_sub(1)
            )));
        }

        let action = &actions[action_idx];
        self.game.apply_action(action);

        let done = self.game.is_game_over();
        let won = if done {
            if let Some(GameOutcome::Win(winner)) = self.game.state().winner {
                // Return true if Player 0 won (usually the agent)
                winner == 0
            } else {
                false
            }
        } else {
            false
        };

        Ok((done, won))
    }

    pub fn step_with_id(&mut self, action_id: usize) -> PyResult<(bool, bool)> {
        let (_actor, actions) = generate_possible_actions(self.game.state());

        let mut found_idx = None;
        for (i, action) in actions.iter().enumerate() {
            if let Some(enc_id) = encoding::encode_action(&action.action) {
                if enc_id == action_id {
                    found_idx = Some(i);
                    break;
                }
            }
        }

        if let Some(idx) = found_idx {
            return self.step(idx);
        }

        let available_ids: Vec<usize> = actions
            .iter()
            .filter_map(|a| encoding::encode_action(&a.action))
            .collect();

        Err(PyValueError::new_err(format!(
            "Action ID {} ({}) is not currently legal. Available: {:?}",
            action_id,
            encoding::action_name(action_id),
            available_ids
        )))
    }

    pub fn get_action_probabilities(&self, action_id: usize) -> PyResult<Vec<f64>> {
        let (_actor, actions) = generate_possible_actions(self.game.state());

        // Find action
        let mut found_action = None;
        for action in &actions {
            if let Some(enc_id) = encoding::encode_action(&action.action) {
                if enc_id == action_id {
                    found_action = Some(action);
                    break;
                }
            }
        }

        let action = found_action.ok_or_else(|| {
            let available_ids: Vec<usize> = actions
                .iter()
                .filter_map(|a| encoding::encode_action(&a.action))
                .collect();
            PyValueError::new_err(format!(
                "Action ID {} ({}) is not currently legal. Available: {:?}",
                action_id,
                encoding::action_name(action_id),
                available_ids
            ))
        })?;

        let (probs, _) = crate::actions::forecast_action(self.game.state(), action);
        Ok(probs)
    }

    pub fn apply_action_outcome(
        &mut self,
        action_id: usize,
        outcome_idx: usize,
    ) -> PyResult<(bool, bool)> {
        let (_actor, actions) = generate_possible_actions(self.game.state());

        // Find action
        let mut found_action = None;
        for action in &actions {
            if let Some(enc_id) = encoding::encode_action(&action.action) {
                if enc_id == action_id {
                    found_action = Some(action.clone());
                    break;
                }
            }
        }

        let action = found_action.ok_or_else(|| {
            PyValueError::new_err(format!(
                "Action ID {} ({}) is not currently legal",
                action_id,
                encoding::action_name(action_id)
            ))
        })?;

        self.game.apply_action_with_outcome(&action, outcome_idx);

        let done = self.game.is_game_over();
        let won = if done {
            if let Some(GameOutcome::Win(winner)) = self.game.state().winner {
                // Return true if Player 0 won (usually the agent)
                winner == 0
            } else {
                false
            }
        } else {
            false
        };

        Ok((done, won))
    }

    pub fn reset(&mut self, py: Python) -> PyResult<()> {
        let deck_a_deck = parse_deck(py, &self.deck_a_source)?;
        let deck_b_deck = parse_deck(py, &self.deck_b_source)?;

        let game_seed = self.seed.unwrap_or_else(rand::random::<u64>);
        self.game = create_game_from_decks(deck_a_deck, deck_b_deck, game_seed);
        Ok(())
    }

    pub fn clone(&self, py: Python) -> PyResult<Self> {
        let state = self.game.get_state_clone();

        let deck_a_deck = parse_deck(py, &self.deck_a_source)?;
        let deck_b_deck = parse_deck(py, &self.deck_b_source)?;

        let player_a = Box::new(crate::players::RandomPlayer {
            deck: deck_a_deck.clone(),
        });
        let player_b = Box::new(crate::players::RandomPlayer {
            deck: deck_b_deck.clone(),
        });
        let players: Vec<Box<dyn crate::players::Player + Send>> = vec![player_a, player_b];

        let seed = self.seed.unwrap_or(0);
        let game = Game::from_state(state, players, seed);

        Ok(PyGameState {
            game,
            deck_a_source: self.deck_a_source.clone_ref(py),
            deck_b_source: self.deck_b_source.clone_ref(py),
            seed: self.seed,
        })
    }

    pub fn get_state(&self) -> PyState {
        PyState {
            state: self.game.get_state_clone(),
        }
    }

    #[pyo3(signature = (player_id=None, public_only=None))]
    pub fn encode_observation(
        &self,
        player_id: Option<usize>,
        public_only: Option<bool>,
    ) -> Vec<f32> {
        let public_only = public_only.unwrap_or(false);
        let player = player_id.unwrap_or_else(|| self.game.state().current_player);
        encoding::encode_state(self.game.state(), player, public_only)
    }

    pub fn legal_actions(&self) -> Vec<usize> {
        let (_actor, actions) = generate_possible_actions(self.game.state());
        actions
            .iter()
            .filter_map(|a| encoding::encode_action(&a.action))
            .collect()
    }

    #[staticmethod]
    pub fn action_name(action_id: usize) -> String {
        encoding::action_name(action_id)
    }

    #[staticmethod]
    pub fn get_action_space_size() -> usize {
        encoding::get_action_space_size()
    }
}

// Helper functions for PyGameState

fn parse_deck(py: Python, source: &PyObject) -> PyResult<Deck> {
    if let Ok(path) = source.extract::<String>(py) {
        // Try to load from file first
        if std::path::Path::new(&path).exists() {
            Deck::from_file(&path).map_err(|e| PyIOError::new_err(e))
        } else {
            // Treat as string content
            Deck::from_string(&path).map_err(|e| PyValueError::new_err(e))
        }
    } else if let Ok(card_ids) = source.extract::<Vec<String>>(py) {
        // List of card IDs
        let mut deck_content = String::new();
        for id in card_ids {
            deck_content.push_str(&format!("1 {}\n", id));
        }
        Deck::from_string(&deck_content).map_err(|e| PyValueError::new_err(e))
    } else {
        Err(PyValueError::new_err(
            "Deck source must be a file path, deck string, or list of card IDs",
        ))
    }
}

fn create_game_from_decks(deck_a: Deck, deck_b: Deck, seed: u64) -> Game<'static> {
    use crate::players::Player;
    let player_a = Box::new(RandomPlayer {
        deck: deck_a.clone(),
    });
    let player_b = Box::new(RandomPlayer {
        deck: deck_b.clone(),
    });
    let players: Vec<Box<dyn Player + Send>> = vec![player_a, player_b];
    Game::new(players, seed)
}

/// Simulation results
#[pyclass]
pub struct PySimulationResults {
    #[pyo3(get)]
    pub total_games: u32,
    #[pyo3(get)]
    pub player_a_wins: u32,
    #[pyo3(get)]
    pub player_b_wins: u32,
    #[pyo3(get)]
    pub ties: u32,
    #[pyo3(get)]
    pub player_a_win_rate: f32,
    #[pyo3(get)]
    pub player_b_win_rate: f32,
    #[pyo3(get)]
    pub tie_rate: f32,
}

#[pymethods]
impl PySimulationResults {
    fn __repr__(&self) -> String {
        format!(
            "SimulationResults(games={}, A_wins={} ({:.1}%), B_wins={} ({:.1}%), ties={} ({:.1}%))",
            self.total_games,
            self.player_a_wins,
            self.player_a_win_rate * 100.0,
            self.player_b_wins,
            self.player_b_win_rate * 100.0,
            self.ties,
            self.tie_rate * 100.0
        )
    }
}

/// Run multiple game simulations
#[pyfunction]
#[pyo3(signature = (deck_a_path, deck_b_path, players=None, num_simulations=100, seed=None))]
pub fn py_simulate(
    deck_a_path: &str,
    deck_b_path: &str,
    players: Option<Vec<String>>,
    num_simulations: u32,
    seed: Option<u64>,
) -> PyResult<PySimulationResults> {
    let deck_a = Deck::from_file(deck_a_path).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to load deck A: {}", e))
    })?;
    let deck_b = Deck::from_file(deck_b_path).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to load deck B: {}", e))
    })?;

    let player_codes = if let Some(player_strs) = players {
        let mut codes = Vec::new();
        for player_str in player_strs {
            let code = parse_player_code(&player_str)
                .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)?;
            codes.push(code);
        }
        Some(codes)
    } else {
        None
    };

    let cli_players = fill_code_array(player_codes);

    // Run simulations
    let mut wins_per_deck = [0u32, 0u32, 0u32]; // [player_a, player_b, ties]

    for _ in 0..num_simulations {
        let players = create_players(deck_a.clone(), deck_b.clone(), cli_players.clone());
        let game_seed = seed.unwrap_or_else(rand::random::<u64>);
        let mut game = Game::new(players, game_seed);
        let outcome = game.play();

        match outcome {
            Some(GameOutcome::Win(winner)) => {
                if winner < 2 {
                    wins_per_deck[winner] += 1;
                } else {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Invalid winner index: {}",
                        winner
                    )));
                }
            }
            Some(GameOutcome::Tie) | None => {
                wins_per_deck[2] += 1;
            }
        }
    }

    Ok(PySimulationResults {
        total_games: num_simulations,
        player_a_wins: wins_per_deck[0],
        player_b_wins: wins_per_deck[1],
        ties: wins_per_deck[2],
        player_a_win_rate: wins_per_deck[0] as f32 / num_simulations as f32,
        player_b_win_rate: wins_per_deck[1] as f32 / num_simulations as f32,
        tie_rate: wins_per_deck[2] as f32 / num_simulations as f32,
    })
}

/// Get available player types
#[pyfunction]
pub fn get_player_types() -> HashMap<String, String> {
    let mut types = HashMap::new();
    types.insert("r".to_string(), "Random Player".to_string());
    types.insert("aa".to_string(), "Attach-Attack Player".to_string());
    types.insert("et".to_string(), "End Turn Player".to_string());
    types.insert("h".to_string(), "Human Player".to_string());
    types.insert("w".to_string(), "Weighted Random Player".to_string());
    types.insert("m".to_string(), "MCTS Player".to_string());
    types.insert("v".to_string(), "Value Function Player".to_string());
    types.insert("e".to_string(), "Expectiminimax Player".to_string());
    types
}

/// Python module definition
#[pyfunction]
pub fn get_card(id: String) -> PyResult<PyCard> {
    let card_id = CardId::from_card_id(&id).ok_or_else(|| PyValueError::new_err("Invalid Card ID"))?;
    Ok(crate::database::get_card_by_enum(card_id).into())
}

#[pyfunction]
pub fn get_all_cards() -> Vec<PyCard> {
    use strum::IntoEnumIterator;
    CardId::iter()
        .map(|id| crate::database::get_card_by_enum(id).into())
        .collect()
}

pub fn deckgym(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_card, m)?)?;
    m.add_function(wrap_pyfunction!(get_all_cards, m)?)?;
    m.add_class::<PyEnergyType>()?;
    m.add_class::<PyAttack>()?;
    m.add_class::<PyAbility>()?;
    m.add_class::<PyCard>()?;
    m.add_class::<PyPlayedCard>()?;
    m.add_class::<PyDeck>()?;
    m.add_class::<PyGame>()?;
    m.add_class::<PyGameState>()?;
    m.add_class::<PyState>()?;
    m.add_class::<PyGameOutcome>()?;
    m.add_class::<PySimulationResults>()?;
    m.add_function(wrap_pyfunction!(py_simulate, m)?)?;
    m.add_function(wrap_pyfunction!(get_player_types, m)?)?;
    m.add_class::<PyBatchedSimulator>()?;
    Ok(())
}

/// Batched Simulator for RL
#[pyclass]
pub struct PyBatchedSimulator {
    games: Vec<Game<'static>>,
    batch_size: usize,
    win_reward: f32,
    point_reward: f32,
    damage_reward: f32,
    deck_cache: HashMap<String, Deck>,
}

#[pymethods]
impl PyBatchedSimulator {
    #[new]
    #[pyo3(signature = (deck_a_path, deck_b_path, batch_size, win_reward=1.0, point_reward=0.0, damage_reward=0.0))]
    pub fn new(
        deck_a_path: String,
        deck_b_path: String,
        batch_size: usize,
        win_reward: f32,
        point_reward: f32,
        damage_reward: f32,
    ) -> PyResult<Self> {
        let mut deck_cache = HashMap::new();
        let deck_a = Deck::from_file(&deck_a_path).map_err(|e| {
            PyIOError::new_err(format!("Failed to load deck A: {}", e))
        })?;
        let deck_b = Deck::from_file(&deck_b_path).map_err(|e| {
            PyIOError::new_err(format!("Failed to load deck B: {}", e))
        })?;
        
        deck_cache.insert(deck_a_path, deck_a);
        deck_cache.insert(deck_b_path, deck_b);

        Ok(PyBatchedSimulator {
            games: Vec::with_capacity(batch_size),
            batch_size,
            win_reward,
            point_reward,
            damage_reward,
            deck_cache,
        })
    }

    #[pyo3(signature = (seed=None, deck_ids_1=None, deck_ids_2=None))]
    pub fn reset<'py>(
        &mut self, 
        py: Python<'py>,
        seed: Option<u64>, 
        deck_ids_1: Option<Vec<String>>, 
        deck_ids_2: Option<Vec<String>>
    ) -> PyResult<(Py<numpy::PyArray2<f32>>, Py<numpy::PyArray1<usize>>, Py<numpy::PyArray2<f32>>)> {
        self.games.clear();
        let mut rng = if let Some(s) = seed {
            rand::rngs::StdRng::seed_from_u64(s)
        } else {
            rand::rngs::StdRng::from_entropy()
        };

        // Get default decks from cache if not specified
        let cached_paths: Vec<String> = self.deck_cache.keys().cloned().collect();
        if deck_ids_1.is_none() || deck_ids_2.is_none() {
            if cached_paths.len() < 2 {
                 return Err(PyValueError::new_err("Not enough decks in cache to reset without explicit deck_ids"));
            }
        }

        for i in 0..self.batch_size {
            let deck_1_path = if let Some(ref ids) = deck_ids_1 {
                &ids[i]
            } else {
                &cached_paths[0]
            };
            let deck_2_path = if let Some(ref ids) = deck_ids_2 {
                &ids[i]
            } else {
                &cached_paths[1]
            };

            let deck_1 = if let Some(d) = self.deck_cache.get(deck_1_path) {
                d.clone()
            } else {
                let d = Deck::from_file(deck_1_path).map_err(|e| {
                    PyIOError::new_err(format!("Failed to load deck: {}", e))
                })?;
                self.deck_cache.insert(deck_1_path.to_string(), d.clone());
                d
            };

            let deck_2 = if let Some(d) = self.deck_cache.get(deck_2_path) {
                d.clone()
            } else {
                let d = Deck::from_file(deck_2_path).map_err(|e| {
                    PyIOError::new_err(format!("Failed to load deck: {}", e))
                })?;
                self.deck_cache.insert(deck_2_path.to_string(), d.clone());
                d
            };

            let players = if rng.next_u32() % 2 == 0 {
                create_players(deck_1, deck_2, vec![PlayerCode::H, PlayerCode::H])
            } else {
                create_players(deck_2, deck_1, vec![PlayerCode::H, PlayerCode::H])
            };
            let game_seed = rng.next_u64();
            self.games.push(Game::new(players, game_seed));
        }

        let action_space_size = encoding::get_action_space_size();
        let obs_dim = if !self.games.is_empty() {
            encoding::observation_length(&self.games[0].state())
        } else {
            0
        };

        let py_obs = numpy::PyArray2::<f32>::zeros_bound(py, [self.batch_size, obs_dim], false);
        let py_cp = numpy::PyArray1::<usize>::zeros_bound(py, [self.batch_size], false);
        let py_mask = numpy::PyArray2::<f32>::zeros_bound(py, [self.batch_size, action_space_size], false);

        {
            let mut obs_view = py_obs.try_readwrite().unwrap();
            let mut cp_view = py_cp.try_readwrite().unwrap();
            let mut mask_view = py_mask.try_readwrite().unwrap();

            let obs_slice = obs_view.as_slice_mut().unwrap();
            let cp_slice = cp_view.as_slice_mut().unwrap();
            let mask_slice = mask_view.as_slice_mut().unwrap();

            for (i, game) in self.games.iter().enumerate() {
                let current_obs = encoding::encode_observation(game.state(), game.state().current_player);
                let start = i * obs_dim;
                obs_slice[start..start + obs_dim].copy_from_slice(&current_obs);

                cp_slice[i] = game.state().current_player;

                let (_, legal_actions) = generate_possible_actions(game.state());
                let mask_start = i * action_space_size;
                for action in legal_actions {
                    if let Some(id) = encoding::encode_action(&action.action) {
                        if id < action_space_size {
                            mask_slice[mask_start + id] = 1.0;
                        }
                    }
                }
            }
        }

        Ok((py_obs.unbind(), py_cp.unbind(), py_mask.unbind()))
    }

    // Returns (obs, rewards, dones, timed_out, valid_mask)
    // valid_mask indicates if the environment was active and stepped successfully
    pub fn step(&mut self, actions: Vec<usize>) -> PyResult<(Vec<Vec<f32>>, Vec<f32>, Vec<bool>, Vec<bool>, Vec<bool>)> {
        if actions.len() != self.batch_size {
             return Err(PyValueError::new_err(format!(
                "Actions length {} does not match batch size {}",
                actions.len(),
                self.batch_size
            )));
        }

        let mut obs_batch = Vec::with_capacity(self.batch_size);
        let mut rew_batch = Vec::with_capacity(self.batch_size);
        let mut done_batch = Vec::with_capacity(self.batch_size);
        let mut timed_out_batch = Vec::with_capacity(self.batch_size); // Not used currently
        let mut valid_batch = Vec::with_capacity(self.batch_size); // True if stepped

        for (i, &action_id) in actions.iter().enumerate() {
            let game = &mut self.games[i];

            let state_before = game.state().clone();
            let points_before = state_before.points;

            if game.is_game_over() {
                // Game already over
                obs_batch.push(encoding::encode_observation(game.state(), game.state().current_player));
                rew_batch.push(0.0);
                done_batch.push(true);
                timed_out_batch.push(false);
                valid_batch.push(false); // active=False
                continue;
            }
            
            // 1. Generate legal actions
            let (_actor, legal_actions) = generate_possible_actions(game.state());
            
            // 2. Decode/Find Action
            let mut found_action: Option<Action> = None;
            for action in &legal_actions {
                if let Some(enc_id) = encoding::encode_action(&action.action) {
                    if enc_id == action_id {
                        found_action = Some(action.clone());
                        break;
                    }
                }
            }

            if let Some(action) = found_action {
                
                let actor_before = action.actor; // Should match _actor

                // 3. Apply Action
                game.apply_action(&action);
                
                // 4. Check status
                let done = game.is_game_over();
                
                // 5. Reward
                let mut r = 0.0;

                // Win/Loss reward
                if done {
                    if let Some(GameOutcome::Win(winner)) = game.state().winner {
                        if winner == actor_before {
                            r += self.win_reward; // Win
                        } else {
                            r -= self.win_reward; // Loss
                        }
                    }
                }

                // Point reward
                if self.point_reward != 0.0 {
                    let points_after = game.state().points;
                    let opponent = (actor_before + 1) % 2;
                    let point_diff = (points_after[actor_before] as f32 - points_before[actor_before] as f32)
                        - (points_after[opponent] as f32 - points_before[opponent] as f32);
                    r += self.point_reward * point_diff;
                }

                // Damage reward
                if self.damage_reward != 0.0 {
                    if let SimpleAction::Attack(_) = action.action {
                        let opponent = (actor_before + 1) % 2;
                        let mut total_damage = 0.0;
                        for pos in 0..4 {
                            if let (Some(before), Some(after)) = (
                                &state_before.in_play_pokemon[opponent][pos],
                                &game.state().in_play_pokemon[opponent][pos],
                            ) {
                                if (before.remaining_hp as i32) > (after.remaining_hp as i32) {
                                    total_damage += (before.remaining_hp - after.remaining_hp) as f32;
                                }
                            } else if let Some(before) = &state_before.in_play_pokemon[opponent][pos] {
                                // KO happened
                                total_damage += before.remaining_hp as f32;
                            }
                        }
                        r += self.damage_reward * total_damage;
                    }
                }

                obs_batch.push(encoding::encode_observation(game.state(), game.state().current_player));
                rew_batch.push(r);
                done_batch.push(done);
                timed_out_batch.push(false);
                valid_batch.push(true);

            } else {
                 return Err(PyValueError::new_err(format!(
                    "Action ID {} is not legal for game {}",
                    action_id, i
                )));
            }
        }

        Ok((obs_batch, rew_batch, done_batch, timed_out_batch, valid_batch))
    }

    pub fn get_legal_actions(&self) -> PyResult<Vec<Vec<usize>>> {
        let mut batched_legal_actions = Vec::with_capacity(self.batch_size);

        for game in &self.games {
            if game.is_game_over() {
                batched_legal_actions.push(Vec::new());
                continue;
            }

            let (_, actions) = generate_possible_actions(game.state());
            let mut action_ids = Vec::with_capacity(actions.len());
            for action in actions {
                if let Some(id) = encoding::encode_action(&action.action) {
                    action_ids.push(id);
                }
            }
            batched_legal_actions.push(action_ids);
        }
        Ok(batched_legal_actions)
    }



    #[pyo3(name = "sample_and_step")]
    pub fn sample_and_step<'py>(
        &mut self,
        py: Python<'py>,
        logits: numpy::PyReadonlyArray2<'py, f32>,
    ) -> PyResult<(
        Py<numpy::PyArray2<f32>>,
        Py<numpy::PyArray1<f32>>,
        Py<numpy::PyArray1<bool>>,
        Py<numpy::PyArray1<bool>>,
        Py<numpy::PyArray1<bool>>,
        Py<numpy::PyArray1<usize>>,
        Py<numpy::PyArray1<f32>>,
        Py<numpy::PyArray1<usize>>,
        Py<numpy::PyArray2<f32>>,
    )> {
        // ... (lines 1417-1569 are unchanged) ...
        let logits_array = logits.as_array();
        let shape = logits_array.shape();
        
        if shape[0] != self.batch_size {
            return Err(PyValueError::new_err(format!(
                "Logits batch size {} does not match simulator batch size {}",
                shape[0],
                self.batch_size
            )));
        }

        let win_reward = self.win_reward;
        let point_reward = self.point_reward;
        let damage_reward = self.damage_reward;

        let results: Vec<_> = py.allow_threads(|| {
            self.games
                .par_iter_mut()
                .enumerate()
                .map(|(i, game)| {
                    let logit_row = logits_array.row(i);
                     // ... (simulation logic unchanged, omitted for brevity) ...
                     // (lines 1428-1568)
                    if game.is_game_over() {
                        return (
                            encoding::encode_observation(game.state(), game.state().current_player),
                            0.0,
                            true,
                            false,
                            false,
                            0,
                            0.0,
                            game.state().current_player,
                            vec![0.0; encoding::get_action_space_size()],
                        );
                    }

                    let state_before = game.state().clone();
                    let points_before = state_before.points;

                    let mut sampled_action_id = 0;
                    let mut sampled_action: Option<Action> = None;
                    let mut sampled_log_prob = 0.0;

                    let (actor, legal_actions) = generate_possible_actions(game.state());
                    let mut filtered_probs = Vec::with_capacity(legal_actions.len());
                    let mut filtered_action_ids = Vec::with_capacity(legal_actions.len());

                    for action in &legal_actions {
                        if let Some(id) = encoding::encode_action(&action.action) {
                            if id < logit_row.len() {
                                let val = logit_row[id];
                                filtered_probs.push(val);
                                filtered_action_ids.push(id);
                            }
                        }
                    }

                    if !filtered_probs.is_empty() {
                        let max_logit = filtered_probs
                            .iter()
                            .fold(f32::NEG_INFINITY, |a: f32, &b| a.max(b));
                        let mut sum_exp = 0.0;
                        for p in filtered_probs.iter_mut() {
                            *p = (*p - max_logit).exp();
                            sum_exp += *p;
                        }
                        for p in filtered_probs.iter_mut() {
                            *p /= sum_exp;
                        }

                        let dist = WeightedIndex::new(&filtered_probs).unwrap();
                        let idx = dist.sample(&mut rand::thread_rng());

                        sampled_action_id = filtered_action_ids[idx];
                        sampled_log_prob = (filtered_probs[idx] + 1e-10).ln();

                        for action in &legal_actions {
                            if let Some(id) = encoding::encode_action(&action.action) {
                                if id == sampled_action_id {
                                    sampled_action = Some(action.clone());
                                    break;
                                }
                            }
                        }
                    }

                    if let Some(action) = sampled_action {
                        let actor_before = action.actor;
                        game.apply_action(&action);
                        let done = game.is_game_over();

                        let mut r = 0.0;
                        if done {
                            if let Some(GameOutcome::Win(winner)) = game.state().winner {
                                if winner == actor_before {
                                    r += win_reward;
                                } else {
                                    r -= win_reward;
                                }
                            }
                        }

                        if point_reward != 0.0 {
                            let points_after = game.state().points;
                            let opponent = (actor_before + 1) % 2;
                            let point_diff = (points_after[actor_before] as f32
                                - points_before[actor_before] as f32)
                                - (points_after[opponent] as f32 - points_before[opponent] as f32);
                            r += point_reward * point_diff;
                        }

                        if damage_reward != 0.0 {
                            if let SimpleAction::Attack(_) = action.action {
                                let opponent = (actor_before + 1) % 2;
                                let mut total_damage = 0.0;
                                for pos in 0..4 {
                                    if let (Some(before), Some(after)) = (
                                        &state_before.in_play_pokemon[opponent][pos],
                                        &game.state().in_play_pokemon[opponent][pos],
                                    ) {
                                        if (before.remaining_hp as i32) > (after.remaining_hp as i32)
                                        {
                                            total_damage +=
                                                (before.remaining_hp - after.remaining_hp) as f32;
                                        }
                                    } else if let Some(before) =
                                        &state_before.in_play_pokemon[opponent][pos]
                                    {
                                        total_damage += before.remaining_hp as f32;
                                    }
                                }
                                r += damage_reward * total_damage;
                            }
                        }

                        // Get mask for next state
                        let action_space_size = encoding::get_action_space_size();
                        let mut mask_after = vec![0.0; action_space_size];
                        if !done {
                            let (_, next_legal_actions) = generate_possible_actions(game.state());
                            for action in next_legal_actions {
                                if let Some(id) = encoding::encode_action(&action.action) {
                                    if id < action_space_size {
                                        mask_after[id] = 1.0;
                                    }
                                }
                            }
                        }

                        (
                            encoding::encode_observation(game.state(), game.state().current_player),
                            r,
                            done,
                            false,
                            true,
                            sampled_action_id,
                            sampled_log_prob,
                            game.state().current_player,
                            mask_after,
                        )
                    } else {
                        // Get mask for current state (still current if couldn't step)
                        let action_space_size = encoding::get_action_space_size();
                        let mut mask_after = vec![0.0; action_space_size];
                        let (_, next_legal_actions) = generate_possible_actions(game.state());
                        for action in next_legal_actions {
                            if let Some(id) = encoding::encode_action(&action.action) {
                                if id < action_space_size {
                                    mask_after[id] = 1.0;
                                }
                            }
                        }

                        (
                            encoding::encode_observation(game.state(), game.state().current_player),
                            0.0,
                            game.is_game_over(),
                            false,
                            false,
                            0,
                            0.0,
                            game.state().current_player,
                            mask_after,
                        )
                    }
                })
                .collect()
        });

        // Unzip results into numpy arrays
        // obs is flattened
        let obs_dim: usize = if !results.is_empty() && !results[0].0.is_empty() {
            results[0].0.len()
        } else {
            // Fallback or error, assume standard size?
            // Usually games[0] has size
            if !self.games.is_empty() {
                encoding::observation_length(&self.games[0].state())
            } else {
                0
            }
        };

        // Allocate numpy arrays
        let py_obs = numpy::PyArray2::<f32>::zeros_bound(py, [self.batch_size, obs_dim], false);
        let py_rew = numpy::PyArray1::<f32>::zeros_bound(py, [self.batch_size], false);
        let py_done = numpy::PyArray1::<bool>::zeros_bound(py, [self.batch_size], false);
        let py_timeout = numpy::PyArray1::<bool>::zeros_bound(py, [self.batch_size], false);
        let py_valid = numpy::PyArray1::<bool>::zeros_bound(py, [self.batch_size], false);
        let py_act = numpy::PyArray1::<usize>::zeros_bound(py, [self.batch_size], false);
        let py_logp = numpy::PyArray1::<f32>::zeros_bound(py, [self.batch_size], false);
        let py_cp = numpy::PyArray1::<usize>::zeros_bound(py, [self.batch_size], false);
        let action_space_size = encoding::get_action_space_size();
        let py_mask = numpy::PyArray2::<f32>::zeros_bound(py, [self.batch_size, action_space_size], false);

        {
            let mut obs_view = py_obs.try_readwrite().unwrap();
            let mut rew_view = py_rew.try_readwrite().unwrap();
            let mut done_view = py_done.try_readwrite().unwrap();
            let mut timeout_view = py_timeout.try_readwrite().unwrap();
            let mut valid_view = py_valid.try_readwrite().unwrap();
            let mut act_view = py_act.try_readwrite().unwrap();
            let mut logp_view = py_logp.try_readwrite().unwrap();
            let mut cp_view = py_cp.try_readwrite().unwrap();
            let mut mask_view = py_mask.try_readwrite().unwrap();

            let obs_slice = obs_view.as_slice_mut().unwrap();
            let rew_slice = rew_view.as_slice_mut().unwrap();
            let done_slice = done_view.as_slice_mut().unwrap();
            let timeout_slice = timeout_view.as_slice_mut().unwrap();
            let valid_slice = valid_view.as_slice_mut().unwrap();
            let act_slice = act_view.as_slice_mut().unwrap();
            let logp_slice = logp_view.as_slice_mut().unwrap();
            let cp_slice = cp_view.as_slice_mut().unwrap();
            let mask_slice = mask_view.as_slice_mut().unwrap();

            for (i, (o, r, d, t, v, a, lp, cp, m)) in results.into_iter().enumerate() {
                // Copy observation
                if o.len() == obs_dim {
                     // efficient copy
                     let start = i * obs_dim;
                     let end = start + obs_dim;
                     obs_slice[start..end].copy_from_slice(&o);
                }
                
                rew_slice[i] = r;
                done_slice[i] = d;
                timeout_slice[i] = t;
                valid_slice[i] = v;
                act_slice[i] = a;
                logp_slice[i] = lp;
                cp_slice[i] = cp;

                // Copy mask
                if m.len() == action_space_size {
                    let start = i * action_space_size;
                    let end = start + action_space_size;
                    mask_slice[start..end].copy_from_slice(&m);
                }
            }
        }

        Ok((
            py_obs.unbind(),
            py_rew.unbind(),
            py_done.unbind(),
            py_timeout.unbind(),
            py_valid.unbind(),
            py_act.unbind(),
            py_logp.unbind(),
            py_cp.unbind(),
            py_mask.unbind(),
        ))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pygamestate_init_and_step() {
        use pyo3::types::PyList;

        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            // Use a hardcoded list of card IDs to avoid dependency on external files
            // We need 20 cards, max 2 copies of each name.
            // Using IDs from Genetic Apex (A1)
            let card_ids = vec![
                "A1 001", "A1 001", // Bulbasaur
                "A1 002", "A1 002", // Ivysaur
                "A1 003", "A1 003", // Venusaur
                "A1 004", "A1 004", // Charmander
                "A1 005", "A1 005", // Charmeleon
                "A1 006", "A1 006", // Charizard
                "A1 007", "A1 007", // Squirtle
                "A1 008", "A1 008", // Wartortle
                "A1 009", "A1 009", // Blastoise
                "A1 010", "A1 010", // Caterpie
            ];

            let py_list = PyList::new_bound(py, &card_ids);
            let deck_a = py_list.into_any().unbind();
            let deck_b = deck_a.clone_ref(py);

            let mut game_state =
                PyGameState::new(py, deck_a, deck_b, Some(42)).expect("Failed to create game");

            // Initial state check
            let state = game_state.get_state();
            assert_eq!(state.turn_count(), 0);

            // Step 1
            // We need to know valid actions.
            // Since we can't easily introspect actions from PyGameState directly without generate_possible_actions exposed or wrapped,
            // we assume index 0 is valid for the start of the game (usually EndTurn or Place).

            let (done, _won) = game_state.step(0).expect("Step failed");
            assert!(!done);

            // Reset
            game_state.reset(py).expect("Reset failed");
            let state = game_state.get_state();
            assert_eq!(state.turn_count(), 0);
        });
    }
}
