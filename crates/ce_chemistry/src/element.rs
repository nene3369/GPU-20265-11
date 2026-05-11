/// Unique identifier for an element (1-based atomic number).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ElementId(pub u8);

/// Physical and chemical properties of a single element.
#[derive(Debug, Clone)]
pub struct ElementProperties {
    pub atomic_number: u8,
    pub symbol: &'static str,
    pub name: &'static str,
    pub atomic_mass: f64,
    pub electronegativity: Option<f64>,
    pub phase_at_stp: Phase,
    pub group: u8,
    pub period: u8,
    pub category: ElementCategory,
}

/// Phase of matter at standard temperature and pressure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Solid,
    Liquid,
    Gas,
    Unknown,
}

/// Category within the periodic table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementCategory {
    AlkaliMetal,
    AlkalineEarthMetal,
    TransitionMetal,
    PostTransitionMetal,
    Metalloid,
    NonMetal,
    Halogen,
    NobleGas,
    Lanthanide,
    Actinide,
    Unknown,
}

/// The complete periodic table of elements.
pub struct PeriodicTable {
    elements: Vec<ElementProperties>,
}

impl PeriodicTable {
    /// Creates a new periodic table populated with all 118 elements.
    pub fn new() -> Self {
        let elements = vec![
            // ── Period 1 ──────────────────────────────────────────
            ElementProperties {
                atomic_number: 1,
                symbol: "H",
                name: "Hydrogen",
                atomic_mass: 1.008,
                electronegativity: Some(2.20),
                phase_at_stp: Phase::Gas,
                group: 1,
                period: 1,
                category: ElementCategory::NonMetal,
            },
            ElementProperties {
                atomic_number: 2,
                symbol: "He",
                name: "Helium",
                atomic_mass: 4.0026,
                electronegativity: None,
                phase_at_stp: Phase::Gas,
                group: 18,
                period: 1,
                category: ElementCategory::NobleGas,
            },
            // ── Period 2 ──────────────────────────────────────────
            ElementProperties {
                atomic_number: 3,
                symbol: "Li",
                name: "Lithium",
                atomic_mass: 6.941,
                electronegativity: Some(0.98),
                phase_at_stp: Phase::Solid,
                group: 1,
                period: 2,
                category: ElementCategory::AlkaliMetal,
            },
            ElementProperties {
                atomic_number: 4,
                symbol: "Be",
                name: "Beryllium",
                atomic_mass: 9.0122,
                electronegativity: Some(1.57),
                phase_at_stp: Phase::Solid,
                group: 2,
                period: 2,
                category: ElementCategory::AlkalineEarthMetal,
            },
            ElementProperties {
                atomic_number: 5,
                symbol: "B",
                name: "Boron",
                atomic_mass: 10.81,
                electronegativity: Some(2.04),
                phase_at_stp: Phase::Solid,
                group: 13,
                period: 2,
                category: ElementCategory::Metalloid,
            },
            ElementProperties {
                atomic_number: 6,
                symbol: "C",
                name: "Carbon",
                atomic_mass: 12.011,
                electronegativity: Some(2.55),
                phase_at_stp: Phase::Solid,
                group: 14,
                period: 2,
                category: ElementCategory::NonMetal,
            },
            ElementProperties {
                atomic_number: 7,
                symbol: "N",
                name: "Nitrogen",
                atomic_mass: 14.007,
                electronegativity: Some(3.04),
                phase_at_stp: Phase::Gas,
                group: 15,
                period: 2,
                category: ElementCategory::NonMetal,
            },
            ElementProperties {
                atomic_number: 8,
                symbol: "O",
                name: "Oxygen",
                atomic_mass: 15.999,
                electronegativity: Some(3.44),
                phase_at_stp: Phase::Gas,
                group: 16,
                period: 2,
                category: ElementCategory::NonMetal,
            },
            ElementProperties {
                atomic_number: 9,
                symbol: "F",
                name: "Fluorine",
                atomic_mass: 18.998,
                electronegativity: Some(3.98),
                phase_at_stp: Phase::Gas,
                group: 17,
                period: 2,
                category: ElementCategory::Halogen,
            },
            ElementProperties {
                atomic_number: 10,
                symbol: "Ne",
                name: "Neon",
                atomic_mass: 20.180,
                electronegativity: None,
                phase_at_stp: Phase::Gas,
                group: 18,
                period: 2,
                category: ElementCategory::NobleGas,
            },
            // ── Period 3 ──────────────────────────────────────────
            ElementProperties {
                atomic_number: 11,
                symbol: "Na",
                name: "Sodium",
                atomic_mass: 22.990,
                electronegativity: Some(0.93),
                phase_at_stp: Phase::Solid,
                group: 1,
                period: 3,
                category: ElementCategory::AlkaliMetal,
            },
            ElementProperties {
                atomic_number: 12,
                symbol: "Mg",
                name: "Magnesium",
                atomic_mass: 24.305,
                electronegativity: Some(1.31),
                phase_at_stp: Phase::Solid,
                group: 2,
                period: 3,
                category: ElementCategory::AlkalineEarthMetal,
            },
            ElementProperties {
                atomic_number: 13,
                symbol: "Al",
                name: "Aluminium",
                atomic_mass: 26.982,
                electronegativity: Some(1.61),
                phase_at_stp: Phase::Solid,
                group: 13,
                period: 3,
                category: ElementCategory::PostTransitionMetal,
            },
            ElementProperties {
                atomic_number: 14,
                symbol: "Si",
                name: "Silicon",
                atomic_mass: 28.086,
                electronegativity: Some(1.90),
                phase_at_stp: Phase::Solid,
                group: 14,
                period: 3,
                category: ElementCategory::Metalloid,
            },
            ElementProperties {
                atomic_number: 15,
                symbol: "P",
                name: "Phosphorus",
                atomic_mass: 30.974,
                electronegativity: Some(2.19),
                phase_at_stp: Phase::Solid,
                group: 15,
                period: 3,
                category: ElementCategory::NonMetal,
            },
            ElementProperties {
                atomic_number: 16,
                symbol: "S",
                name: "Sulfur",
                atomic_mass: 32.06,
                electronegativity: Some(2.58),
                phase_at_stp: Phase::Solid,
                group: 16,
                period: 3,
                category: ElementCategory::NonMetal,
            },
            ElementProperties {
                atomic_number: 17,
                symbol: "Cl",
                name: "Chlorine",
                atomic_mass: 35.45,
                electronegativity: Some(3.16),
                phase_at_stp: Phase::Gas,
                group: 17,
                period: 3,
                category: ElementCategory::Halogen,
            },
            ElementProperties {
                atomic_number: 18,
                symbol: "Ar",
                name: "Argon",
                atomic_mass: 39.948,
                electronegativity: None,
                phase_at_stp: Phase::Gas,
                group: 18,
                period: 3,
                category: ElementCategory::NobleGas,
            },
            // ── Period 4 ──────────────────────────────────────────
            ElementProperties {
                atomic_number: 19,
                symbol: "K",
                name: "Potassium",
                atomic_mass: 39.098,
                electronegativity: Some(0.82),
                phase_at_stp: Phase::Solid,
                group: 1,
                period: 4,
                category: ElementCategory::AlkaliMetal,
            },
            ElementProperties {
                atomic_number: 20,
                symbol: "Ca",
                name: "Calcium",
                atomic_mass: 40.078,
                electronegativity: Some(1.00),
                phase_at_stp: Phase::Solid,
                group: 2,
                period: 4,
                category: ElementCategory::AlkalineEarthMetal,
            },
            ElementProperties {
                atomic_number: 21,
                symbol: "Sc",
                name: "Scandium",
                atomic_mass: 44.956,
                electronegativity: Some(1.36),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 4,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 22,
                symbol: "Ti",
                name: "Titanium",
                atomic_mass: 47.867,
                electronegativity: Some(1.54),
                phase_at_stp: Phase::Solid,
                group: 4,
                period: 4,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 23,
                symbol: "V",
                name: "Vanadium",
                atomic_mass: 50.942,
                electronegativity: Some(1.63),
                phase_at_stp: Phase::Solid,
                group: 5,
                period: 4,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 24,
                symbol: "Cr",
                name: "Chromium",
                atomic_mass: 51.996,
                electronegativity: Some(1.66),
                phase_at_stp: Phase::Solid,
                group: 6,
                period: 4,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 25,
                symbol: "Mn",
                name: "Manganese",
                atomic_mass: 54.938,
                electronegativity: Some(1.55),
                phase_at_stp: Phase::Solid,
                group: 7,
                period: 4,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 26,
                symbol: "Fe",
                name: "Iron",
                atomic_mass: 55.845,
                electronegativity: Some(1.83),
                phase_at_stp: Phase::Solid,
                group: 8,
                period: 4,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 27,
                symbol: "Co",
                name: "Cobalt",
                atomic_mass: 58.933,
                electronegativity: Some(1.88),
                phase_at_stp: Phase::Solid,
                group: 9,
                period: 4,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 28,
                symbol: "Ni",
                name: "Nickel",
                atomic_mass: 58.693,
                electronegativity: Some(1.91),
                phase_at_stp: Phase::Solid,
                group: 10,
                period: 4,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 29,
                symbol: "Cu",
                name: "Copper",
                atomic_mass: 63.546,
                electronegativity: Some(1.90),
                phase_at_stp: Phase::Solid,
                group: 11,
                period: 4,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 30,
                symbol: "Zn",
                name: "Zinc",
                atomic_mass: 65.38,
                electronegativity: Some(1.65),
                phase_at_stp: Phase::Solid,
                group: 12,
                period: 4,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 31,
                symbol: "Ga",
                name: "Gallium",
                atomic_mass: 69.723,
                electronegativity: Some(1.81),
                phase_at_stp: Phase::Solid,
                group: 13,
                period: 4,
                category: ElementCategory::PostTransitionMetal,
            },
            ElementProperties {
                atomic_number: 32,
                symbol: "Ge",
                name: "Germanium",
                atomic_mass: 72.630,
                electronegativity: Some(2.01),
                phase_at_stp: Phase::Solid,
                group: 14,
                period: 4,
                category: ElementCategory::Metalloid,
            },
            ElementProperties {
                atomic_number: 33,
                symbol: "As",
                name: "Arsenic",
                atomic_mass: 74.922,
                electronegativity: Some(2.18),
                phase_at_stp: Phase::Solid,
                group: 15,
                period: 4,
                category: ElementCategory::Metalloid,
            },
            ElementProperties {
                atomic_number: 34,
                symbol: "Se",
                name: "Selenium",
                atomic_mass: 78.971,
                electronegativity: Some(2.55),
                phase_at_stp: Phase::Solid,
                group: 16,
                period: 4,
                category: ElementCategory::NonMetal,
            },
            ElementProperties {
                atomic_number: 35,
                symbol: "Br",
                name: "Bromine",
                atomic_mass: 79.904,
                electronegativity: Some(2.96),
                phase_at_stp: Phase::Liquid,
                group: 17,
                period: 4,
                category: ElementCategory::Halogen,
            },
            ElementProperties {
                atomic_number: 36,
                symbol: "Kr",
                name: "Krypton",
                atomic_mass: 83.798,
                electronegativity: Some(3.00),
                phase_at_stp: Phase::Gas,
                group: 18,
                period: 4,
                category: ElementCategory::NobleGas,
            },
            // ── Period 5 ──────────────────────────────────────────
            ElementProperties {
                atomic_number: 37,
                symbol: "Rb",
                name: "Rubidium",
                atomic_mass: 85.468,
                electronegativity: Some(0.82),
                phase_at_stp: Phase::Solid,
                group: 1,
                period: 5,
                category: ElementCategory::AlkaliMetal,
            },
            ElementProperties {
                atomic_number: 38,
                symbol: "Sr",
                name: "Strontium",
                atomic_mass: 87.62,
                electronegativity: Some(0.95),
                phase_at_stp: Phase::Solid,
                group: 2,
                period: 5,
                category: ElementCategory::AlkalineEarthMetal,
            },
            ElementProperties {
                atomic_number: 39,
                symbol: "Y",
                name: "Yttrium",
                atomic_mass: 88.906,
                electronegativity: Some(1.22),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 5,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 40,
                symbol: "Zr",
                name: "Zirconium",
                atomic_mass: 91.224,
                electronegativity: Some(1.33),
                phase_at_stp: Phase::Solid,
                group: 4,
                period: 5,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 41,
                symbol: "Nb",
                name: "Niobium",
                atomic_mass: 92.906,
                electronegativity: Some(1.6),
                phase_at_stp: Phase::Solid,
                group: 5,
                period: 5,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 42,
                symbol: "Mo",
                name: "Molybdenum",
                atomic_mass: 95.95,
                electronegativity: Some(2.16),
                phase_at_stp: Phase::Solid,
                group: 6,
                period: 5,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 43,
                symbol: "Tc",
                name: "Technetium",
                atomic_mass: 98.0,
                electronegativity: Some(1.9),
                phase_at_stp: Phase::Solid,
                group: 7,
                period: 5,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 44,
                symbol: "Ru",
                name: "Ruthenium",
                atomic_mass: 101.07,
                electronegativity: Some(2.2),
                phase_at_stp: Phase::Solid,
                group: 8,
                period: 5,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 45,
                symbol: "Rh",
                name: "Rhodium",
                atomic_mass: 102.91,
                electronegativity: Some(2.28),
                phase_at_stp: Phase::Solid,
                group: 9,
                period: 5,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 46,
                symbol: "Pd",
                name: "Palladium",
                atomic_mass: 106.42,
                electronegativity: Some(2.20),
                phase_at_stp: Phase::Solid,
                group: 10,
                period: 5,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 47,
                symbol: "Ag",
                name: "Silver",
                atomic_mass: 107.87,
                electronegativity: Some(1.93),
                phase_at_stp: Phase::Solid,
                group: 11,
                period: 5,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 48,
                symbol: "Cd",
                name: "Cadmium",
                atomic_mass: 112.41,
                electronegativity: Some(1.69),
                phase_at_stp: Phase::Solid,
                group: 12,
                period: 5,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 49,
                symbol: "In",
                name: "Indium",
                atomic_mass: 114.82,
                electronegativity: Some(1.78),
                phase_at_stp: Phase::Solid,
                group: 13,
                period: 5,
                category: ElementCategory::PostTransitionMetal,
            },
            ElementProperties {
                atomic_number: 50,
                symbol: "Sn",
                name: "Tin",
                atomic_mass: 118.71,
                electronegativity: Some(1.96),
                phase_at_stp: Phase::Solid,
                group: 14,
                period: 5,
                category: ElementCategory::PostTransitionMetal,
            },
            ElementProperties {
                atomic_number: 51,
                symbol: "Sb",
                name: "Antimony",
                atomic_mass: 121.76,
                electronegativity: Some(2.05),
                phase_at_stp: Phase::Solid,
                group: 15,
                period: 5,
                category: ElementCategory::Metalloid,
            },
            ElementProperties {
                atomic_number: 52,
                symbol: "Te",
                name: "Tellurium",
                atomic_mass: 127.60,
                electronegativity: Some(2.1),
                phase_at_stp: Phase::Solid,
                group: 16,
                period: 5,
                category: ElementCategory::Metalloid,
            },
            ElementProperties {
                atomic_number: 53,
                symbol: "I",
                name: "Iodine",
                atomic_mass: 126.90,
                electronegativity: Some(2.66),
                phase_at_stp: Phase::Solid,
                group: 17,
                period: 5,
                category: ElementCategory::Halogen,
            },
            ElementProperties {
                atomic_number: 54,
                symbol: "Xe",
                name: "Xenon",
                atomic_mass: 131.29,
                electronegativity: Some(2.60),
                phase_at_stp: Phase::Gas,
                group: 18,
                period: 5,
                category: ElementCategory::NobleGas,
            },
            // ── Period 6 ──────────────────────────────────────────
            ElementProperties {
                atomic_number: 55,
                symbol: "Cs",
                name: "Caesium",
                atomic_mass: 132.91,
                electronegativity: Some(0.79),
                phase_at_stp: Phase::Solid,
                group: 1,
                period: 6,
                category: ElementCategory::AlkaliMetal,
            },
            ElementProperties {
                atomic_number: 56,
                symbol: "Ba",
                name: "Barium",
                atomic_mass: 137.33,
                electronegativity: Some(0.89),
                phase_at_stp: Phase::Solid,
                group: 2,
                period: 6,
                category: ElementCategory::AlkalineEarthMetal,
            },
            // ── Lanthanides (57-71) ───────────────────────────────
            ElementProperties {
                atomic_number: 57,
                symbol: "La",
                name: "Lanthanum",
                atomic_mass: 138.91,
                electronegativity: Some(1.1),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 6,
                category: ElementCategory::Lanthanide,
            },
            ElementProperties {
                atomic_number: 58,
                symbol: "Ce",
                name: "Cerium",
                atomic_mass: 140.12,
                electronegativity: Some(1.12),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 6,
                category: ElementCategory::Lanthanide,
            },
            ElementProperties {
                atomic_number: 59,
                symbol: "Pr",
                name: "Praseodymium",
                atomic_mass: 140.91,
                electronegativity: Some(1.13),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 6,
                category: ElementCategory::Lanthanide,
            },
            ElementProperties {
                atomic_number: 60,
                symbol: "Nd",
                name: "Neodymium",
                atomic_mass: 144.24,
                electronegativity: Some(1.14),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 6,
                category: ElementCategory::Lanthanide,
            },
            ElementProperties {
                atomic_number: 61,
                symbol: "Pm",
                name: "Promethium",
                atomic_mass: 145.0,
                electronegativity: Some(1.13),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 6,
                category: ElementCategory::Lanthanide,
            },
            ElementProperties {
                atomic_number: 62,
                symbol: "Sm",
                name: "Samarium",
                atomic_mass: 150.36,
                electronegativity: Some(1.17),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 6,
                category: ElementCategory::Lanthanide,
            },
            ElementProperties {
                atomic_number: 63,
                symbol: "Eu",
                name: "Europium",
                atomic_mass: 151.96,
                electronegativity: Some(1.2),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 6,
                category: ElementCategory::Lanthanide,
            },
            ElementProperties {
                atomic_number: 64,
                symbol: "Gd",
                name: "Gadolinium",
                atomic_mass: 157.25,
                electronegativity: Some(1.2),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 6,
                category: ElementCategory::Lanthanide,
            },
            ElementProperties {
                atomic_number: 65,
                symbol: "Tb",
                name: "Terbium",
                atomic_mass: 158.93,
                electronegativity: Some(1.1),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 6,
                category: ElementCategory::Lanthanide,
            },
            ElementProperties {
                atomic_number: 66,
                symbol: "Dy",
                name: "Dysprosium",
                atomic_mass: 162.50,
                electronegativity: Some(1.22),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 6,
                category: ElementCategory::Lanthanide,
            },
            ElementProperties {
                atomic_number: 67,
                symbol: "Ho",
                name: "Holmium",
                atomic_mass: 164.93,
                electronegativity: Some(1.23),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 6,
                category: ElementCategory::Lanthanide,
            },
            ElementProperties {
                atomic_number: 68,
                symbol: "Er",
                name: "Erbium",
                atomic_mass: 167.26,
                electronegativity: Some(1.24),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 6,
                category: ElementCategory::Lanthanide,
            },
            ElementProperties {
                atomic_number: 69,
                symbol: "Tm",
                name: "Thulium",
                atomic_mass: 168.93,
                electronegativity: Some(1.25),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 6,
                category: ElementCategory::Lanthanide,
            },
            ElementProperties {
                atomic_number: 70,
                symbol: "Yb",
                name: "Ytterbium",
                atomic_mass: 173.05,
                electronegativity: Some(1.1),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 6,
                category: ElementCategory::Lanthanide,
            },
            ElementProperties {
                atomic_number: 71,
                symbol: "Lu",
                name: "Lutetium",
                atomic_mass: 174.97,
                electronegativity: Some(1.27),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 6,
                category: ElementCategory::Lanthanide,
            },
            // ── Period 6 continued (72-86) ────────────────────────
            ElementProperties {
                atomic_number: 72,
                symbol: "Hf",
                name: "Hafnium",
                atomic_mass: 178.49,
                electronegativity: Some(1.3),
                phase_at_stp: Phase::Solid,
                group: 4,
                period: 6,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 73,
                symbol: "Ta",
                name: "Tantalum",
                atomic_mass: 180.95,
                electronegativity: Some(1.5),
                phase_at_stp: Phase::Solid,
                group: 5,
                period: 6,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 74,
                symbol: "W",
                name: "Tungsten",
                atomic_mass: 183.84,
                electronegativity: Some(2.36),
                phase_at_stp: Phase::Solid,
                group: 6,
                period: 6,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 75,
                symbol: "Re",
                name: "Rhenium",
                atomic_mass: 186.21,
                electronegativity: Some(1.9),
                phase_at_stp: Phase::Solid,
                group: 7,
                period: 6,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 76,
                symbol: "Os",
                name: "Osmium",
                atomic_mass: 190.23,
                electronegativity: Some(2.2),
                phase_at_stp: Phase::Solid,
                group: 8,
                period: 6,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 77,
                symbol: "Ir",
                name: "Iridium",
                atomic_mass: 192.22,
                electronegativity: Some(2.20),
                phase_at_stp: Phase::Solid,
                group: 9,
                period: 6,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 78,
                symbol: "Pt",
                name: "Platinum",
                atomic_mass: 195.08,
                electronegativity: Some(2.28),
                phase_at_stp: Phase::Solid,
                group: 10,
                period: 6,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 79,
                symbol: "Au",
                name: "Gold",
                atomic_mass: 196.97,
                electronegativity: Some(2.54),
                phase_at_stp: Phase::Solid,
                group: 11,
                period: 6,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 80,
                symbol: "Hg",
                name: "Mercury",
                atomic_mass: 200.59,
                electronegativity: Some(2.00),
                phase_at_stp: Phase::Liquid,
                group: 12,
                period: 6,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 81,
                symbol: "Tl",
                name: "Thallium",
                atomic_mass: 204.38,
                electronegativity: Some(1.62),
                phase_at_stp: Phase::Solid,
                group: 13,
                period: 6,
                category: ElementCategory::PostTransitionMetal,
            },
            ElementProperties {
                atomic_number: 82,
                symbol: "Pb",
                name: "Lead",
                atomic_mass: 207.2,
                electronegativity: Some(1.87),
                phase_at_stp: Phase::Solid,
                group: 14,
                period: 6,
                category: ElementCategory::PostTransitionMetal,
            },
            ElementProperties {
                atomic_number: 83,
                symbol: "Bi",
                name: "Bismuth",
                atomic_mass: 208.98,
                electronegativity: Some(2.02),
                phase_at_stp: Phase::Solid,
                group: 15,
                period: 6,
                category: ElementCategory::PostTransitionMetal,
            },
            ElementProperties {
                atomic_number: 84,
                symbol: "Po",
                name: "Polonium",
                atomic_mass: 209.0,
                electronegativity: Some(2.0),
                phase_at_stp: Phase::Solid,
                group: 16,
                period: 6,
                category: ElementCategory::PostTransitionMetal,
            },
            ElementProperties {
                atomic_number: 85,
                symbol: "At",
                name: "Astatine",
                atomic_mass: 210.0,
                electronegativity: Some(2.2),
                phase_at_stp: Phase::Solid,
                group: 17,
                period: 6,
                category: ElementCategory::Halogen,
            },
            ElementProperties {
                atomic_number: 86,
                symbol: "Rn",
                name: "Radon",
                atomic_mass: 222.0,
                electronegativity: Some(2.2),
                phase_at_stp: Phase::Gas,
                group: 18,
                period: 6,
                category: ElementCategory::NobleGas,
            },
            // ── Period 7 ──────────────────────────────────────────
            ElementProperties {
                atomic_number: 87,
                symbol: "Fr",
                name: "Francium",
                atomic_mass: 223.0,
                electronegativity: Some(0.7),
                phase_at_stp: Phase::Solid,
                group: 1,
                period: 7,
                category: ElementCategory::AlkaliMetal,
            },
            ElementProperties {
                atomic_number: 88,
                symbol: "Ra",
                name: "Radium",
                atomic_mass: 226.0,
                electronegativity: Some(0.9),
                phase_at_stp: Phase::Solid,
                group: 2,
                period: 7,
                category: ElementCategory::AlkalineEarthMetal,
            },
            // ── Actinides (89-103) ────────────────────────────────
            ElementProperties {
                atomic_number: 89,
                symbol: "Ac",
                name: "Actinium",
                atomic_mass: 227.0,
                electronegativity: Some(1.1),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 7,
                category: ElementCategory::Actinide,
            },
            ElementProperties {
                atomic_number: 90,
                symbol: "Th",
                name: "Thorium",
                atomic_mass: 232.04,
                electronegativity: Some(1.3),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 7,
                category: ElementCategory::Actinide,
            },
            ElementProperties {
                atomic_number: 91,
                symbol: "Pa",
                name: "Protactinium",
                atomic_mass: 231.04,
                electronegativity: Some(1.5),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 7,
                category: ElementCategory::Actinide,
            },
            ElementProperties {
                atomic_number: 92,
                symbol: "U",
                name: "Uranium",
                atomic_mass: 238.03,
                electronegativity: Some(1.38),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 7,
                category: ElementCategory::Actinide,
            },
            ElementProperties {
                atomic_number: 93,
                symbol: "Np",
                name: "Neptunium",
                atomic_mass: 237.0,
                electronegativity: Some(1.36),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 7,
                category: ElementCategory::Actinide,
            },
            ElementProperties {
                atomic_number: 94,
                symbol: "Pu",
                name: "Plutonium",
                atomic_mass: 244.0,
                electronegativity: Some(1.28),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 7,
                category: ElementCategory::Actinide,
            },
            ElementProperties {
                atomic_number: 95,
                symbol: "Am",
                name: "Americium",
                atomic_mass: 243.0,
                electronegativity: Some(1.3),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 7,
                category: ElementCategory::Actinide,
            },
            ElementProperties {
                atomic_number: 96,
                symbol: "Cm",
                name: "Curium",
                atomic_mass: 247.0,
                electronegativity: Some(1.3),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 7,
                category: ElementCategory::Actinide,
            },
            ElementProperties {
                atomic_number: 97,
                symbol: "Bk",
                name: "Berkelium",
                atomic_mass: 247.0,
                electronegativity: Some(1.3),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 7,
                category: ElementCategory::Actinide,
            },
            ElementProperties {
                atomic_number: 98,
                symbol: "Cf",
                name: "Californium",
                atomic_mass: 251.0,
                electronegativity: Some(1.3),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 7,
                category: ElementCategory::Actinide,
            },
            ElementProperties {
                atomic_number: 99,
                symbol: "Es",
                name: "Einsteinium",
                atomic_mass: 252.0,
                electronegativity: Some(1.3),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 7,
                category: ElementCategory::Actinide,
            },
            ElementProperties {
                atomic_number: 100,
                symbol: "Fm",
                name: "Fermium",
                atomic_mass: 257.0,
                electronegativity: Some(1.3),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 7,
                category: ElementCategory::Actinide,
            },
            ElementProperties {
                atomic_number: 101,
                symbol: "Md",
                name: "Mendelevium",
                atomic_mass: 258.0,
                electronegativity: Some(1.3),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 7,
                category: ElementCategory::Actinide,
            },
            ElementProperties {
                atomic_number: 102,
                symbol: "No",
                name: "Nobelium",
                atomic_mass: 259.0,
                electronegativity: Some(1.3),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 7,
                category: ElementCategory::Actinide,
            },
            ElementProperties {
                atomic_number: 103,
                symbol: "Lr",
                name: "Lawrencium",
                atomic_mass: 266.0,
                electronegativity: Some(1.3),
                phase_at_stp: Phase::Solid,
                group: 3,
                period: 7,
                category: ElementCategory::Actinide,
            },
            // ── Period 7 continued (104-118) ──────────────────────
            ElementProperties {
                atomic_number: 104,
                symbol: "Rf",
                name: "Rutherfordium",
                atomic_mass: 267.0,
                electronegativity: None,
                phase_at_stp: Phase::Unknown,
                group: 4,
                period: 7,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 105,
                symbol: "Db",
                name: "Dubnium",
                atomic_mass: 268.0,
                electronegativity: None,
                phase_at_stp: Phase::Unknown,
                group: 5,
                period: 7,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 106,
                symbol: "Sg",
                name: "Seaborgium",
                atomic_mass: 269.0,
                electronegativity: None,
                phase_at_stp: Phase::Unknown,
                group: 6,
                period: 7,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 107,
                symbol: "Bh",
                name: "Bohrium",
                atomic_mass: 270.0,
                electronegativity: None,
                phase_at_stp: Phase::Unknown,
                group: 7,
                period: 7,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 108,
                symbol: "Hs",
                name: "Hassium",
                atomic_mass: 277.0,
                electronegativity: None,
                phase_at_stp: Phase::Unknown,
                group: 8,
                period: 7,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 109,
                symbol: "Mt",
                name: "Meitnerium",
                atomic_mass: 278.0,
                electronegativity: None,
                phase_at_stp: Phase::Unknown,
                group: 9,
                period: 7,
                category: ElementCategory::Unknown,
            },
            ElementProperties {
                atomic_number: 110,
                symbol: "Ds",
                name: "Darmstadtium",
                atomic_mass: 281.0,
                electronegativity: None,
                phase_at_stp: Phase::Unknown,
                group: 10,
                period: 7,
                category: ElementCategory::Unknown,
            },
            ElementProperties {
                atomic_number: 111,
                symbol: "Rg",
                name: "Roentgenium",
                atomic_mass: 282.0,
                electronegativity: None,
                phase_at_stp: Phase::Unknown,
                group: 11,
                period: 7,
                category: ElementCategory::Unknown,
            },
            ElementProperties {
                atomic_number: 112,
                symbol: "Cn",
                name: "Copernicium",
                atomic_mass: 285.0,
                electronegativity: None,
                phase_at_stp: Phase::Unknown,
                group: 12,
                period: 7,
                category: ElementCategory::TransitionMetal,
            },
            ElementProperties {
                atomic_number: 113,
                symbol: "Nh",
                name: "Nihonium",
                atomic_mass: 286.0,
                electronegativity: None,
                phase_at_stp: Phase::Unknown,
                group: 13,
                period: 7,
                category: ElementCategory::Unknown,
            },
            ElementProperties {
                atomic_number: 114,
                symbol: "Fl",
                name: "Flerovium",
                atomic_mass: 289.0,
                electronegativity: None,
                phase_at_stp: Phase::Unknown,
                group: 14,
                period: 7,
                category: ElementCategory::Unknown,
            },
            ElementProperties {
                atomic_number: 115,
                symbol: "Mc",
                name: "Moscovium",
                atomic_mass: 290.0,
                electronegativity: None,
                phase_at_stp: Phase::Unknown,
                group: 15,
                period: 7,
                category: ElementCategory::Unknown,
            },
            ElementProperties {
                atomic_number: 116,
                symbol: "Lv",
                name: "Livermorium",
                atomic_mass: 293.0,
                electronegativity: None,
                phase_at_stp: Phase::Unknown,
                group: 16,
                period: 7,
                category: ElementCategory::Unknown,
            },
            ElementProperties {
                atomic_number: 117,
                symbol: "Ts",
                name: "Tennessine",
                atomic_mass: 294.0,
                electronegativity: None,
                phase_at_stp: Phase::Unknown,
                group: 17,
                period: 7,
                category: ElementCategory::Unknown,
            },
            ElementProperties {
                atomic_number: 118,
                symbol: "Og",
                name: "Oganesson",
                atomic_mass: 294.0,
                electronegativity: None,
                phase_at_stp: Phase::Unknown,
                group: 18,
                period: 7,
                category: ElementCategory::NobleGas,
            },
        ];

        PeriodicTable { elements }
    }

    /// Look up an element by its [`ElementId`] (1-based atomic number).
    pub fn get(&self, id: ElementId) -> Option<&ElementProperties> {
        if id.0 == 0 {
            return None;
        }
        self.elements.get((id.0 as usize).wrapping_sub(1))
    }

    /// Look up an element by its chemical symbol (case-sensitive).
    pub fn by_symbol(&self, symbol: &str) -> Option<&ElementProperties> {
        self.elements.iter().find(|e| e.symbol == symbol)
    }

    /// Returns the number of elements in the table.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Returns `true` if the table is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

impl Default for PeriodicTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn periodic_table_has_118_elements() {
        let table = PeriodicTable::new();
        assert_eq!(table.len(), 118);
    }

    #[test]
    fn hydrogen_is_element_1() {
        let table = PeriodicTable::new();
        let h = table.get(ElementId(1)).expect("Element 1 should exist");
        assert_eq!(h.symbol, "H");
        assert_eq!(h.name, "Hydrogen");
        assert!((h.atomic_mass - 1.008).abs() < 0.01);
        assert_eq!(h.phase_at_stp, Phase::Gas);
        assert_eq!(h.category, ElementCategory::NonMetal);
    }

    #[test]
    fn carbon_is_element_6() {
        let table = PeriodicTable::new();
        let c = table.get(ElementId(6)).expect("Element 6 should exist");
        assert_eq!(c.symbol, "C");
        assert_eq!(c.name, "Carbon");
        assert!((c.atomic_mass - 12.011).abs() < 0.01);
    }

    #[test]
    fn iron_is_element_26() {
        let table = PeriodicTable::new();
        let fe = table.get(ElementId(26)).expect("Element 26 should exist");
        assert_eq!(fe.symbol, "Fe");
        assert_eq!(fe.name, "Iron");
        assert_eq!(fe.category, ElementCategory::TransitionMetal);
    }

    #[test]
    fn gold_is_element_79() {
        let table = PeriodicTable::new();
        let au = table.get(ElementId(79)).expect("Element 79 should exist");
        assert_eq!(au.symbol, "Au");
        assert_eq!(au.name, "Gold");
        assert!((au.atomic_mass - 196.97).abs() < 0.1);
    }

    #[test]
    fn silver_is_element_47() {
        let table = PeriodicTable::new();
        let ag = table.get(ElementId(47)).expect("Element 47 should exist");
        assert_eq!(ag.symbol, "Ag");
        assert_eq!(ag.name, "Silver");
    }

    #[test]
    fn platinum_is_element_78() {
        let table = PeriodicTable::new();
        let pt = table.get(ElementId(78)).expect("Element 78 should exist");
        assert_eq!(pt.symbol, "Pt");
        assert_eq!(pt.name, "Platinum");
    }

    #[test]
    fn uranium_is_element_92() {
        let table = PeriodicTable::new();
        let u = table.get(ElementId(92)).expect("Element 92 should exist");
        assert_eq!(u.symbol, "U");
        assert_eq!(u.name, "Uranium");
        assert_eq!(u.category, ElementCategory::Actinide);
    }

    #[test]
    fn plutonium_is_element_94() {
        let table = PeriodicTable::new();
        let pu = table.get(ElementId(94)).expect("Element 94 should exist");
        assert_eq!(pu.symbol, "Pu");
        assert_eq!(pu.name, "Plutonium");
        assert_eq!(pu.category, ElementCategory::Actinide);
    }

    #[test]
    fn copper_is_element_29() {
        let table = PeriodicTable::new();
        let cu = table.get(ElementId(29)).expect("Element 29 should exist");
        assert_eq!(cu.symbol, "Cu");
        assert_eq!(cu.name, "Copper");
    }

    #[test]
    fn by_symbol_oxygen() {
        let table = PeriodicTable::new();
        let o = table.by_symbol("O").expect("Oxygen should exist");
        assert_eq!(o.name, "Oxygen");
        assert_eq!(o.atomic_number, 8);
    }

    #[test]
    fn by_symbol_unknown_returns_none() {
        let table = PeriodicTable::new();
        assert!(table.by_symbol("XX").is_none());
    }

    #[test]
    fn element_id_zero_returns_none() {
        let table = PeriodicTable::new();
        assert!(table.get(ElementId(0)).is_none());
    }

    #[test]
    fn element_id_out_of_range_returns_none() {
        let table = PeriodicTable::new();
        assert!(table.get(ElementId(119)).is_none());
    }

    #[test]
    fn oganesson_is_element_118() {
        let table = PeriodicTable::new();
        let og = table.get(ElementId(118)).expect("Element 118 should exist");
        assert_eq!(og.symbol, "Og");
        assert_eq!(og.name, "Oganesson");
    }
}
