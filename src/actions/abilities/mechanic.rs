use serde::{Deserialize, Serialize};

use crate::models::EnergyType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AbilityMechanic {
    HealAllYourPokemon { amount: u32 },
    DamageOneOpponentPokemon { amount: u32 },
    SwitchActiveTypedWithBench { energy_type: EnergyType },
    ReduceDamageFromAttacks { amount: u32 },
    StartTurnRandomPokemonToHand { energy_type: EnergyType },
    PreventFirstAttack,
    ElectromagneticWall,
}
