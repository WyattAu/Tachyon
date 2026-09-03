//! Client-side password strength metering.
//!
//! Single source of password-strength logic for this crate; pages must
//! not re-implement it inline.
//!
//! Server-side counterpart: the `salting` crate's `Policy` and
//! `check_password` (feature `strength`) are the authoritative checks —
//! a 12-character minimum with uppercase, lowercase, digit, and special
//! character classes, plus zxcvbn guessability scoring. This meter only
//! mirrors those thresholds for UX feedback and is not a security control.

/// Coarse strength bucket shown in the registration strength meter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PasswordStrength {
    Weak,
    Medium,
    Strong,
}

impl PasswordStrength {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            PasswordStrength::Weak => "Weak",
            PasswordStrength::Medium => "Medium",
            PasswordStrength::Strong => "Strong",
        }
    }

    pub(crate) fn color_class(&self) -> &'static str {
        match self {
            PasswordStrength::Weak => "bg-red-500",
            PasswordStrength::Medium => "bg-yellow-500",
            PasswordStrength::Strong => "bg-green-500",
        }
    }

    pub(crate) fn text_color(&self) -> &'static str {
        match self {
            PasswordStrength::Weak => "text-red-500",
            PasswordStrength::Medium => "text-yellow-500",
            PasswordStrength::Strong => "text-green-500",
        }
    }

    pub(crate) fn width_pct(&self) -> &'static str {
        match self {
            PasswordStrength::Weak => "w-1/3",
            PasswordStrength::Medium => "w-2/3",
            PasswordStrength::Strong => "w-full",
        }
    }
}

/// Score a password for the client-side strength meter.
///
/// Thresholds mirror the server-side `salting::Policy` defaults: 12+
/// characters satisfies the policy minimum, 8+ remains the registration
/// floor. Each of length ≥ 8, length ≥ 12, an uppercase letter, a digit,
/// and a non-alphanumeric character adds one point:
/// 0-2 → Weak, 3 → Medium, 4+ → Strong.
pub(crate) fn calc_password_strength(password: &str) -> PasswordStrength {
    let mut score = 0u8;
    if password.len() >= 8 {
        score += 1;
    }
    if password.len() >= 12 {
        score += 1;
    }
    if password.chars().any(|c| c.is_uppercase()) {
        score += 1;
    }
    if password.chars().any(|c| c.is_ascii_digit()) {
        score += 1;
    }
    if password.chars().any(|c| !c.is_alphanumeric()) {
        score += 1;
    }
    match score {
        0..=2 => PasswordStrength::Weak,
        3 => PasswordStrength::Medium,
        _ => PasswordStrength::Strong,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_or_short_passwords_are_weak() {
        assert_eq!(calc_password_strength(""), PasswordStrength::Weak);
        assert_eq!(calc_password_strength("abc123"), PasswordStrength::Weak);
        assert_eq!(calc_password_strength("abcdefgh"), PasswordStrength::Weak);
        assert_eq!(calc_password_strength("Abcdefgh"), PasswordStrength::Weak);
    }

    #[test]
    fn three_criteria_is_medium() {
        assert_eq!(calc_password_strength("Abcdefg1"), PasswordStrength::Medium);
        assert_eq!(
            calc_password_strength("abcdefgh1!"),
            PasswordStrength::Medium
        );
    }

    #[test]
    fn four_or_more_criteria_is_strong() {
        assert_eq!(
            calc_password_strength("Abcdefgh1!"),
            PasswordStrength::Strong
        );
        assert_eq!(
            calc_password_strength("Abcdefghij1!"),
            PasswordStrength::Strong
        );
    }

    #[test]
    fn twelve_chars_counts_double_length_points() {
        // Length ≥ 8 and ≥ 12 both fire: 2 + upper + digit = 4 → Strong.
        assert_eq!(
            calc_password_strength("Abcdefgh1234"),
            PasswordStrength::Strong
        );
        // Without classes: 2 length points only → Weak.
        assert_eq!(
            calc_password_strength("abcdefghijkl"),
            PasswordStrength::Weak
        );
    }
}
