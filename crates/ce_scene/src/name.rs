/// Human-readable name for an entity (debugging, editor, logging).
#[derive(Debug, Clone)]
pub struct Name(pub String);

impl Name {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_display() {
        let name = Name::new("Player");
        assert_eq!(format!("{}", name), "Player");
    }

    #[test]
    fn name_as_str() {
        let name = Name::new("Camera");
        assert_eq!(name.as_str(), "Camera");
    }

    #[test]
    fn name_clone() {
        let name = Name::new("Entity");
        let cloned = name.clone();
        assert_eq!(cloned.as_str(), "Entity");
    }

    #[test]
    fn name_debug() {
        let name = Name::new("Debug");
        let debug_str = format!("{:?}", name);
        assert!(debug_str.contains("Debug"));
    }
}
