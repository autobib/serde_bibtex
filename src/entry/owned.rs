use serde::de::{Deserializer, MapAccess, Visitor};
use serde::Deserialize;
use std::fmt;
use unicase::UniCase;

use std::collections::BTreeMap;

#[derive(Deserialize, Debug, PartialEq)]
pub enum Entry {
    Regular {
        entry_type: String,
        #[serde(deserialize_with = "deserialize_unicase")]
        entry_key: UniCase<String>,
        fields: Fields,
    },
    Macro,
    Comment,
    Preamble,
}

#[derive(Debug, PartialEq)]
pub struct Fields(pub BTreeMap<UniCase<String>, String>);

struct FieldsVisitor;

impl<'de> Visitor<'de> for FieldsVisitor {
    type Value = Fields;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("fields map")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = BTreeMap::default();

        while let Some((key, value)) = access.next_entry()? {
            map.insert(UniCase::new(key), value);
        }

        Ok(Fields(map))
    }
}

impl<'de> Deserialize<'de> for Fields {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(FieldsVisitor)
    }
}

#[inline]
fn deserialize_unicase<'de, D>(deserializer: D) -> Result<UniCase<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(UniCase::new(String::deserialize(deserializer)?))
}
