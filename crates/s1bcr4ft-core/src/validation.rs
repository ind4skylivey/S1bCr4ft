use crate::config::Config;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

pub struct ConfigValidator;

impl ConfigValidator {
    pub fn validate(config: &Config) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate version
        if config.version.is_empty() {
            errors.push(ValidationError {
                field: "version".to_string(),
                message: "Version cannot be empty".to_string(),
            });
        }

        // Validate name
        if config.name.is_empty() {
            errors.push(ValidationError {
                field: "name".to_string(),
                message: "Name cannot be empty".to_string(),
            });
        }

        // Validate modules
        if config.modules.is_empty() {
            errors.push(ValidationError {
                field: "modules".to_string(),
                message: "At least one module must be specified".to_string(),
            });
        }

        Ok(errors)
    }
}
