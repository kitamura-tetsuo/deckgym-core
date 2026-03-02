use common::get_initialized_game;
use deckgym::{
    actions::{Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    generate_possible_actions,
    models::{EnergyType, PlayedCard},
};

mod common;

/// Test Greavard B2a 052 - Soul Shot
/// Attack should NOT be selectable when player has no cards in hand.
#[test]
fn test_greavard_soul_shot_no_card_in_hand() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.current_player = 0;

    // Set up Greavard with Psychic energy
    state.in_play_pokemon[0][0] =
        Some(PlayedCard::from_id(CardId::B2a052Greavard).with_energy(vec![EnergyType::Psychic]));

    // Set up opponent
    state.in_play_pokemon[1][0] = Some(PlayedCard::from_id(CardId::A1001Bulbasaur));

    // Empty hand
    state.hands[0] = vec![];
    state.hands_visibility[0] = vec![];

    game.set_state(state);

    let (_actor, valid_actions) = generate_possible_actions(game.state());
    assert!(
        !valid_actions
            .iter()
            .any(|a| matches!(a.action, SimpleAction::Attack(0))),
        "Soul Shot should not be selectable when hand is empty"
    );
}

/// Test Greavard B2a 052 - Soul Shot
/// Attack should deal 30 damage and prompt for a card discard when hand has cards.
#[test]
fn test_greavard_soul_shot_with_card_in_hand() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.current_player = 0;

    // Set up Greavard with Psychic energy
    state.in_play_pokemon[0][0] =
        Some(PlayedCard::from_id(CardId::B2a052Greavard).with_energy(vec![EnergyType::Psychic]));

    // Set up opponent with enough HP to survive
    state.in_play_pokemon[1][0] = Some(PlayedCard::from_id(CardId::A1001Bulbasaur).with_hp(100));
    state.in_play_pokemon[1][0].as_mut().unwrap().total_hp = 100;

    // Give player one card in hand
    state.hands[0] = vec![get_card_by_enum(CardId::A1001Bulbasaur)];
    state.hands_visibility[0] = vec![true];

    game.set_state(state);

    // Verify attack IS selectable
    let (_actor, valid_actions) = generate_possible_actions(game.state());
    assert!(
        valid_actions
            .iter()
            .any(|a| matches!(a.action, SimpleAction::Attack(0))),
        "Soul Shot should be selectable when hand has at least 1 card"
    );

    // Apply the attack
    let attack_action = Action {
        actor: 0,
        action: SimpleAction::Attack(0),
        is_stack: false,
    };
    game.apply_action(&attack_action);

    let state = game.get_state_clone();

    // Opponent should have taken 30 damage (100 - 30 = 70)
    let opponent_active = state.get_active(1);
    assert_eq!(
        opponent_active.remaining_hp, 70,
        "Opponent should have 70 HP remaining (100 - 30)"
    );

    // Should be prompted to discard a card
    assert!(
        !state.move_generation_stack.is_empty(),
        "Move generation stack should prompt for card discard"
    );
    let (_actor, choices) = state.move_generation_stack.last().unwrap();
    assert!(
        choices
            .iter()
            .any(|a| matches!(a, SimpleAction::DiscardOwnCard { .. })),
        "Should have DiscardOwnCard choices"
    );
}

/// Test Houndstone B2a 053 - Last Respects
/// Should deal exactly 50 damage when discard pile has no Psychic Pokémon.
#[test]
fn test_houndstone_last_respects_no_psychic_in_discard() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.current_player = 0;

    // Set up Houndstone with Psychic + Colorless energy
    state.in_play_pokemon[0][0] = Some(
        PlayedCard::from_id(CardId::B2a053Houndstone).with_energy(vec![
            EnergyType::Psychic,
            EnergyType::Colorless,
        ]),
    );

    // Set up opponent with enough HP to survive
    state.in_play_pokemon[1][0] = Some(PlayedCard::from_id(CardId::A1001Bulbasaur).with_hp(100));
    state.in_play_pokemon[1][0].as_mut().unwrap().total_hp = 100;

    // Empty discard pile (no Psychic Pokémon)
    state.discard_piles[0] = vec![];

    game.set_state(state);

    let attack_action = Action {
        actor: 0,
        action: SimpleAction::Attack(0),
        is_stack: false,
    };
    game.apply_action(&attack_action);

    let state = game.get_state_clone();

    // Should deal exactly 50 damage (base only, no bonus)
    let opponent_active = state.get_active(1);
    assert_eq!(
        opponent_active.remaining_hp, 50,
        "Opponent should have 50 HP remaining (100 - 50 base damage)"
    );
}

/// Test Houndstone B2a 053 - Last Respects
/// Should deal 50 + 20 * N damage when discard pile has N Psychic Pokémon.
#[test]
fn test_houndstone_last_respects_with_psychic_in_discard() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.current_player = 0;

    // Set up Houndstone with Psychic + Colorless energy
    state.in_play_pokemon[0][0] = Some(
        PlayedCard::from_id(CardId::B2a053Houndstone).with_energy(vec![
            EnergyType::Psychic,
            EnergyType::Colorless,
        ]),
    );

    // Set up opponent with enough HP to survive multiple scenarios
    state.in_play_pokemon[1][0] = Some(PlayedCard::from_id(CardId::A1001Bulbasaur).with_hp(200));
    state.in_play_pokemon[1][0].as_mut().unwrap().total_hp = 200;

    // Put 3 Psychic Pokémon in the discard pile
    // B2a052Greavard is a Psychic Pokémon
    state.discard_piles[0] = vec![
        get_card_by_enum(CardId::B2a052Greavard),
        get_card_by_enum(CardId::B2a050Flittle),
        get_card_by_enum(CardId::B2a046Fidough),
        // Fidough is also Psychic type
    ];

    game.set_state(state);

    let attack_action = Action {
        actor: 0,
        action: SimpleAction::Attack(0),
        is_stack: false,
    };
    game.apply_action(&attack_action);

    let state = game.get_state_clone();

    // Greavard is Psychic, Flittle is Psychic, Fidough is Psychic
    // So: 50 + 3 * 20 = 110 damage
    let opponent_active = state.get_active(1);
    assert_eq!(
        opponent_active.remaining_hp, 90,
        "Opponent should have 90 HP remaining (200 - 50 - 3*20 = 90)"
    );
}

/// Test Houndstone B2a 053 - Last Respects
/// Non-Psychic Pokémon in discard pile should not count toward bonus damage.
#[test]
fn test_houndstone_last_respects_non_psychic_not_counted() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.current_player = 0;

    // Set up Houndstone with Psychic + Colorless energy
    state.in_play_pokemon[0][0] = Some(
        PlayedCard::from_id(CardId::B2a053Houndstone).with_energy(vec![
            EnergyType::Psychic,
            EnergyType::Colorless,
        ]),
    );

    // Set up opponent with enough HP to survive
    state.in_play_pokemon[1][0] = Some(PlayedCard::from_id(CardId::A1001Bulbasaur).with_hp(150));
    state.in_play_pokemon[1][0].as_mut().unwrap().total_hp = 150;

    // Put only non-Psychic Pokémon (Grass/Fire) and a Trainer in the discard pile
    state.discard_piles[0] = vec![
        get_card_by_enum(CardId::A1001Bulbasaur), // Grass
        get_card_by_enum(CardId::A1033Charmander), // Fire
        get_card_by_enum(CardId::A2b111PokeBall), // Trainer (Item)
    ];

    game.set_state(state);

    let attack_action = Action {
        actor: 0,
        action: SimpleAction::Attack(0),
        is_stack: false,
    };
    game.apply_action(&attack_action);

    let state = game.get_state_clone();

    // No Psychic Pokémon, so only 50 base damage
    let opponent_active = state.get_active(1);
    assert_eq!(
        opponent_active.remaining_hp, 100,
        "Opponent should have 100 HP remaining (150 - 50, no Psychic bonus)"
    );
}
