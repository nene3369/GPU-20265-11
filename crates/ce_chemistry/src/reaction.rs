/// A rule describing a chemical reaction.
#[derive(Debug, Clone)]
pub struct ReactionRule {
    /// Human-readable name for the reaction.
    pub name: String,
    /// Chemical formulae of the reactants.
    pub reactants: Vec<String>,
    /// Chemical formulae of the products.
    pub products: Vec<String>,
    /// Activation energy in kJ/mol.
    pub activation_energy: f64,
    /// Enthalpy change in kJ/mol (negative = exothermic).
    pub enthalpy_change: f64,
    /// Rate constant (k) for the reaction.
    pub rate_constant: f64,
}

/// Registry of known chemical reactions.
pub struct ReactionRegistry {
    pub rules: Vec<ReactionRule>,
}

impl ReactionRegistry {
    /// Creates an empty reaction registry.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Registers a new reaction rule.
    pub fn register(&mut self, rule: ReactionRule) {
        self.rules.push(rule);
    }

    /// Finds all reactions whose reactant lists match the given set of
    /// formula strings. A reaction matches if every one of the given
    /// reactants appears in the rule's reactant list.
    pub fn find_matching(&self, reactants: &[&str]) -> Vec<&ReactionRule> {
        self.rules
            .iter()
            .filter(|rule| {
                reactants
                    .iter()
                    .all(|r| rule.reactants.iter().any(|rr| rr == r))
            })
            .collect()
    }
}

impl Default for ReactionRegistry {
    /// Creates a registry pre-loaded with common chemical reactions.
    fn default() -> Self {
        let mut registry = Self::new();

        // 1. Water formation: 2H2 + O2 -> 2H2O
        registry.register(ReactionRule {
            name: "Water formation".to_string(),
            reactants: vec!["H2".to_string(), "O2".to_string()],
            products: vec!["H2O".to_string()],
            activation_energy: 75.0,
            enthalpy_change: -572.0,
            rate_constant: 1.0e6,
        });

        // 2. Methane combustion: CH4 + 2O2 -> CO2 + 2H2O
        registry.register(ReactionRule {
            name: "Methane combustion".to_string(),
            reactants: vec!["CH4".to_string(), "O2".to_string()],
            products: vec!["CO2".to_string(), "H2O".to_string()],
            activation_energy: 150.0,
            enthalpy_change: -890.4,
            rate_constant: 5.0e5,
        });

        // 3. Rust formation: 4Fe + 3O2 -> 2Fe2O3
        registry.register(ReactionRule {
            name: "Rust formation".to_string(),
            reactants: vec!["Fe".to_string(), "O2".to_string()],
            products: vec!["Fe2O3".to_string()],
            activation_energy: 50.0,
            enthalpy_change: -1648.0,
            rate_constant: 1.0e2,
        });

        // 4. Photosynthesis (simplified): 6CO2 + 6H2O -> C6H12O6 + 6O2
        registry.register(ReactionRule {
            name: "Photosynthesis".to_string(),
            reactants: vec!["CO2".to_string(), "H2O".to_string()],
            products: vec!["C6H12O6".to_string(), "O2".to_string()],
            activation_energy: 200.0,
            enthalpy_change: 2803.0,
            rate_constant: 1.0e3,
        });

        // 5. Acid-base neutralisation: HCl + NaOH -> NaCl + H2O
        registry.register(ReactionRule {
            name: "Acid-base neutralisation".to_string(),
            reactants: vec!["HCl".to_string(), "NaOH".to_string()],
            products: vec!["NaCl".to_string(), "H2O".to_string()],
            activation_energy: 20.0,
            enthalpy_change: -57.1,
            rate_constant: 1.0e8,
        });

        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_at_least_five_reactions() {
        let registry = ReactionRegistry::default();
        assert!(
            registry.rules.len() >= 5,
            "Default registry should have at least 5 reactions, got {}",
            registry.rules.len()
        );
    }

    #[test]
    fn find_matching_water_formation() {
        let registry = ReactionRegistry::default();
        let matches = registry.find_matching(&["H2", "O2"]);
        assert!(
            !matches.is_empty(),
            "Should find water formation for H2 + O2"
        );
        assert!(
            matches.iter().any(|r| r.name == "Water formation"),
            "Water formation should be among matched reactions"
        );
    }

    #[test]
    fn find_matching_unrelated_returns_empty() {
        let registry = ReactionRegistry::default();
        let matches = registry.find_matching(&["Xe", "Au"]);
        assert!(matches.is_empty(), "No reactions should match Xe + Au");
    }

    #[test]
    fn find_matching_methane_combustion() {
        let registry = ReactionRegistry::default();
        let matches = registry.find_matching(&["CH4", "O2"]);
        assert!(matches.iter().any(|r| r.name == "Methane combustion"));
    }

    #[test]
    fn register_custom_reaction() {
        let mut registry = ReactionRegistry::new();
        assert_eq!(registry.rules.len(), 0);

        registry.register(ReactionRule {
            name: "Test reaction".to_string(),
            reactants: vec!["A".to_string()],
            products: vec!["B".to_string()],
            activation_energy: 10.0,
            enthalpy_change: -5.0,
            rate_constant: 1.0,
        });

        assert_eq!(registry.rules.len(), 1);
        assert_eq!(registry.rules[0].name, "Test reaction");
    }

    #[test]
    fn find_matching_acid_base() {
        let registry = ReactionRegistry::default();
        let matches = registry.find_matching(&["HCl", "NaOH"]);
        assert!(matches.iter().any(|r| r.name == "Acid-base neutralisation"));
    }
}
