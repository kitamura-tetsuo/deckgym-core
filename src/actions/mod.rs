mod apply_abilities_action;
mod apply_action;
mod apply_action_helpers;
mod apply_attack_action;
mod apply_trainer_action;
pub mod attacks;
mod effect_mechanic_map;
mod mutations;
mod shared_mutations;
pub mod trainer_mechanic;
mod types;
#[cfg(test)]
#[cfg(test)]
mod guzma_test;


pub use apply_action::apply_action;
pub(crate) use apply_action::apply_evolve;
pub(crate) use apply_action::forecast_action;
pub(crate) use apply_action_helpers::handle_damage;
pub use apply_trainer_action::may_effect;
pub use effect_mechanic_map::EFFECT_MECHANIC_MAP;
pub use types::Action;
pub use types::SimpleAction;
