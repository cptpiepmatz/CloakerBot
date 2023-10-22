use std::fmt::{Display, Write};

pub trait Template {
    fn template<const N: usize>(&self, inserts: [&dyn Display; N]) -> String;
}

impl Template for &str {
    fn template<const N: usize>(&self, inserts: [&dyn Display; N]) -> String {
        let mut output = String::with_capacity(self.len() * 2);
        let mut insertions: usize = 0;
        let mut chars = self.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '{' if insertions < N && chars.peek() == Some(&'}') => {
                    write!(output, "{}", inserts[insertions]).expect("infallible on string");
                    insertions += 1;
                    chars.next(); // Consume the '}' character
                }
                _ => output.push(c)
            }
        }

        output
    }
}
