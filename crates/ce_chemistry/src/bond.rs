use ce_core::Entity;

/// The type of chemical bond between two atoms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BondType {
    Single,
    Double,
    Triple,
    Aromatic,
    Ionic,
    Hydrogen,
    VanDerWaals,
}

/// A chemical bond linking two atom entities.
#[derive(Debug, Clone, Copy)]
pub struct Bond {
    /// First atom in the bond.
    pub atom_a: Entity,
    /// Second atom in the bond.
    pub atom_b: Entity,
    /// Classification of the bond.
    pub bond_type: BondType,
    /// Bond order (1.0 for single, 2.0 for double, etc.).
    pub bond_order: f64,
    /// Equilibrium bond length in angstroms.
    pub equilibrium_length: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bond_type_equality() {
        assert_eq!(BondType::Single, BondType::Single);
        assert_ne!(BondType::Single, BondType::Double);
    }

    #[test]
    fn bond_creation() {
        let a = Entity {
            index: 0,
            generation: 0,
        };
        let b = Entity {
            index: 1,
            generation: 0,
        };
        let bond = Bond {
            atom_a: a,
            atom_b: b,
            bond_type: BondType::Double,
            bond_order: 2.0,
            equilibrium_length: 1.21,
        };
        assert_eq!(bond.bond_type, BondType::Double);
        assert!((bond.bond_order - 2.0).abs() < f64::EPSILON);
    }
}
