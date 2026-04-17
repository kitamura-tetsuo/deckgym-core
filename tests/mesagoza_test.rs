use deckgym::{
    actions::{apply_action, Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    models::Card,
    Deck, State,
};
use rand::{rngs::StdRng, SeedableRng};

#[test]
fn test_mesagoza_action_generation() {
    let mut state = State::new(&Deck::default(), &Deck::default());
    let mesagoza = get_card_by_enum(CardId::B2a093Mesagoza);

    // No action without stadium
    let (_, actions) = deckgym::move_generation::generate_possible_actions(&state);
    assert!(!actions
        .iter()
        .any(|a| matches!(a.action, SimpleAction::UseStadium)));

    // Set Mesagoza
    state.set_stadium(mesagoza, 0);

    // Action should be present
    let (current_player, dbg_actions) = deckgym::move_generation::generate_possible_actions(&state);
    println!("Current Player: {}", current_player);
    println!("Actions: {:?}", dbg_actions);
    assert!(
        dbg_actions
            .iter()
            .any(|a| matches!(a.action, SimpleAction::UseStadium)),
        "Action UseStadium should be present after setting stadium"
    );

    // Use it
    state.stadium_used_this_turn = true;

    // Action should be gone
    let (_, actions_after) = deckgym::move_generation::generate_possible_actions(&state);
    assert!(!actions_after
        .iter()
        .any(|a| matches!(a.action, SimpleAction::UseStadium)));
}

#[test]
fn test_mesagoza_reset_on_new_stadium() {
    let mut state = State::new(&Deck::default(), &Deck::default());
    let mesagoza = get_card_by_enum(CardId::B2a093Mesagoza);

    state.set_stadium(mesagoza.clone(), 0);
    state.stadium_used_this_turn = true;

    // Played a new Mesagoza (replaces old)
    state.set_stadium(mesagoza, 0);
    assert_eq!(state.stadium_used_this_turn, false);
}

#[test]
fn test_mesagoza_effect_heads() {
    let mut state = State::new(&Deck::default(), &Deck::default());
    let mesagoza = get_card_by_enum(CardId::B2a093Mesagoza);
    let pikachu = get_card_by_enum(CardId::A1094Pikachu);

    // Add Pikachu to deck
    state.decks[0].cards.push(pikachu.clone());
    state.set_stadium(mesagoza, 0);

    let action = Action {
        actor: 0,
        action: SimpleAction::UseStadium,
        is_stack: false,
    };

    // Find a seed that gives Heads
    let mut heads_seed = None;
    for seed in 0..200 {
        let mut test_state = state.clone();
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        deckgym::actions::apply_action(&mut rng, &mut test_state, &action);
        if test_state.stadium_used_this_turn
            && test_state.hands[0]
                .iter()
                .any(|c| c.get_id() == pikachu.get_id())
        {
            heads_seed = Some(seed);
            break;
        }
    }

    let heads_seed = heads_seed.expect("Should find a seed for Heads in 200 tries");
    let mut rng = rand::rngs::StdRng::seed_from_u64(heads_seed);
    deckgym::actions::apply_action(&mut rng, &mut state, &action);

    assert!(state.stadium_used_this_turn);
    assert!(state.hands[0]
        .iter()
        .any(|c| c.get_id() == pikachu.get_id()));
}

#[test]
fn test_mesagoza_effect_tails() {
    let mut state = State::new(&Deck::default(), &Deck::default());
    let mesagoza = get_card_by_enum(CardId::B2a093Mesagoza);
    let pikachu = get_card_by_enum(CardId::A1094Pikachu);

    state.decks[0].cards.push(pikachu.clone());
    state.set_stadium(mesagoza, 0);

    let action = Action {
        actor: 0,
        action: SimpleAction::UseStadium,
        is_stack: false,
    };

    // Find a seed for Tails
    let mut tails_seed = None;
    for seed in 0..200 {
        let mut test_state = state.clone();
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        deckgym::actions::apply_action(&mut rng, &mut test_state, &action);
        if test_state.stadium_used_this_turn
            && !test_state.hands[0]
                .iter()
                .any(|c| c.get_id() == pikachu.get_id())
        {
            tails_seed = Some(seed);
            break;
        }
    }

    let tails_seed = tails_seed.expect("Should find a seed for Tails in 200 tries");
    let mut rng = rand::rngs::StdRng::seed_from_u64(tails_seed);
    deckgym::actions::apply_action(&mut rng, &mut state, &action);

    assert!(state.stadium_used_this_turn);
    assert!(!state.hands[0]
        .iter()
        .any(|c| c.get_id() == pikachu.get_id()));
}

#[test]
fn test_mesagoza_comprehensive() {
    let mut state = State::new(&Deck::default(), &Deck::default());
    let mesagoza = get_card_by_enum(CardId::B2a093Mesagoza);
    let pikachu = get_card_by_enum(CardId::A1094Pikachu);
    let bulbasaur = get_card_by_enum(CardId::A1001Bulbasaur);

    // Clear hands and decks to be sure
    state.hands[0].clear();
    state.decks[0].cards.clear();

    state.decks[0].cards.push(pikachu.clone());
    state.decks[0].cards.push(bulbasaur.clone());
    state.set_stadium(mesagoza, 0);

    let action = Action {
        actor: 0,
        action: SimpleAction::UseStadium,
        is_stack: false,
    };

    let mut found_pikachu = false;
    let mut found_bulbasaur = false;
    let mut found_tails = false;

    // Run samples to cover all outcomes (Tails 50%, Pikachu 25%, Bulbasaur 25%)
    for seed in 0..100 {
        let mut test_state = state.clone();
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        deckgym::actions::apply_action(&mut rng, &mut test_state, &action);

        assert!(test_state.stadium_used_this_turn);

        let has_pikachu = test_state.hands[0]
            .iter()
            .any(|c| c.get_id() == pikachu.get_id());
        let has_bulbasaur = test_state.hands[0]
            .iter()
            .any(|c| c.get_id() == bulbasaur.get_id());

        if has_pikachu {
            found_pikachu = true;
        } else if has_bulbasaur {
            found_bulbasaur = true;
        } else {
            found_tails = true;
        }

        if found_pikachu && found_bulbasaur && found_tails {
            break;
        }
    }

    assert!(found_pikachu, "Should have found Pikachu at least once");
    assert!(found_bulbasaur, "Should have found Bulbasaur at least once");
    assert!(found_tails, "Should have got Tails at least once");
}
#[test]
fn test_play_mesagoza_from_hand() {
    let mesagoza = get_card_by_enum(CardId::B2a093Mesagoza);
    let mut state = State::new(&Deck::default(), &Deck::default());
    state.hands[0].push(mesagoza.clone());
    state.hands_visibility[0].push(true);
    state.turn_count = 1;

    let action = Action {
        actor: 0,
        action: SimpleAction::Play {
            trainer_card: match mesagoza {
                Card::Trainer(t) => t,
                _ => panic!("Expected trainer card"),
            },
        },
        is_stack: false,
    };

    let mut rng = StdRng::seed_from_u64(42);
    apply_action(&mut rng, &mut state, &action);

    assert!(
        state.get_stadium().is_some(),
        "Stadium should be set after playing Mesagoza"
    );
    assert_eq!(
        state.get_stadium().unwrap().get_id(),
        "B2a 093",
        "Stadium should be Mesagoza"
    );
}
