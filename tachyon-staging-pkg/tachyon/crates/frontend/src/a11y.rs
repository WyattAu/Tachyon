pub fn aria_label(label: &str) -> String {
    format!(r#"aria-label="{}""#, label)
}

pub fn aria_labelledby(id: &str) -> String {
    format!(r#"aria-labelledby="{}""#, id)
}

pub fn aria_describedby(id: &str) -> String {
    format!(r#"aria-describedby="{}""#, id)
}

pub fn aria_hidden(val: bool) -> String {
    format!(r#"aria-hidden="{}""#, if val { "true" } else { "false" })
}

pub fn aria_expanded(val: bool) -> String {
    format!(r#"aria-expanded="{}""#, if val { "true" } else { "false" })
}

pub fn aria_selected(val: bool) -> String {
    format!(r#"aria-selected="{}""#, if val { "true" } else { "false" })
}

pub fn aria_controls(id: &str) -> String {
    format!(r#"aria-controls="{}""#, id)
}

pub fn aria_haspopup(val: &str) -> String {
    format!(r#"aria-haspopup="{}""#, val)
}

pub fn role(role: &str) -> String {
    format!(r#"role="{}""#, role)
}

pub fn tabindex(value: i32) -> String {
    format!(r#"tabindex="{}""#, value)
}

pub fn sr_only_class() -> &'static str {
    "sr-only"
}

pub fn focus_visible_class() -> &'static str {
    "focus-visible:outline-2 focus-visible:outline-blue-600 focus-visible:outline-offset-2"
}

pub fn screen_reader_only_class() -> &'static str {
    "sr-only"
}

pub fn screen_reader_only_style() -> &'static str {
    "position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border-width:0"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aria_label() {
        assert_eq!(aria_label("Close"), r#"aria-label="Close""#);
    }

    #[test]
    fn test_role() {
        assert_eq!(role("button"), r#"role="button""#);
    }

    #[test]
    fn test_aria_hidden() {
        assert_eq!(aria_hidden(true), r#"aria-hidden="true""#);
        assert_eq!(aria_hidden(false), r#"aria-hidden="false""#);
    }

    #[test]
    fn test_aria_expanded() {
        assert_eq!(aria_expanded(true), r#"aria-expanded="true""#);
        assert_eq!(aria_expanded(false), r#"aria-expanded="false""#);
    }

    #[test]
    fn test_tabindex() {
        assert_eq!(tabindex(0), r#"tabindex="0""#);
        assert_eq!(tabindex(-1), r#"tabindex="-1""#);
    }

    #[test]
    fn test_screen_reader_only_style() {
        assert!(screen_reader_only_style().contains("clip:rect"));
        assert!(screen_reader_only_style().contains("position:absolute"));
    }
}
