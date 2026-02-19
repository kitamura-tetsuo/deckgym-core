use deckgym::{
    actions::{Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    to_playable_card,
    models::EnergyType,

    test_helpers::load_test_decks,
    generate_possible_actions,
};


#[test]
fn test_manual_end_turn_still_possible() {
    let (deck_a, deck_b) = load_test_decks();
    use deckgym::{Game, players::RandomPlayer};

    
    // Create random players
    let player_a = Box::new(RandomPlayer { deck: deck_a.clone() });
    let player_b = Box::new(RandomPlayer { deck: deck_b.clone() });
    
    let mut game = Game::new(vec![player_a, player_b], 42);
    let mut state = game.get_state_clone();

    // Setup: Player 0 has a Pokemon with energy to attack
    let charmander = get_card_by_enum(CardId::A1033Charmander); // 1 Fire for Gnaw (10 dmg)
    let mut played_charmander = to_playable_card(&charmander, false);
    played_charmander.attached_energy.push(EnergyType::Fire);
    state.in_play_pokemon[0][0] = Some(played_charmander);

    // Opponent has a Pokemon
    let bulbasaur = get_card_by_enum(CardId::A1001Bulbasaur);
    state.in_play_pokemon[1][0] = Some(to_playable_card(&bulbasaur, false));

    // Fast forward to turn 3 (so player CAN attack if they want)
    state.turn_count = 3;
    state.current_player = 0;
    
    game.set_state(state.clone()); // Need to update game state for apply_action to work on correct state

    // Generate possible actions
    let (_, possible_actions) = generate_possible_actions(&state);

    // Verify EndTurn is present
    let has_end_turn = possible_actions.iter().any(|a| matches!(a.action, SimpleAction::EndTurn));
    assert!(has_end_turn, "SimpleAction::EndTurn should be available even if player can attack");

    // Also verify Attack is present (sanity check)
    let has_attack = possible_actions.iter().any(|a| matches!(a.action, SimpleAction::Attack(_)));
    assert!(has_attack, "Attack should be available");

    // Execute EndTurn manually
    let end_turn_action = Action {
        actor: 0,
        action: SimpleAction::EndTurn,
        is_stack: false,
    };
    
    game.apply_action(&end_turn_action);
    let state = game.get_state_clone();

    // Verify turn advanced
    assert_eq!(state.current_player, 1);
    assert_eq!(state.turn_count, 4);
}
