#![no_main]

use libfuzzer_sys::fuzz_target;
use s1bcr4ft_core::command_validator::CommandValidator;

// Fuzz target for CommandValidator::parse()
fuzz_target!(|data: &[u8]| {
    // Skip if data is too large (avoid DoS)
    if data.len() > 10_000 {
        return;
    }

    // Try to convert bytes to string (handle invalid UTF-8)
    let command_str = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return, // Skip invalid UTF-8
    };

    // Skip empty strings
    if command_str.is_empty() {
        return;
    }

    // Create validator with default whitelist
    let validator = CommandValidator::new();

    // Parse the command - should never panic or crash
    let _ = validator.parse(command_str);

    // Test various whitelist configurations
    if data.len() > 1 {
        let test_whitelist = vec!["pacman".to_string(), "paru".to_string()];
        let validator_whitelist = CommandValidator::with_whitelist(test_whitelist);
        let _ = validator_whitelist.parse(command_str);
    }

    // Test with builder methods
    if data.len() > 2 {
        let validator_abs = CommandValidator::new().allow_absolute_paths();
        let _ = validator_abs.parse(command_str);

        let validator_shell = CommandValidator::new().allow_shell_metachars();
        let _ = validator_shell.parse(command_str);
    }
});
