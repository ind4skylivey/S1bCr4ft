use crate::error::{Result, S1bCr4ftError};
use std::path::Path;
use std::process::Command;

/// Parsed command with executable and arguments
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedCommand {
    pub executable: String,
    pub arguments: Vec<String>,
}

/// Command validator with whitelist and sanitization
pub struct CommandValidator {
    /// Whitelist of allowed executables
    allowed_executables: Vec<String>,
    /// Whether to allow absolute paths
    allow_absolute_paths: bool,
    /// Whether to allow shell metacharacters in arguments
    allow_shell_metachars: bool,
}

impl Default for CommandValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandValidator {
    /// Create a new command validator with default whitelist
    ///
    /// Default allowed commands for system modules:
    /// - systemctl (systemd service management)
    /// - usermod, useradd, userdel (user management)
    /// - groupmod, groupadd, groupdel (group management)
    /// - sysctl (kernel parameter configuration)
    /// - udevadm (device management)
    /// - locale-gen (locale generation)
    /// - hwclock (hardware clock)
    /// - timedatectl (time management)
    pub fn new() -> Self {
        Self {
            allowed_executables: vec![
                "systemctl".to_string(),
                "usermod".to_string(),
                "useradd".to_string(),
                "userdel".to_string(),
                "groupmod".to_string(),
                "groupadd".to_string(),
                "groupdel".to_string(),
                "sysctl".to_string(),
                "udevadm".to_string(),
                "locale-gen".to_string(),
                "hwclock".to_string(),
                "timedatectl".to_string(),
            ],
            allow_absolute_paths: false,
            allow_shell_metachars: false,
        }
    }

    /// Create a custom validator with explicit whitelist
    pub fn with_whitelist(allowed: Vec<String>) -> Self {
        Self {
            allowed_executables: allowed,
            allow_absolute_paths: false,
            allow_shell_metachars: false,
        }
    }

    /// Allow absolute paths in executables
    pub fn allow_absolute_paths(mut self) -> Self {
        self.allow_absolute_paths = true;
        self
    }

    /// Allow shell metacharacters in arguments (DANGEROUS)
    pub fn allow_shell_metachars(mut self) -> Self {
        self.allow_shell_metachars = true;
        self
    }

    /// Parse a command string into executable and arguments
    ///
    /// This uses a simple parser that respects quotes and escaping:
    /// - Single quotes: 'text'
    /// - Double quotes: "text"
    /// - Backslashes: \escape
    ///
    /// # Examples
    ///
    /// ```
    /// use s1bcr4ft_core::command_validator::CommandValidator;
    ///
    /// let validator = CommandValidator::new();
    /// let parsed = validator.parse("systemctl enable NetworkManager").unwrap();
    /// assert_eq!(parsed.executable, "systemctl");
    /// assert_eq!(parsed.arguments, ["enable", "NetworkManager"]);
    /// ```
    pub fn parse(&self, command: &str) -> Result<ParsedCommand> {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return Err(S1bCr4ftError::package("Empty command".to_string()));
        }

        // Simple shell-like parser
        let chars = trimmed.chars().peekable();
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut escaped = false;

        for c in chars {
            match c {
                '\\' if !in_single_quote && !escaped => {
                    escaped = true;
                }
                '\'' if !in_double_quote && !escaped => {
                    in_single_quote = !in_single_quote;
                }
                '"' if !in_single_quote && !escaped => {
                    in_double_quote = !in_double_quote;
                }
                ' ' | '\t' if !in_single_quote && !in_double_quote && !escaped => {
                    if !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                }
                _ => {
                    if !escaped {
                        current.push(c);
                    } else {
                        current.push(c);
                        escaped = false;
                    }
                }
            }
        }

        if !current.is_empty() {
            parts.push(current);
        }

        if parts.is_empty() {
            return Err(S1bCr4ftError::package(
                "Failed to parse command: no executable found".to_string(),
            ));
        }

        let executable = parts[0].clone();
        let arguments = parts[1..].to_vec();

        Ok(ParsedCommand {
            executable,
            arguments,
        })
    }

    /// Validate a parsed command
    ///
    /// Checks:
    /// 1. Executable is in whitelist or is an absolute path (if allowed)
    /// 2. Executable contains only safe characters
    /// 3. Arguments don't contain shell metacharacters (unless allowed)
    /// 4. Arguments are within reasonable length limits
    ///
    /// # Security
    ///
    /// This method prevents command injection by:
    /// - Restricting executables to a whitelist
    /// - Disallowing shell operators (;, &, |, >, <, $, `, etc.)
    /// - Validating character sets
    pub fn validate(&self, command: &ParsedCommand) -> Result<()> {
        // Validate executable
        self.validate_executable(&command.executable)?;

        // Validate each argument
        for arg in &command.arguments {
            self.validate_argument(arg)?;
        }

        Ok(())
    }

    /// Validate the executable name
    fn validate_executable(&self, executable: &str) -> Result<()> {
        // Check if it's an absolute path
        if executable.starts_with('/') {
            if !self.allow_absolute_paths {
                return Err(S1bCr4ftError::package(format!(
                    "Absolute paths not allowed: {}",
                    executable
                )));
            }

            // Validate path components
            let path = Path::new(executable);
            if !path.is_absolute() || path.components().count() > 20 {
                return Err(S1bCr4ftError::package(format!(
                    "Invalid absolute path: {}",
                    executable
                )));
            }

            // Validate path characters
            for c in executable.chars() {
                if !is_safe_path_char(c) {
                    return Err(S1bCr4ftError::package(format!(
                        "Invalid character in path '{}': {}",
                        executable, c
                    )));
                }
            }

            return Ok(());
        }

        // Check whitelist
        if !self.allowed_executables.contains(&executable.to_string()) {
            return Err(S1bCr4ftError::package(format!(
                "Executable not in whitelist: {}. Allowed: {:?}",
                executable, self.allowed_executables
            )));
        }

        // Validate executable name characters
        for c in executable.chars() {
            if !is_safe_executable_char(c) {
                return Err(S1bCr4ftError::package(format!(
                    "Invalid character in executable '{}': {}",
                    executable, c
                )));
            }
        }

        Ok(())
    }

    /// Validate an argument
    fn validate_argument(&self, arg: &str) -> Result<()> {
        // Check length limits
        if arg.len() > 4096 {
            return Err(S1bCr4ftError::package(format!(
                "Argument too long: {} characters (max 4096)",
                arg.len()
            )));
        }

        // Check for shell metacharacters
        if !self.allow_shell_metachars {
            for c in arg.chars() {
                if is_shell_metachar(c) {
                    return Err(S1bCr4ftError::package(format!(
                        "Shell metacharacter not allowed in argument: '{}'. Character: {}",
                        arg, c
                    )));
                }
            }
        }

        // Validate character set
        for c in arg.chars() {
            if !is_safe_arg_char(c) {
                return Err(S1bCr4ftError::package(format!(
                    "Invalid character in argument '{}': {}",
                    arg, c
                )));
            }
        }

        Ok(())
    }

    /// Parse and validate a command string
    pub fn parse_and_validate(&self, command: &str) -> Result<ParsedCommand> {
        let parsed = self.parse(command)?;
        self.validate(&parsed)?;
        Ok(parsed)
    }

    /// Execute a command safely (after validation)
    ///
    /// This should only be called after parse_and_validate()
    ///
    /// # Security
    ///
    /// This method is safe because:
    /// - Command is validated before execution
    /// - No shell is involved
    /// - Arguments are passed directly to the process
    pub fn execute_safe(&self, command: &ParsedCommand) -> Result<std::process::Output> {
        Command::new(&command.executable)
            .args(&command.arguments)
            .output()
            .map_err(|e| {
                S1bCr4ftError::package(format!("Failed to execute {}: {}", command.executable, e))
            })
    }

    /// Execute a command string with validation
    ///
    /// This is the main entry point for safe command execution
    pub fn execute(&self, command: &str) -> Result<std::process::Output> {
        let parsed = self.parse_and_validate(command)?;
        self.execute_safe(&parsed)
    }
}

/// Check if a character is safe in an executable name
fn is_safe_executable_char(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_' || c == '.'
}

/// Check if a character is safe in a path
fn is_safe_path_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '/' | '-' | '_' | '.' | '@')
}

/// Check if a character is safe in an argument
fn is_safe_arg_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | ':' | '@' | '=' | ',')
}

/// Check if a character is a shell metacharacter
fn is_shell_metachar(c: char) -> bool {
    matches!(
        c,
        ';' | '&'
            | '|'
            | '>'
            | '<'
            | '$'
            | '`'
            | '\\'
            | '('
            | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | '!'
            | '#'
            | '~'
            | '*'
            | '?'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let validator = CommandValidator::new();
        let parsed = validator.parse("systemctl enable NetworkManager").unwrap();
        assert_eq!(parsed.executable, "systemctl");
        assert_eq!(parsed.arguments, vec!["enable", "NetworkManager"]);
    }

    #[test]
    fn test_parse_with_quotes() {
        let validator = CommandValidator::new();
        let parsed = validator.parse("echo \"hello world\"").unwrap();
        assert_eq!(parsed.executable, "echo");
        assert_eq!(parsed.arguments, vec!["hello world"]);
    }

    #[test]
    fn test_parse_single_quotes() {
        let validator = CommandValidator::new();
        let parsed = validator.parse("echo 'test value'").unwrap();
        assert_eq!(parsed.executable, "echo");
        assert_eq!(parsed.arguments, vec!["test value"]);
    }

    #[test]
    fn test_parse_empty_command() {
        let validator = CommandValidator::new();
        assert!(validator.parse("").is_err());
        assert!(validator.parse("   ").is_err());
    }

    #[test]
    fn test_validate_allowed_executable() {
        let validator = CommandValidator::new();
        let parsed = ParsedCommand {
            executable: "systemctl".to_string(),
            arguments: vec!["enable".to_string(), "NetworkManager".to_string()],
        };
        assert!(validator.validate(&parsed).is_ok());
    }

    #[test]
    fn test_validate_disallowed_executable() {
        let validator = CommandValidator::new();
        let parsed = ParsedCommand {
            executable: "rm".to_string(),
            arguments: vec!["-rf".to_string(), "/".to_string()],
        };
        assert!(validator.validate(&parsed).is_err());
    }

    #[test]
    fn test_validate_shell_injection() {
        let validator = CommandValidator::new();
        let parsed = ParsedCommand {
            executable: "systemctl".to_string(),
            arguments: vec!["enable; rm -rf /".to_string()],
        };
        assert!(validator.validate(&parsed).is_err());
    }

    #[test]
    fn test_validate_backticks() {
        let validator = CommandValidator::new();
        let parsed = ParsedCommand {
            executable: "systemctl".to_string(),
            arguments: vec!["enable`whoami`".to_string()],
        };
        assert!(validator.validate(&parsed).is_err());
    }

    #[test]
    fn test_validate_command_substitution() {
        let validator = CommandValidator::new();
        let parsed = ParsedCommand {
            executable: "systemctl".to_string(),
            arguments: vec!["enable$(echo pwned)".to_string()],
        };
        assert!(validator.validate(&parsed).is_err());
    }

    #[test]
    fn test_validate_pipe() {
        let validator = CommandValidator::new();
        let parsed = ParsedCommand {
            executable: "systemctl".to_string(),
            arguments: vec!["enable | cat".to_string()],
        };
        assert!(validator.validate(&parsed).is_err());
    }

    #[test]
    fn test_validate_ampersand() {
        let validator = CommandValidator::new();
        let parsed = ParsedCommand {
            executable: "systemctl".to_string(),
            arguments: vec!["enable & rm -rf /".to_string()],
        };
        assert!(validator.validate(&parsed).is_err());
    }

    #[test]
    fn test_validate_too_long_argument() {
        let validator = CommandValidator::new();
        let long_arg = "a".repeat(5000);
        let parsed = ParsedCommand {
            executable: "systemctl".to_string(),
            arguments: vec![long_arg],
        };
        assert!(validator.validate(&parsed).is_err());
    }

    #[test]
    fn test_parse_and_validate_safe_command() {
        let validator = CommandValidator::new();
        let result = validator.parse_and_validate("systemctl enable NetworkManager");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_and_validate_injected_command() {
        let validator = CommandValidator::new();
        let result = validator.parse_and_validate("systemctl enable NetworkManager; rm -rf /");
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_whitelist() {
        let validator = CommandValidator::with_whitelist(vec!["custom-cmd".to_string()]);
        let parsed = ParsedCommand {
            executable: "custom-cmd".to_string(),
            arguments: vec!["arg1".to_string()],
        };
        assert!(validator.validate(&parsed).is_ok());
    }

    #[test]
    fn test_absolute_path_not_allowed() {
        let validator = CommandValidator::new();
        let parsed = ParsedCommand {
            executable: "/usr/bin/rm".to_string(),
            arguments: vec!["-rf".to_string(), "/".to_string()],
        };
        assert!(validator.validate(&parsed).is_err());
    }

    #[test]
    fn test_absolute_path_allowed() {
        let validator = CommandValidator::new().allow_absolute_paths();
        let parsed = ParsedCommand {
            executable: "/usr/bin/systemctl".to_string(),
            arguments: vec!["enable".to_string(), "NetworkManager".to_string()],
        };
        assert!(validator.validate(&parsed).is_ok());
    }

    #[test]
    fn test_safe_arg_characters() {
        assert!(is_safe_arg_char('a'));
        assert!(is_safe_arg_char('Z'));
        assert!(is_safe_arg_char('0'));
        assert!(is_safe_arg_char('-'));
        assert!(is_safe_arg_char('_'));
        assert!(is_safe_arg_char('.'));
        assert!(is_safe_arg_char('/'));
        assert!(is_safe_arg_char(':'));
        assert!(is_safe_arg_char('@'));
        assert!(is_safe_arg_char('='));
        assert!(is_safe_arg_char(','));
        assert!(!is_safe_arg_char(';'));
        assert!(!is_safe_arg_char('&'));
        assert!(!is_safe_arg_char('|'));
        assert!(!is_safe_arg_char('`'));
        assert!(!is_safe_arg_char('$'));
    }

    #[test]
    fn test_shell_metacharacters() {
        assert!(is_shell_metachar(';'));
        assert!(is_shell_metachar('&'));
        assert!(is_shell_metachar('|'));
        assert!(is_shell_metachar('>'));
        assert!(is_shell_metachar('<'));
        assert!(is_shell_metachar('$'));
        assert!(is_shell_metachar('`'));
        assert!(is_shell_metachar('\\'));
        assert!(is_shell_metachar('('));
        assert!(is_shell_metachar(')'));
        assert!(is_shell_metachar('['));
        assert!(is_shell_metachar(']'));
        assert!(is_shell_metachar('{'));
        assert!(is_shell_metachar('}'));
        assert!(is_shell_metachar('!'));
        assert!(is_shell_metachar('#'));
        assert!(is_shell_metachar('~'));
        assert!(is_shell_metachar('*'));
        assert!(is_shell_metachar('?'));
        assert!(!is_shell_metachar('a'));
        assert!(!is_shell_metachar(' '));
        assert!(!is_shell_metachar('-'));
    }

    #[test]
    fn test_escape_sequences() {
        let validator = CommandValidator::new();
        let parsed = validator.parse("echo hello\\ world").unwrap();
        assert_eq!(parsed.arguments, vec!["hello world"]);
    }

    #[test]
    fn test_mixed_quotes() {
        let validator = CommandValidator::new();
        let parsed = validator.parse("echo \"it's\" 'test'").unwrap();
        assert_eq!(parsed.arguments, vec!["it's", "test"]);
    }
}
