pub mod atom;
pub mod bond;
pub mod element;
pub mod molecule;
pub mod reaction;

pub use atom::Atom;
pub use bond::{Bond, BondType};
pub use element::{ElementCategory, ElementId, ElementProperties, PeriodicTable, Phase};
pub use molecule::Molecule;
pub use reaction::{ReactionRegistry, ReactionRule};

use ce_app::{App, Plugin};

/// Plugin that initialises the chemistry subsystem.
///
/// Inserts a [`PeriodicTable`] resource (all 118 elements) and a
/// [`ReactionRegistry`] pre-loaded with common reactions.
pub struct ChemistryPlugin;

impl Plugin for ChemistryPlugin {
    fn build(&self, app: &mut App) {
        let table = PeriodicTable::new();
        let registry = ReactionRegistry::default();
        let n_reactions = registry.rules.len();

        app.insert_resource(table);
        app.insert_resource(registry);

        log::info!(
            "ChemistryPlugin loaded — {} elements, {} reactions",
            118,
            n_reactions
        );
    }
}
