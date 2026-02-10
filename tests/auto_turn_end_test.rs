use deckgym::{
    actions::{Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    to_playable_card,
    models::EnergyType,
    players::{Player, RandomPlayer},
    test_helpers::load_test_decks,
    Game,
};

#[test]
fn test_auto_turn_end_after_attack() {
    let (deck_a, deck_b) = load_test_decks();
    
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

    // Fast forward to turn 3 (so player can attack)
    state.turn_count = 3;
    state.current_player = 0;
    
    game.set_state(state);

    // Use Gnaw attack (index 0)
    let action = Action {
        actor: 0,
        action: SimpleAction::Attack(0),
        is_stack: false,
    };

    game.apply_action(&action);
    
    let state = game.get_state_clone();

    // Verification
    // BEFORE FIX: Turn count should be 3, current player 0, stack has EndTurn
    // AFTER FIX: Turn count should be 4, current player 1, stack empty
    
    // Check if turn advanced
    if state.current_player == 0 {
        panic!("Turn did not advance! Still player 0's turn. Stack: {:?}", state.move_generation_stack);
    }
    
    assert_eq!(state.current_player, 1);
    assert_eq!(state.current_player, 1);
    assert_eq!(state.turn_count, 4);
    // Stack should contain DrawCard for the next player
    assert!(!state.move_generation_stack.is_empty());
    let (actor, actions) = state.move_generation_stack.last().unwrap();
    assert_eq!(*actor, 1);
    assert!(matches!(actions[0], SimpleAction::DrawCard { .. }));
}
