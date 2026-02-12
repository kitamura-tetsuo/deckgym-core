use deckgym::card_ids::CardId;
use deckgym::database::get_card_by_enum;
use deckgym::models::EnergyType;
use deckgym::state::State;
use deckgym::hooks::to_playable_card;
use rand::SeedableRng;

#[test]
fn test_darkrai_ex_nightmare_aura_lethal_damage() {
    let mut state = State::default();
    let darkrai_ex = get_card_by_enum(CardId::A2110DarkraiEx);
    let opponent_mon = get_card_by_enum(CardId::A1033Charmander); // 60 HP

    state.in_play_pokemon[0][0] = Some(to_playable_card(&darkrai_ex, false));
    
    let mut played_opponent = to_playable_card(&opponent_mon, false);
    played_opponent.remaining_hp = 20; // Set to 20 HP so 20 damage is lethal
    state.in_play_pokemon[1][0] = Some(played_opponent);
    
    state.current_player = 0;
    state.turn_count = 1;
    
    // Manual attachment of turn energy to Darkrai ex
    // This should trigger Nightmare Aura and deal 20 damage to Charmander
    let action = deckgym::actions::Action {
        actor: 0,
        action: deckgym::actions::SimpleAction::Attach {
            attachments: vec![(1, EnergyType::Darkness, 0)],
            is_turn_energy: true,
        },
        is_stack: false,
    };
    
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    deckgym::actions::apply_action(&mut rng, &mut state, &action);
    
    // Verify Charmander is knocked out
    assert!(state.in_play_pokemon[1][0].is_none(), "Charmander should be fainted");
    assert_eq!(state.points[0], 1, "Player 0 should have 1 point");
}

#[test]
fn test_electromagnetic_wall_lethal_damage() {
    let mut state = State::default();
    
    // Player 1 has Jolteon ex with Electromagnetic Wall
    let jolteon_ex = get_card_by_enum(CardId::B1081JolteonEx);
    state.in_play_pokemon[1][0] = Some(to_playable_card(&jolteon_ex, false));
    
    // Player 0 has a Pokemon with low HP
    let bulbasaur = get_card_by_enum(CardId::A1001Bulbasaur);
    let mut played_bulbasaur = to_playable_card(&bulbasaur, false);
    played_bulbasaur.remaining_hp = 20;
    state.in_play_pokemon[0][0] = Some(played_bulbasaur);
    
    state.current_player = 0;
    state.turn_count = 1;
    
    // Player 0 attaches energy from zone
    let action = deckgym::actions::Action {
        actor: 0,
        action: deckgym::actions::SimpleAction::Attach {
            attachments: vec![(1, EnergyType::Grass, 0)],
            is_turn_energy: true,
        },
        is_stack: false,
    };
    
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    deckgym::actions::apply_action(&mut rng, &mut state, &action);
    
    // Verify Bulbasaur is knocked out
    assert!(state.in_play_pokemon[0][0].is_none(), "Bulbasaur should be fainted due to Electromagnetic Wall");
    assert_eq!(state.points[1], 1, "Player 1 should have 1 point");
}
