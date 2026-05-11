use crate::element::ElementId;

/// An atom component representing a single atom in the ECS world.
#[derive(Debug, Clone, Copy)]
pub struct Atom {
    /// Which element this atom is.
    pub element: ElementId,
    /// Ionic charge (0 for neutral atoms).
    pub charge: i8,
    /// Optional override for the atomic mass (e.g. for isotopes).
    pub mass_override: Option<f64>,
}

impl Atom {
    /// Creates a neutral atom of the given element.
    pub fn new(element: ElementId) -> Self {
        Self {
            element,
            charge: 0,
            mass_override: None,
        }
    }

    /// Creates an ion of the given element with the specified charge.
    pub fn ion(element: ElementId, charge: i8) -> Self {
        Self {
            element,
            charge,
            mass_override: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_atom_is_neutral() {
        let atom = Atom::new(ElementId(8));
        assert_eq!(atom.element, ElementId(8));
        assert_eq!(atom.charge, 0);
        assert!(atom.mass_override.is_none());
    }

    #[test]
    fn ion_has_charge() {
        let ion = Atom::ion(ElementId(11), 1);
        assert_eq!(ion.element, ElementId(11));
        assert_eq!(ion.charge, 1);
    }

    #[test]
    fn negative_ion() {
        let ion = Atom::ion(ElementId(17), -1);
        assert_eq!(ion.charge, -1);
    }
}
