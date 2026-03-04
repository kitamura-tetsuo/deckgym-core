use crate::{
    actions::SimpleAction,
    actions::{attacks::Mechanic, EFFECT_MECHANIC_MAP},
    effects::CardEffect,
    hooks::{contains_energy, get_attack_cost},
    State,
};

pub(crate) fn generate_attack_actions(state: &State) -> Vec<SimpleAction> {
    let current_player = state.current_player;
    let mut actions = Vec::new();
    if let Some(active_pokemon) = &state.in_play_pokemon[current_player][0] {
        // Fossil cards cannot attack
        if active_pokemon.is_fossil() {
            return actions;
        }

        // Check if the active Pokémon has the CannotAttack effect
        let active_effects = active_pokemon.get_active_effects();
        let cannot_attack = active_effects
            .iter()
            .any(|effect| matches!(effect, CardEffect::CannotAttack));
        if cannot_attack {
            return actions;
        }

        let restricted_attack_names: Vec<String> = active_effects
            .iter()
            .filter_map(|effect| match effect {
                CardEffect::CannotUseAttack(attack_name) => Some(attack_name.clone()),
                _ => None,
            })
            .collect();

        for (i, attack) in active_pokemon.get_attacks().iter().enumerate() {
            let modified_cost = get_attack_cost(&attack.energy_required, state, current_player);
            if contains_energy(active_pokemon, &modified_cost, state, current_player) {
                let attack_is_restricted = restricted_attack_names
                    .iter()
                    .any(|name| name == &attack.title);
                if attack_is_restricted {
                    continue;
                }

                // Check mechanic-specific conditions for using the attack
                if let Some(effect_text) = &attack.effect {
                    if let Some(Mechanic::DiscardHandCard { count }) =
                        EFFECT_MECHANIC_MAP.get(effect_text.as_str())
                    {
                        if state.hands[current_player].len() < *count {
                            continue;
                        }
                    }
                }

                actions.push(SimpleAction::Attack(i));
            }
        }
    }
    actions
}
