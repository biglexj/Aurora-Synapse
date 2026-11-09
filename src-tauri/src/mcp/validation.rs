//! Valida el subconjunto de JSON Schema usado por el catálogo antes de despachar.
use serde_json::Value;

pub fn validate(value: &Value, schema: &Value) -> Result<(), String> {
    if let Some(kind) = schema["type"].as_str() {
        let valid = match kind {
            "object" => value.is_object(),
            "string" => value.is_string(),
            "boolean" => value.is_boolean(),
            "number" => value.is_number(),
            "integer" => value.as_u64().is_some() || value.as_i64().is_some(),
            _ => false,
        };
        if !valid {
            return Err(format!("se esperaba {kind}"));
        }
    }
    if let Some(allowed) = schema["enum"].as_array() {
        if !allowed.contains(value) {
            return Err(format!("valor no admitido: {value}"));
        }
    }
    if let Some(number) = value.as_f64() {
        if schema["minimum"].as_f64().is_some_and(|min| number < min)
            || schema["maximum"].as_f64().is_some_and(|max| number > max)
        {
            return Err("valor numérico fuera del rango admitido".into());
        }
    }
    if let Some(object) = value.as_object() {
        if let Some(required) = schema["required"].as_array() {
            for key in required.iter().filter_map(Value::as_str) {
                if !object.contains_key(key) {
                    return Err(format!("falta el parámetro '{key}'"));
                }
            }
        }
        for (key, field) in object {
            if let Some(field_schema) = schema["properties"].get(key) {
                validate(field, field_schema).map_err(|error| format!("{key}: {error}"))?;
            } else if schema["additionalProperties"] == false {
                return Err(format!("parámetro no admitido: {key}"));
            }
        }
    }
    Ok(())
}
