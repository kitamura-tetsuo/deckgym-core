use serde::{Deserialize, Serialize};

use crate::models::{EnergyType, StatusCondition};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AbilityMechanic {
    HealAllYourPokemon { amount: u32 },
    DamageOneOpponentPokemon { amount: u32 },
    SwitchActiveTypedWithBench { energy_type: EnergyType },
    ReduceDamageFromAttacks { amount: u32 },
    StartTurnRandomPokemonToHand { energy_type: EnergyType },
    PreventFirstAttack,
    ElectromagneticWall,
    // New variants for vectorization
    ApplyStatus {
        condition: StatusCondition,
        target: TargetScope,
    },
    EnergyAttachment {
        amount: u32,
        energy_type: Option<EnergyType>,
        from_zone: String, // "Deck", "Discard", "EnergyZone", "Field"
        target: TargetScope,
    },
    SwitchSelfWithBench,
    ForcedSwitchActive,
    Heal {
        amount: u32,
        target: TargetScope,
    },
    MoveEnergy {
        amount: u32,
        energy_type: EnergyType,
        from: TargetScope,
        to: TargetScope,
    },
    DamageOpponent {
        amount: u32,
        target: TargetScope,
    },
    MoveDamage {
        amount: Option<u32>, // None = All
        from: TargetScope,
        to: TargetScope,
    },
    ChargeSelfAndEndTurn {
        energy_type: EnergyType,
        amount: u32,
    },
    Search {
        target_type: String,
        amount: u32,
        from_zone: String,
    },
    OnEnergyAttachDamage {
        energy_type: EnergyType,
        amount: u32,
        from_zone: String,
        target: TargetScope,
    },
    NoRetreatCost,
    Safeguard,
    IncreasedPoisonDamage { amount: u32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TargetScope {
    SelfActive,
    SelfBench,
    SelfBoard,
    OpponentActive,
    OpponentBench,
    OpponentBoard,
}
