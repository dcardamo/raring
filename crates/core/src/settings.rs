//! Field-only preference updates. The serialized Settings type is the schema;
//! omission means unchanged, never a default or a cached client value.
use crate::{view::Settings, Error, Result};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct SettingsPatch(BTreeMap<String, Value>);

impl SettingsPatch {
    pub(crate) fn apply(self, current: &Settings) -> Result<Settings> {
        let mut merged = serde_json::to_value(current).map_err(|_| Error::InvalidSetting)?;
        let fields = merged.as_object_mut().ok_or(Error::InvalidSetting)?;
        for (name, value) in self.0 {
            let previous = fields.get(&name).ok_or(Error::InvalidSetting)?;
            // No null-as-omission, string booleans, or unknown fields. Settings'
            // deserializer below also enforces numeric bounds/integral types.
            let same_type = matches!((previous, &value),
                (Value::Bool(_), Value::Bool(_)) | (Value::Number(_), Value::Number(_))
                | (Value::String(_), Value::String(_)));
            if !same_type { return Err(Error::InvalidSetting); }
            fields.insert(name, value);
        }
        let settings: Settings = serde_json::from_value(merged).map_err(|_| Error::InvalidSetting)?;
        if settings.fall_asleep_min > 60 || !settings.fall_asleep_min.is_multiple_of(5) {
            return Err(Error::InvalidSetting);
        }
        Ok(settings)
    }
}
