/// Feature discovery and management for cameras.
///
/// Provides a dynamic registry of camera features (properties) that can be queried,
/// read, and written at runtime. This is essential for supporting multiple camera
/// models that have different feature sets (Basler, IDS, RPi, etc.).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{OptikError, Result};

/// Possible data types for camera features.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FeatureValue {
    /// 64-bit signed integer
    Integer(i64),
    /// 64-bit floating point
    Float(f64),
    /// Boolean flag
    Boolean(bool),
    /// String value
    String(String),
    /// Enumeration with discrete values
    Enum(String),
}

impl FeatureValue {
    /// Convert to f64 (useful for exposure, gain)
    pub fn as_f64(&self) -> Result<f64> {
        match self {
            FeatureValue::Integer(i) => Ok(*i as f64),
            FeatureValue::Float(f) => Ok(*f),
            _ => Err(OptikError::ConfigError(
                "Cannot convert feature to f64".to_string(),
            )),
        }
    }

    /// Convert to i64
    pub fn as_i64(&self) -> Result<i64> {
        match self {
            FeatureValue::Integer(i) => Ok(*i),
            FeatureValue::Float(f) => Ok(*f as i64),
            _ => Err(OptikError::ConfigError(
                "Cannot convert feature to i64".to_string(),
            )),
        }
    }

    /// Convert to bool
    pub fn as_bool(&self) -> Result<bool> {
        match self {
            FeatureValue::Boolean(b) => Ok(*b),
            _ => Err(OptikError::ConfigError(
                "Cannot convert feature to bool".to_string(),
            )),
        }
    }

    /// Convert to String
    pub fn as_string(&self) -> Result<String> {
        match self {
            FeatureValue::String(s) => Ok(s.clone()),
            FeatureValue::Enum(e) => Ok(e.clone()),
            FeatureValue::Integer(i) => Ok(i.to_string()),
            FeatureValue::Float(f) => Ok(f.to_string()),
            FeatureValue::Boolean(b) => Ok(b.to_string()),
        }
    }
}

impl std::fmt::Display for FeatureValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeatureValue::Integer(i) => write!(f, "{}", i),
            FeatureValue::Float(fl) => write!(f, "{}", fl),
            FeatureValue::Boolean(b) => write!(f, "{}", b),
            FeatureValue::String(s) => write!(f, "{}", s),
            FeatureValue::Enum(e) => write!(f, "{}", e),
        }
    }
}

/// Constraints for a feature value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConstraints {
    /// Minimum value (for numeric types)
    pub min: Option<f64>,
    /// Maximum value (for numeric types)
    pub max: Option<f64>,
    /// Increment step (for numeric types)
    pub step: Option<f64>,
    /// Possible enum values
    pub enum_values: Option<Vec<String>>,
}

impl FeatureConstraints {
    /// Validate a value against constraints.
    pub fn validate(&self, value: &FeatureValue) -> Result<()> {
        match value {
            FeatureValue::Integer(i) => {
                let f = *i as f64;
                if let Some(min) = self.min {
                    if f < min {
                        return Err(OptikError::ConfigError(format!(
                            "Value {} is below minimum {}",
                            i, min
                        )));
                    }
                }
                if let Some(max) = self.max {
                    if f > max {
                        return Err(OptikError::ConfigError(format!(
                            "Value {} is above maximum {}",
                            i, max
                        )));
                    }
                }
            }
            FeatureValue::Float(f) => {
                if let Some(min) = self.min {
                    if f < &min {
                        return Err(OptikError::ConfigError(format!(
                            "Value {} is below minimum {}",
                            f, min
                        )));
                    }
                }
                if let Some(max) = self.max {
                    if f > &max {
                        return Err(OptikError::ConfigError(format!(
                            "Value {} is above maximum {}",
                            f, max
                        )));
                    }
                }
            }
            FeatureValue::Enum(e) => {
                if let Some(ref values) = self.enum_values {
                    if !values.contains(e) {
                        return Err(OptikError::ConfigError(format!(
                            "Value '{}' not in allowed enum values: {:?}",
                            e, values
                        )));
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// Information about a single feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDescriptor {
    /// Feature name (e.g., "ExposureTime", "Gain")
    pub name: String,
    /// Human-readable description
    pub description: Option<String>,
    /// Current value
    pub current_value: FeatureValue,
    /// Whether the feature can be read
    pub readable: bool,
    /// Whether the feature can be written
    pub writable: bool,
    /// Constraints and allowed values
    pub constraints: FeatureConstraints,
}

/// Registry of all available features on a camera.
#[derive(Debug, Clone)]
pub struct FeatureRegistry {
    features: Arc<parking_lot::RwLock<HashMap<String, FeatureDescriptor>>>,
}

impl FeatureRegistry {
    /// Create a new empty feature registry.
    pub fn new() -> Self {
        Self {
            features: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }

    /// Register a new feature.
    pub fn register(&self, descriptor: FeatureDescriptor) {
        let mut features = self.features.write();
        features.insert(descriptor.name.clone(), descriptor);
    }

    /// Get a feature descriptor by name.
    pub fn get(&self, name: &str) -> Result<FeatureDescriptor> {
        let features = self.features.read();
        features.get(name).cloned().ok_or_else(|| {
            OptikError::ConfigError(format!("Feature '{}' not found", name))
        })
    }

    /// Get the current value of a feature.
    pub fn get_value(&self, name: &str) -> Result<FeatureValue> {
        Ok(self.get(name)?.current_value)
    }

    /// Set the value of a feature.
    pub fn set_value(&self, name: &str, value: FeatureValue) -> Result<()> {
        let mut features = self.features.write();
        if let Some(descriptor) = features.get_mut(name) {
            if !descriptor.writable {
                return Err(OptikError::ConfigError(format!(
                    "Feature '{}' is not writable",
                    name
                )));
            }
            descriptor.constraints.validate(&value)?;
            descriptor.current_value = value;
            Ok(())
        } else {
            Err(OptikError::ConfigError(format!(
                "Feature '{}' not found",
                name
            )))
        }
    }

    /// List all registered features.
    pub fn list(&self) -> Vec<String> {
        let features = self.features.read();
        features.keys().cloned().collect()
    }

    /// Get count of registered features.
    pub fn count(&self) -> usize {
        let features = self.features.read();
        features.len()
    }

    /// Clear all features.
    pub fn clear(&self) {
        let mut features = self.features.write();
        features.clear();
    }

    /// Get all descriptors.
    pub fn all(&self) -> Vec<FeatureDescriptor> {
        let features = self.features.read();
        features.values().cloned().collect()
    }
}

impl Default for FeatureRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for FeatureDescriptor
pub struct FeatureDescriptorBuilder {
    name: String,
    description: Option<String>,
    current_value: FeatureValue,
    readable: bool,
    writable: bool,
    constraints: FeatureConstraints,
}

impl FeatureDescriptorBuilder {
    /// Create a new feature descriptor builder.
    pub fn new(name: String, value: FeatureValue) -> Self {
        Self {
            name,
            description: None,
            current_value: value,
            readable: true,
            writable: true,
            constraints: FeatureConstraints {
                min: None,
                max: None,
                step: None,
                enum_values: None,
            },
        }
    }

    /// Set the description.
    pub fn description(mut self, desc: String) -> Self {
        self.description = Some(desc);
        self
    }

    /// Set readable flag.
    pub fn readable(mut self, readable: bool) -> Self {
        self.readable = readable;
        self
    }

    /// Set writable flag.
    pub fn writable(mut self, writable: bool) -> Self {
        self.writable = writable;
        self
    }

    /// Set min constraint.
    pub fn min(mut self, min: f64) -> Self {
        self.constraints.min = Some(min);
        self
    }

    /// Set max constraint.
    pub fn max(mut self, max: f64) -> Self {
        self.constraints.max = Some(max);
        self
    }

    /// Set step constraint.
    pub fn step(mut self, step: f64) -> Self {
        self.constraints.step = Some(step);
        self
    }

    /// Set enum values.
    pub fn enum_values(mut self, values: Vec<String>) -> Self {
        self.constraints.enum_values = Some(values);
        self
    }

    /// Build the descriptor.
    pub fn build(self) -> FeatureDescriptor {
        FeatureDescriptor {
            name: self.name,
            description: self.description,
            current_value: self.current_value,
            readable: self.readable,
            writable: self.writable,
            constraints: self.constraints,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_value_conversions() {
        let int_val = FeatureValue::Integer(100);
        assert_eq!(int_val.as_f64().unwrap(), 100.0);
        assert_eq!(int_val.as_i64().unwrap(), 100);

        let float_val = FeatureValue::Float(42.5);
        assert_eq!(float_val.as_f64().unwrap(), 42.5);
        assert_eq!(float_val.as_i64().unwrap(), 42);

        let bool_val = FeatureValue::Boolean(true);
        assert!(bool_val.as_bool().unwrap());

        let str_val = FeatureValue::String("test".to_string());
        assert_eq!(str_val.as_string().unwrap(), "test");
    }

    #[test]
    fn test_feature_constraints_validation() {
        let constraints = FeatureConstraints {
            min: Some(0.0),
            max: Some(100.0),
            step: None,
            enum_values: None,
        };

        let valid = FeatureValue::Float(50.0);
        assert!(constraints.validate(&valid).is_ok());

        let below_min = FeatureValue::Float(-10.0);
        assert!(constraints.validate(&below_min).is_err());

        let above_max = FeatureValue::Float(150.0);
        assert!(constraints.validate(&above_max).is_err());
    }

    #[test]
    fn test_feature_enum_validation() {
        let constraints = FeatureConstraints {
            min: None,
            max: None,
            step: None,
            enum_values: Some(vec!["RAW".to_string(), "JPEG".to_string()]),
        };

        let valid = FeatureValue::Enum("RAW".to_string());
        assert!(constraints.validate(&valid).is_ok());

        let invalid = FeatureValue::Enum("PNG".to_string());
        assert!(constraints.validate(&invalid).is_err());
    }

    #[test]
    fn test_feature_registry_register() {
        let registry = FeatureRegistry::new();

        let descriptor = FeatureDescriptorBuilder::new(
            "ExposureTime".to_string(),
            FeatureValue::Float(1000.0),
        )
        .description("Exposure time in microseconds".to_string())
        .min(100.0)
        .max(1000000.0)
        .build();

        registry.register(descriptor);
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_feature_registry_get_set() {
        let registry = FeatureRegistry::new();

        let descriptor = FeatureDescriptorBuilder::new(
            "Gain".to_string(),
            FeatureValue::Float(10.0),
        )
        .min(0.0)
        .max(48.0)
        .build();

        registry.register(descriptor);

        // Get current value
        let current = registry.get_value("Gain").unwrap();
        assert_eq!(current.as_f64().unwrap(), 10.0);

        // Set new value
        registry
            .set_value("Gain", FeatureValue::Float(25.0))
            .unwrap();
        let new_value = registry.get_value("Gain").unwrap();
        assert_eq!(new_value.as_f64().unwrap(), 25.0);
    }

    #[test]
    fn test_feature_registry_constraints() {
        let registry = FeatureRegistry::new();

        let descriptor = FeatureDescriptorBuilder::new(
            "Gain".to_string(),
            FeatureValue::Float(10.0),
        )
        .min(0.0)
        .max(48.0)
        .build();

        registry.register(descriptor);

        // Try to set value above max
        let result = registry.set_value("Gain", FeatureValue::Float(100.0));
        assert!(result.is_err());

        // Try to set valid value
        let result = registry.set_value("Gain", FeatureValue::Float(30.0));
        assert!(result.is_ok());
    }

    #[test]
    fn test_feature_registry_list() {
        let registry = FeatureRegistry::new();

        registry.register(
            FeatureDescriptorBuilder::new(
                "ExposureTime".to_string(),
                FeatureValue::Float(1000.0),
            )
            .build(),
        );

        registry.register(
            FeatureDescriptorBuilder::new("Gain".to_string(), FeatureValue::Float(10.0))
                .build(),
        );

        let list = registry.list();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"ExposureTime".to_string()));
        assert!(list.contains(&"Gain".to_string()));
    }

    #[test]
    fn test_feature_registry_readonly() {
        let registry = FeatureRegistry::new();

        let descriptor = FeatureDescriptorBuilder::new(
            "SerialNumber".to_string(),
            FeatureValue::String("SN123456".to_string()),
        )
        .readable(true)
        .writable(false)
        .build();

        registry.register(descriptor);

        // Try to write to readonly feature
        let result = registry.set_value(
            "SerialNumber",
            FeatureValue::String("SN999999".to_string()),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_feature_value_display() {
        assert_eq!(FeatureValue::Integer(42).to_string(), "42");
        assert_eq!(
            FeatureValue::String("test".to_string()).to_string(),
            "test"
        );
        assert_eq!(FeatureValue::Boolean(true).to_string(), "true");
    }
}
