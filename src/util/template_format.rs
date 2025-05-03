use std::fmt;

pub struct TemplateFormat<'t, T: ?Sized>(pub &'t T);

pub trait Template {
    fn template<'t>(&'t self) -> TemplateFormat<'t, Self> {
        TemplateFormat(self)
    }
}

impl Template for f64 {}
impl<'t> fmt::Display for TemplateFormat<'t, f64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut formatted = format!("{:.2}", self.0);
        let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
        write!(f, "{trimmed}")
    }
}
