use dialoguer::theme::{ColorfulTheme, Theme};

pub struct CustomTerminalTheme {
    inner: ColorfulTheme,
}

impl Default for CustomTerminalTheme {
    fn default() -> Self {
        Self {
            inner: ColorfulTheme::default(),
        }
    }
}

impl Theme for CustomTerminalTheme {
    fn format_input_prompt(
        &self,
        f: &mut dyn std::fmt::Write,
        prompt: &str,
        _default: Option<&str>,
    ) -> std::fmt::Result {
        write!(f, "{}", prompt)
    }

    fn format_prompt(&self, f: &mut dyn std::fmt::Write, prompt: &str) -> std::fmt::Result {
        write!(f, "{}", prompt)
    }

    fn format_error(&self, f: &mut dyn std::fmt::Write, err: &str) -> std::fmt::Result {
        self.inner.format_error(f, err)
    }
}
