use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, serde::Deserialize)]
pub struct PersonalName(String);

static FORBIDDEN_CHARS: [char; 10] = [';', '/', '(', ')', '"', '<', '>', '\\', '{', '}'];

impl PersonalName {
    pub fn parse(raw_name: String) -> Result<Self, String> {
        if raw_name.trim().is_empty() {
            return Err("name cannot be empty".into());
        }

        if raw_name.graphemes(true).count() > 256 {
            return Err("name cannot be longer than 256 characters".into());
        }

        if raw_name.chars().any(|char| FORBIDDEN_CHARS.contains(&char)) {
            return Err(format!(
                "name cannot contain the following special characters: {:?}",
                &FORBIDDEN_CHARS
            ));
        }

        Ok(Self(raw_name))
    }
}

impl AsRef<str> for PersonalName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};

    use super::*;

    #[test]
    fn a_blank_name_is_rejected() {
        let name = " ".repeat(64);
        assert_err!(PersonalName::parse(name));
    }

    #[test]
    fn a_256_graphemes_long_name_is_valid() {
        let name = "مُحَمَّدَُ".repeat(64);
        assert_ok!(PersonalName::parse(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "مُحَمَّدَُ".repeat(65);
        assert_err!(PersonalName::parse(name));
    }

    #[test]
    fn a_name_containing_forbidden_characters_is_rejected() {
        for &character in &FORBIDDEN_CHARS {
            let name = format!("Mohammad{}", character);
            assert_err!(PersonalName::parse(name));
        }
    }

    #[test]
    fn a_name_containing_no_forbidden_characters_is_valid() {
        let name = "Mohammad".to_string();
        assert_ok!(PersonalName::parse(name));
    }
}
