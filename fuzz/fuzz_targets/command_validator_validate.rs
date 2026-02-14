#![no_main]

use libfuzzer_sys::fuzz_target;
use s1bcr4ft_core::command_validator::CommandValidator;

// Fuzz target for CommandValidator::validate() edge cases
fuzz_target!(|data: &[u8]| {
    // Skip if data is too large
    if data.len() > 8_000 {
        return;
    }

    // Convert to string or skip invalid UTF-8
    let command_str = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return,
    };

    // Skip empty strings
    if command_str.is_empty() {
        return;
    }

    // Test with default validator
    let validator = CommandValidator::new();

    // Parse and validate - should never panic
    if let Ok(parsed) = validator.parse(command_str) {
        let _ = validator.validate(&parsed);
    }

    // Test with different whitelist configurations
    if data.len() > 1 {
        let whitelist = vec!["pacman".to_string(), "paru".to_string()];
        let validator_whitelist = CommandValidator::with_whitelist(whitelist);

        if let Ok(parsed) = validator_whitelist.parse(command_str) {
            let _ = validator_whitelist.validate(&parsed);
        }
    }

    // Test with builder methods
    if data.len() > 2 {
        let validator_abs = CommandValidator::new().allow_absolute_paths();

        if let Ok(parsed) = validator_abs.parse(command_str) {
            let _ = validator_abs.validate(&parsed);
        }

        let validator_shell = CommandValidator::new().allow_shell_metachars();

        if let Ok(parsed) = validator_shell.parse(command_str) {
            let _ = validator_shell.validate(&parsed);
        }
    }

    // Test with special patterns
    if data.len() > 5 {
        let special_patterns = [
            "$(rm -rf /)",
            "; rm -rf /",
            "| cat /etc/passwd",
            "`whoami`",
            "$(malicious)",
            "\\x00", // null byte
            "\\x1b", // escape
        ];

        for pattern in special_patterns {
            let test_cmd = if data.len() > pattern.len() {
                // Use char_indices to find valid UTF-8 boundary
                let end_pos = command_str
                    .char_indices()
                    .nth(command_str.chars().count().min(pattern.chars().count()))
                    .map(|(i, _)| i)
                    .unwrap_or(command_str.len());
                &command_str[..end_pos]
            } else {
                pattern
            };

            let _ = validator.parse(test_cmd);
        }
    }
});
