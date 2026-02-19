#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use deckgym::actions::{apply_action, Action, SimpleAction};
    use deckgym::card_ids::CardId;
    use deckgym::database::get_card_by_enum;
    use deckgym::to_playable_card;
    use deckgym::{Deck, State};
    use deckgym::generate_possible_trainer_actions;

    #[test]
    fn test_lucky_ice_pop_healing_and_return() {
        let mut state = State::new(&Deck::default(), &Deck::default());
        state.turn_count = 1; // Trainers can only be played after turn 0
        let lucky_ice_pop = get_card_by_enum(CardId::B2145LuckyIcePop);
        let mankey = get_card_by_enum(CardId::A1141Mankey); // 60 HP

        // Setup Player 0
        // Active: Mankey (Damage 30, HP 60 -> Remaining 30)
        let mut played_mankey = to_playable_card(&mankey, false);
        played_mankey.remaining_hp = 30;
        state.in_play_pokemon[0][0] = Some(played_mankey);

        // Hand: Lucky Ice Pop
        state.hands[0].push(lucky_ice_pop.clone());
        state.hands_visibility[0].push(true);

        let trainer_card = match &lucky_ice_pop {
            deckgym::models::Card::Trainer(t) => t,
            _ => panic!("Not a trainer"),
        };

        let actions = generate_possible_trainer_actions(&state, trainer_card);

        assert!(actions.is_some(), "Should be able to play Lucky Ice Pop");
        let actions = actions.unwrap();
        assert!(!actions.is_empty(), "Actions should not be empty");

        // Test Logic Execution (Heal + Return on Heads)
        let action = Action {
            actor: 0,
            action: SimpleAction::Play {
                trainer_card: trainer_card.clone(),
            },
            is_stack: false,
        };

        // We try seeds until verify return to hand
        let mut found_heads = false;
        let mut found_tails = false;

        for seed in 0..100 {
            let mut test_state = state.clone();
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

            apply_action(&mut rng, &mut test_state, &action);

            // Verify Healing
            let active = test_state.in_play_pokemon[0][0].as_ref().unwrap();
            assert_eq!(active.remaining_hp, 50, "Should heal 20 damage (30 -> 50)");

            // Verify Return to Hand
            let hand_size = test_state.hands[0].len();
            let discard_size = test_state.discard_piles[0].len();

            if hand_size == 1 {
                found_heads = true;
                assert_eq!(discard_size, 0, "Should not be in discard if returned");
            } else {
                found_tails = true;
                assert_eq!(discard_size, 1, "Should be in discard if not returned");
            }

            if found_heads && found_tails {
                break;
            }
        }

        assert!(
            found_heads,
            "Should check that heads is possible (return to hand)"
        );
        assert!(found_tails, "Should check that tails is possible (discard)");
    }

    #[test]
    fn test_lucky_ice_pop_cannot_play_full_hp() {
        let mut state = State::new(&Deck::default(), &Deck::default());
        state.turn_count = 1;
        let lucky_ice_pop = get_card_by_enum(CardId::B2145LuckyIcePop);
        let mankey = get_card_by_enum(CardId::A1141Mankey); // 60 HP

        // Setup Player 0
        // Active: Mankey (Full HP)
        let played_mankey = to_playable_card(&mankey, false);
        state.in_play_pokemon[0][0] = Some(played_mankey);

        // Hand: Lucky Ice Pop
        state.hands[0].push(lucky_ice_pop.clone());
        state.hands_visibility[0].push(true);

        let trainer_card = match &lucky_ice_pop {
            deckgym::models::Card::Trainer(t) => t,
            _ => panic!("Not a trainer"),
        };

        let actions = generate_possible_trainer_actions(&state, trainer_card);

        if let Some(acts) = actions {
            assert!(acts.is_empty(), "Should not be able to play if full HP");
        }
    }
}
