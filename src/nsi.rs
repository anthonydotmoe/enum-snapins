use serde::{Deserialize, Deserializer};
use std::str::FromStr;

#[derive(Debug, Default, PartialEq)]
pub struct IndirectString {
    pub dllpath: String,
    pub strid: i32,
}

impl<'de> Deserialize<'de> for IndirectString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        IndirectString::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for IndirectString {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dllpath: String;
        let strid: i32;

        if !s.starts_with("@") {
            return Err("String does not start with '@'".to_string());
        }

        let s = &s[1..]; // Remove leading '@'

        if let Some((left, right)) = s.split_once(",") {
            dllpath = left.to_string();
            strid = right.trim_start_matches('-').parse::<i32>().map_err(|_| "Invalid strId format")?;
        } else {
            return Err("Invalid format, expected ','".to_string());
        }

        Ok(IndirectString { dllpath, strid })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_namestringindirect_with_path() {
        let test_str = r"@C:\Windows\System32\example.dll,-1234";
        let name_string_indirect = IndirectString::from_str(test_str).unwrap();
        let expected = IndirectString {
            dllpath: r"C:\Windows\System32\example.dll".into(),
            strid: 1234,
        };

        assert_eq!(name_string_indirect, expected);
    }

    #[test]
    fn test_deserialize_namestringindirect_without_path() {
        let test_str = r"@example.dll,-1234";
        let name_string_indirect = IndirectString::from_str(test_str).unwrap();
        let expected = IndirectString {
            dllpath: r"example.dll".into(),
            strid: 1234,
        };

        assert_eq!(name_string_indirect, expected);
    }

    #[test]
    fn test_deserialize_namestringindirect_invalid_format() {
        let test_str = r"invalid_format";
        let result = IndirectString::from_str(test_str);

        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_namestringindirect_missing_strid() {
        let test_str = r"@C:\Windows\System32\example.dll";
        let result = IndirectString::from_str(test_str);

        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_namestringindirect_non_integer_strid() {
        let test_str = r"@C:\Windows\System32\example.dll,-abc";
        let result = IndirectString::from_str(test_str);

        assert!(result.is_err());
    }
}