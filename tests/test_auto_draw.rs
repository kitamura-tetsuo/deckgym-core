
#[cfg(test)]
mod tests {
    use deckgym::{
        generate_possible_actions,
        actions::{Action, SimpleAction},
        players::{Player, RandomPlayer},
        Game, State,
        test_helpers::load_test_decks,
    };
    use rand::SeedableRng;

    #[test]
    fn test_auto_draw_at_turn_start() {
        // 1. Setup Game
        let (deck_a, deck_b) = load_test_decks();
        let player_a = Box::new(RandomPlayer { deck: deck_a.clone() });
        let player_b = Box::new(RandomPlayer { deck: deck_b.clone() });
        let players: Vec<Box<dyn Player + Send>> = vec![player_a, player_b];
        let mut game = Game::new(players, 42);

        // 2. Advance through setup (turn 0)
        while game.state().turn_count == 0 {
            game.play_tick();
        }

        // Now we are at turn 1. 
        // In the OLD behavior, the first action of the turn should be DrawCard.
        // In the NEW behavior, the draw should have already happened, so DrawCard should NOT be an option 
        // (unless there are other card effects, but basic draw is what we care about).
        // And the hand size should reflect the draw.

        let curr_player = game.state().current_player;
        let hand_size = game.state().hands[curr_player].len();
        
        // Check actions
        let (_, actions) = generate_possible_actions(game.state());
        
        println!("Turn: {}", game.state().turn_count);
        println!("Current Player: {}", curr_player);
        println!("Hand Size: {}", hand_size);
        println!("Actions: {:?}", actions.iter().map(|a| &a.action).collect::<Vec<_>>());

        // For the purpose of this test running BEFORE changes, we expect to see DrawCard.
        // After changes, we expect NOT to see DrawCard, and hand size to be +1 (implied).
        
        // Let's assert based on the DESIRED behavior, so this test fails now and passes later.
        // Initial hand is 5. If auto-draw works, it should be 6 (or more if mulligans, but seeded rng should be stable-ish).
        // Actually, RandomPlayer might Mulligan.
        // But the key is that 'DrawCard' action should NOT be present.
        
        let has_draw_action = actions.iter().any(|a| matches!(a.action, SimpleAction::DrawCard { .. }));
        assert!(!has_draw_action, "Should not have explicit DrawCard action at start of turn");
    }
}
