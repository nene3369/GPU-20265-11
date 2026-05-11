use ce_core::Entity;

/// A molecule composed of atom and bond entities.
#[derive(Debug, Clone)]
pub struct Molecule {
    /// Entity handles of the atoms in this molecule.
    pub atoms: Vec<Entity>,
    /// Entity handles of the bonds in this molecule.
    pub bonds: Vec<Entity>,
    /// Chemical formula string (e.g. "H2O", "CH4").
    pub formula: String,
    /// Optional human-readable name.
    pub name: Option<String>,
}

impl Molecule {
    /// Creates a new molecule with the given formula and empty atom/bond lists.
    pub fn new(formula: &str) -> Self {
        Self {
            atoms: Vec::new(),
            bonds: Vec::new(),
            formula: formula.to_string(),
            name: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_molecule_has_formula() {
        let mol = Molecule::new("H2O");
        assert_eq!(mol.formula, "H2O");
        assert!(mol.atoms.is_empty());
        assert!(mol.bonds.is_empty());
        assert!(mol.name.is_none());
    }

    #[test]
    fn molecule_with_name() {
        let mut mol = Molecule::new("C6H12O6");
        mol.name = Some("Glucose".to_string());
        assert_eq!(mol.name.as_deref(), Some("Glucose"));
    }
}
