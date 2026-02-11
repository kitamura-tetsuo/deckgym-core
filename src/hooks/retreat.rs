use crate::{
    ability_ids::AbilityId,
    card_ids::CardId,
    effects::{CardEffect, TurnEffect},
    models::{Card, EnergyType, PlayedCard},
    tools::has_tool,
    State,
};

pub(crate) fn can_retreat(state: &State) -> bool {
    let active = state.get_active(state.current_player);

    // Check if active card has CardEffect::NoRetreat
    let has_no_retreat_effect = active.get_active_effects().contains(&CardEffect::NoRetreat);

    // Check if active card is a Fossil (Fossils can never retreat)
    let is_fossil = active.is_fossil();

    !state.has_retreated && !has_no_retreat_effect && !is_fossil
}

pub(crate) fn get_retreat_cost(state: &State, card: &PlayedCard) -> Vec<EnergyType> {
    if let Card::Pokemon(pokemon_card) = &card.card {
        if let Some(ability_id) = AbilityId::from_pokemon_id(&card.get_id()) {
            if ability_id == AbilityId::A2078GiratinaLevitate && !card.attached_energy.is_empty() {
                return vec![];
            }
            if ability_id == AbilityId::A2a035RotomSpeedLink && is_speed_link_active(state, state.current_player) {
                return vec![];
            }
        }
        let mut normal_cost = pokemon_card.retreat_cost.clone();
        if has_tool(card, CardId::A4a067InflatableBoat)
            && card.get_energy_type() == Some(EnergyType::Water)
        {
            normal_cost.pop();
        }
        // Implement Retreat Cost Modifiers here
        let mut to_subtract = state
            .get_current_turn_effects()
            .iter()
            .filter(|x| matches!(x, TurnEffect::ReducedRetreatCost { .. }))
            .map(|x| match x {
                TurnEffect::ReducedRetreatCost { amount } => *amount,
                _ => 0,
            })
            .sum::<u8>();

        // Shaymin's Sky Support: As long as this Pokémon is on your Bench, your Active Basic Pokémon's Retreat Cost is 1 less.
        if pokemon_card.stage == 0 {
            // Only affects Basic Pokemon
            let current_player = state.current_player;
            for (_idx, benched_pokemon) in state.enumerate_bench_pokemon(current_player) {
                if let Some(ability_id) = AbilityId::from_pokemon_id(&benched_pokemon.get_id()) {
                    if ability_id == AbilityId::A2a069ShayminSkySupport {
                        to_subtract += 1;
                    }
                }
            }
        }

        // Retreat Effects accumulate so we add them.
        for _ in 0..to_subtract {
            normal_cost.pop(); // Remove one colorless energy from retreat cost
        }

        // Peculiar Plaza Stadium: Psychic Pokémon's Retreat Cost is 2 less
        if pokemon_card.energy_type == EnergyType::Psychic {
            if let Some(stadium) = state.get_stadium() {
                if let Some(stadium_id) = CardId::from_card_id(&stadium.get_id()) {
                    if stadium_id == CardId::B2155PeculiarPlaza {
                        // Reduce retreat cost by 2
                        normal_cost.pop();
                        normal_cost.pop();
                    }
                }
            }
        }

        // Ariados Trap Territory: Your opponent's Active Pokémon's Retreat Cost is 1 more.
        // This check needs to look at if the OPPONENT has Ariados in play
        let opponent = (state.current_player + 1) % 2;
        for (_idx, pokemon) in state.enumerate_in_play_pokemon(opponent) {
            if let Some(ability_id) = AbilityId::from_pokemon_id(&pokemon.get_id()) {
                if ability_id == AbilityId::B1a006AriadosTrapTerritory {
                    // Add 1 Colorless to retreat cost for opponent's active
                    normal_cost.push(EnergyType::Colorless);
                    break; // Only apply once
                }
            }
        }


        normal_cost
    } else {
        vec![]
    }
}

pub(crate) fn is_speed_link_active(state: &State, player_idx: usize) -> bool {
    state
        .enumerate_in_play_pokemon(player_idx)
        .any(|(_, pokemon)| {
            let name = pokemon.get_name();
            name == "Arceus" || name == "Arceus ex"
        })
}

// Test Colorless is wildcard when counting energy
#[cfg(test)]
mod tests {
    use crate::{
        card_ids::CardId, database::get_card_by_enum, effects::TurnEffect,
        hooks::core::to_playable_card,
    };

    use super::*;

    #[test]
    fn test_retreat_costs() {
        let state = State::default();
        let card = get_card_by_enum(CardId::A1055Blastoise);
        let playable_card = to_playable_card(&card, false);
        let retreat_cost = get_retreat_cost(&state, &playable_card);
        assert_eq!(
            retreat_cost,
            vec![
                EnergyType::Colorless,
                EnergyType::Colorless,
                EnergyType::Colorless
            ]
        );
    }

    #[test]
    fn test_retreat_costs_with_xspeed() {
        let mut state = State::default();
        state.add_turn_effect(TurnEffect::ReducedRetreatCost { amount: 1 }, 0);
        let card = get_card_by_enum(CardId::A1055Blastoise);
        let playable_card = to_playable_card(&card, false);
        let retreat_cost = get_retreat_cost(&state, &playable_card);
        assert_eq!(
            retreat_cost,
            vec![EnergyType::Colorless, EnergyType::Colorless]
        );
    }

    #[test]
    fn test_retreat_costs_with_two_xspeed_and_two_leafs() {
        let mut state = State::default();
        state.add_turn_effect(TurnEffect::ReducedRetreatCost { amount: 1 }, 0);
        state.add_turn_effect(TurnEffect::ReducedRetreatCost { amount: 1 }, 0);
        state.add_turn_effect(TurnEffect::ReducedRetreatCost { amount: 2 }, 0);
        let card = get_card_by_enum(CardId::A1211Snorlax);
        let playable_card = to_playable_card(&card, false);
        let retreat_cost = get_retreat_cost(&state, &playable_card);
        assert_eq!(retreat_cost, vec![]);
    }

    #[test]
    fn test_retreat_costs_with_inflatable_boat() {
        let state = State::default();
        let card = get_card_by_enum(CardId::A1055Blastoise);
        let mut playable_card = to_playable_card(&card, false);
        playable_card.attached_tool = Some(crate::database::get_card_by_enum(
            CardId::A4a067InflatableBoat,
        ));
        let retreat_cost = get_retreat_cost(&state, &playable_card);
        assert_eq!(
            retreat_cost,
            vec![EnergyType::Colorless, EnergyType::Colorless]
        );
    }

    #[test]
    fn test_rotom_speed_link_ability() {
        let rotom_card = get_card_by_enum(CardId::A2a035Rotom);
        let arceus_card = get_card_by_enum(CardId::A2a070Arceus);
        let arceus_ex_card = get_card_by_enum(CardId::A2a071ArceusEx);

        let mut state = State::default();
        let player = state.current_player;

        // 1. Check Rotom's retreat cost WITHOUT Arceus in play
        let rotom = to_playable_card(&rotom_card, false);
        state.in_play_pokemon[player][0] = Some(rotom.clone());
        
        let retreat_cost = get_retreat_cost(&state, &state.in_play_pokemon[player][0].as_ref().unwrap());
        assert!(!retreat_cost.is_empty(), "Rotom should have a retreat cost without Arceus");

        // 2. Check Rotom's retreat cost WITH Arceus in play (bench)
        let arceus = to_playable_card(&arceus_card, false);
        state.in_play_pokemon[player][1] = Some(arceus);
        
        let retreat_cost = get_retreat_cost(&state, &state.in_play_pokemon[player][0].as_ref().unwrap());
        assert!(retreat_cost.is_empty(), "Rotom should have 0 retreat cost with Arceus in play");

        // 3. Check Rotom's retreat cost WITH Arceus ex in play (replace bench)
        let arceus_ex = to_playable_card(&arceus_ex_card, false);
        state.in_play_pokemon[player][1] = Some(arceus_ex);
        
        let retreat_cost = get_retreat_cost(&state, &state.in_play_pokemon[player][0].as_ref().unwrap());
        assert!(retreat_cost.is_empty(), "Rotom should have 0 retreat cost with Arceus ex in play");
        
        // 4. Check Rotom's retreat cost when Arceus is opponent's (should not affect)
        let opponent = (player + 1) % 2;
        state.in_play_pokemon[player][1] = None;
        let arceus = to_playable_card(&arceus_card, false);
        state.in_play_pokemon[opponent][0] = Some(arceus);
        
        let retreat_cost = get_retreat_cost(&state, &state.in_play_pokemon[player][0].as_ref().unwrap());
        assert!(!retreat_cost.is_empty(), "Rotom should have a retreat cost if Arceus is only on opponent's side");
    }
}
