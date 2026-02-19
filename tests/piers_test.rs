use common::get_initialized_game;
use deckgym::{
    actions::{Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    models::{Card, EnergyType, PlayedCard, TrainerCard},

};

mod common;

fn make_piers_trainer_card() -> TrainerCard {
    match get_card_by_enum(CardId::B2152Piers) {
        Card::Trainer(tc) => tc,
        _ => panic!("Expected trainer card"),
    }
}

fn make_obstagoon() -> PlayedCard {
    PlayedCard::from_id(CardId::B2100GalarianObstagoon)
}

#[test]
fn test_piers_discards_two_energies() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    let current_player = state.current_player;
    let opponent = (current_player + 1) % 2;

    // Set up player with Obstagoon
    state.in_play_pokemon[current_player][0] = Some(make_obstagoon());
    
    // Set up opponent with 2 energies
    let mut bulbasaur = PlayedCard::from_id(CardId::A1001Bulbasaur);
    bulbasaur.attached_energy = vec![EnergyType::Grass, EnergyType::Grass];
    state.in_play_pokemon[opponent][0] = Some(bulbasaur);

    let trainer_card = make_piers_trainer_card();
    state.hands[current_player].push(Card::Trainer(trainer_card.clone()));
    state.hands_visibility[current_player].push(true);
    game.set_state(state);

    let play_action = Action {
        actor: current_player,
        action: SimpleAction::Play { trainer_card },
        is_stack: false,
    };
    game.apply_action(&play_action);

    let state = game.get_state_clone();
    let opponent_active = state.get_active(opponent);
    assert_eq!(opponent_active.attached_energy.len(), 0, "Piers should discard 2 energies");
}

#[test]
fn test_piers_discards_one_energy_if_only_one_available() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    let current_player = state.current_player;
    let opponent = (current_player + 1) % 2;

    // Set up player with Obstagoon
    state.in_play_pokemon[current_player][0] = Some(make_obstagoon());
    
    // Set up opponent with 1 energy
    let mut bulbasaur = PlayedCard::from_id(CardId::A1001Bulbasaur);
    bulbasaur.attached_energy = vec![EnergyType::Grass];
    state.in_play_pokemon[opponent][0] = Some(bulbasaur);

    let trainer_card = make_piers_trainer_card();
    state.hands[current_player].push(Card::Trainer(trainer_card.clone()));
    state.hands_visibility[current_player].push(true);
    game.set_state(state);

    let play_action = Action {
        actor: current_player,
        action: SimpleAction::Play { trainer_card },
        is_stack: false,
    };
    game.apply_action(&play_action);

    let state = game.get_state_clone();
    let opponent_active = state.get_active(opponent);
    assert_eq!(opponent_active.attached_energy.len(), 0, "Piers should discard the only available energy");
}

#[test]
fn test_piers_is_not_playable_if_opponent_has_no_energy() {
    let game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    let current_player = state.current_player;
    let opponent = (current_player + 1) % 2;

    // Set up player with Obstagoon
    state.in_play_pokemon[current_player][0] = Some(make_obstagoon());
    
    // Set up opponent with 0 energy
    state.in_play_pokemon[opponent][0] = Some(PlayedCard::from_id(CardId::A1001Bulbasaur));

    let trainer_card = make_piers_trainer_card();
    state.hands[current_player].push(Card::Trainer(trainer_card.clone()));
    state.hands_visibility[current_player].push(true);
    
    // Verify move generation doesn't include Piers
    let actions = deckgym::generate_possible_actions(&state);
    let has_piers = actions.1.iter().any(|a| match &a.action {
        SimpleAction::Play { trainer_card: tc } => tc.name == "Piers",
        _ => false,
    });

    assert!(!has_piers, "Piers should not be playable if opponent has no energy");
}

#[test]
fn test_piers_is_not_playable_if_no_obstagoon() {
    let game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    let current_player = state.current_player;
    let opponent = (current_player + 1) % 2;

    // Set up player with something else
    state.in_play_pokemon[current_player][0] = Some(PlayedCard::from_id(CardId::A1001Bulbasaur));
    
    // Set up opponent with 2 energies
    let mut bulbasaur = PlayedCard::from_id(CardId::A1001Bulbasaur);
    bulbasaur.attached_energy = vec![EnergyType::Grass, EnergyType::Grass];
    state.in_play_pokemon[opponent][0] = Some(bulbasaur);

    let trainer_card = make_piers_trainer_card();
    state.hands[current_player].push(Card::Trainer(trainer_card.clone()));
    state.hands_visibility[current_player].push(true);

    // Verify move generation doesn't include Piers
    let actions = deckgym::generate_possible_actions(&state);
    let has_piers = actions.1.iter().any(|a| match &a.action {
        SimpleAction::Play { trainer_card: tc } => tc.name == "Piers",
        _ => false,
    });

    assert!(!has_piers, "Piers should not be playable if no Galarian Obstagoon in play");
}
