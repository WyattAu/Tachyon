// User input validation
// Validates username, email, password, and other user-related inputs

use super::ValidationResult;
use super::common::*;

pub const MIN_USERNAME_LENGTH: usize = 3;
pub const MAX_USERNAME_LENGTH: usize = 50;
pub const MIN_DISPLAY_NAME_LENGTH: usize = 1;
pub const MAX_DISPLAY_NAME_LENGTH: usize = 100;
pub const MIN_PASSWORD_LENGTH: usize = 8;
pub const MAX_PASSWORD_LENGTH: usize = 128;

#[derive(Debug, Clone)]
pub struct ValidatedUsername {
    value: String,
}

impl ValidatedUsername {
    pub fn new(username: &str) -> ValidationResult<Self> {
        let username = username.trim();

        if username.is_empty() {
            return Err(ValidationError::Required);
        }

        validate_length(username, MIN_USERNAME_LENGTH, MAX_USERNAME_LENGTH)?;

        if !username
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(ValidationError::InvalidFormat {
                message: "Username can only contain letters, numbers, underscores, and hyphens"
                    .to_string(),
            });
        }

        if username.starts_with(|c: char| c.is_numeric()) {
            return Err(ValidationError::InvalidFormat {
                message: "Username cannot start with a number".to_string(),
            });
        }

        let reserved = [
            "admin",
            "root",
            "system",
            "administrator",
            "moderator",
            "api",
            "www",
            "mail",
            "email",
            "support",
            "help",
            "guest",
            "user",
            "users",
            "tachyon",
            "server",
        ];

        if reserved.contains(&username.to_lowercase().as_str()) {
            return Err(ValidationError::ForbiddenContent {
                reason: "This username is reserved".to_string(),
            });
        }

        Ok(Self {
            value: username.to_string(),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn into_inner(self) -> String {
        self.value
    }
}

impl AsRef<str> for ValidatedUsername {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedDisplayName {
    value: String,
}

impl ValidatedDisplayName {
    pub fn new(name: &str) -> ValidationResult<Self> {
        let name = name.trim();

        if name.is_empty() {
            return Err(ValidationError::Required);
        }

        validate_length(name, MIN_DISPLAY_NAME_LENGTH, MAX_DISPLAY_NAME_LENGTH)?;
        validate_no_html(name)?;
        validate_no_scripts(name)?;

        let name = normalize_whitespace(name);

        Ok(Self { value: name })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn into_inner(self) -> String {
        self.value
    }
}

impl AsRef<str> for ValidatedDisplayName {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedEmail {
    value: String,
}

impl ValidatedEmail {
    pub fn new(email: &str) -> ValidationResult<Self> {
        let email = email.trim().to_lowercase();

        validate_email(&email)?;

        let disposable_domains = [
            "tempmail.com",
            "throwaway.email",
            "guerrillamail.com",
            "mailinator.com",
            "10minutemail.com",
        ];

        if let Some(domain) = email.split('@').nth(1)
            && disposable_domains.contains(&domain) {
                return Err(ValidationError::ForbiddenContent {
                    reason: "Disposable email addresses are not allowed".to_string(),
                });
            }

        Ok(Self { value: email })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn into_inner(self) -> String {
        self.value
    }
}

impl AsRef<str> for ValidatedEmail {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedPassword {
    value: String,
}

impl ValidatedPassword {
    pub fn new(password: &str) -> ValidationResult<Self> {
        if password.is_empty() {
            return Err(ValidationError::Required);
        }

        validate_length(password, MIN_PASSWORD_LENGTH, MAX_PASSWORD_LENGTH)?;

        let mut has_upper = false;
        let mut has_lower = false;
        let mut has_digit = false;
        let mut has_special = false;

        for c in password.chars() {
            if c.is_uppercase() {
                has_upper = true;
            } else if c.is_lowercase() {
                has_lower = true;
            } else if c.is_numeric() {
                has_digit = true;
            } else if "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c) {
                has_special = true;
            }
        }

        let complexity_count = [has_upper, has_lower, has_digit, has_special]
            .iter()
            .filter(|&&x| x)
            .count();

        if complexity_count < 3 {
            return Err(ValidationError::InvalidFormat {
                message: "Password must contain at least 3 of: uppercase, lowercase, numbers, special characters".to_string(),
            });
        }

        let common_passwords = [
            "password",
            "password123",
            "12345678",
            "qwerty123",
            "admin123",
            "letmein",
            "welcome1",
            "password1!",
        ];

        if common_passwords.contains(&password.to_lowercase().as_str()) {
            return Err(ValidationError::ForbiddenContent {
                reason: "This password is too common".to_string(),
            });
        }

        Ok(Self {
            value: password.to_string(),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn into_inner(self) -> String {
        self.value
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedUserId {
    value: String,
}

impl ValidatedUserId {
    pub fn new(id: &str) -> ValidationResult<Self> {
        let id = id.trim();

        if id.is_empty() {
            return Err(ValidationError::Required);
        }

        validate_uuid(id)?;

        Ok(Self {
            value: id.to_lowercase(),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn into_inner(self) -> String {
        self.value
    }
}

#[derive(Debug, Clone)]
pub enum UserRoleValue {
    Admin,
    Editor,
    Writer,
    Reader,
}

impl UserRoleValue {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> ValidationResult<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Self::Admin),
            "editor" => Ok(Self::Editor),
            "writer" => Ok(Self::Writer),
            "reader" => Ok(Self::Reader),
            _ => Err(ValidationError::InvalidFormat {
                message: "Role must be 'admin', 'editor', 'writer', or 'reader'".to_string(),
            }),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Editor => "editor",
            Self::Writer => "writer",
            Self::Reader => "reader",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedApiKey {
    value: String,
}

impl ValidatedApiKey {
    pub fn new(key: &str) -> ValidationResult<Self> {
        let key = key.trim();

        if key.is_empty() {
            return Err(ValidationError::Required);
        }

        validate_length(key, 20, 100)?;

        if !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(ValidationError::InvalidFormat {
                message: "API key contains invalid characters".to_string(),
            });
        }

        Ok(Self {
            value: key.to_string(),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn into_inner(self) -> String {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validated_username() {
        assert!(ValidatedUsername::new("john_doe").is_ok());
        assert!(ValidatedUsername::new("johndoe123").is_ok());
        assert!(ValidatedUsername::new("ab").is_err());
        assert!(ValidatedUsername::new("123user").is_err());
        assert!(ValidatedUsername::new("admin").is_err());
        assert!(ValidatedUsername::new("user@name").is_err());
    }

    #[test]
    fn test_validated_email() {
        assert!(ValidatedEmail::new("user@example.com").is_ok());
        assert!(ValidatedEmail::new("user.name+tag@example.org").is_ok());
        assert!(ValidatedEmail::new("invalid").is_err());
        assert!(ValidatedEmail::new("user@tempmail.com").is_err());
    }

    #[test]
    fn test_validated_password() {
        assert!(ValidatedPassword::new("Password123!").is_ok());
        assert!(ValidatedPassword::new("Short1!").is_err());
        assert!(ValidatedPassword::new("password123").is_err()); // no uppercase, no special
        assert!(ValidatedPassword::new("PASSWORD123!").is_ok()); // has 3 of 4: upper, digit, special
        assert!(ValidatedPassword::new("Password1!").is_err()); // common password
        assert!(ValidatedPassword::new("password").is_err()); // only lowercase
        assert!(ValidatedPassword::new("PASSWORD").is_err()); // only uppercase
    }

    #[test]
    fn test_validated_display_name() {
        assert!(ValidatedDisplayName::new("John Doe").is_ok());
        assert!(ValidatedDisplayName::new("John<script>").is_err());
        assert!(ValidatedDisplayName::new("").is_err());
    }
}
