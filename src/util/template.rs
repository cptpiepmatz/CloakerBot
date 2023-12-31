use std::fmt::{Display, Write};

pub trait Template {
    fn template<const N: usize>(&self, inserts: [&dyn Display; N]) -> String;
}

impl Template for &str {
    fn template<const N: usize>(&self, inserts: [&dyn Display; N]) -> String {
        let mut output = String::with_capacity(self.len() * 2);
        let mut insertions: usize = 0;
        let mut chars = self.chars().peekable();

        while let (Some(next), peek) = (chars.next(), chars.peek()) {
            match (next, peek) {
                // escape sequence for literal '{'
                ('{', Some('{')) => {
                    chars.next(); // consume the next '{'
                    output.push('{');
                }

                // escape sequence for literal '}'
                ('}', Some('}')) => {
                    chars.next(); // consume the next '}'
                    output.push('}');
                }

                // found insertion element
                ('{', Some(_)) if insertions < N => {
                    // skip over the comment inside the curly braces
                    while chars.peek().is_some() && chars.peek() != Some(&'}') {
                        chars.next();
                    }

                    write!(output, "{}", inserts[insertions]).expect("infallible on string");
                    insertions += 1;
                    chars.next(); // consume the '}' character
                }

                _ => output.push(next)
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestStruct(i32);

    impl Display for TestStruct {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Test({})", self.0)
        }
    }

    #[test]
    fn test_insertions() {
        let template_str = "Hello, {name}! Your score is {score}.";
        let result = template_str.template([&"Alice", &123]);
        assert_eq!(result, "Hello, Alice! Your score is 123.");
    }

    #[test]
    fn test_too_few_insertions() {
        let template_str = "Hello, {name}! Your score is {score}.";
        let result = template_str.template([&"Alice"]);
        assert_eq!(result, "Hello, Alice! Your score is {score}.");
    }

    #[test]
    fn test_too_many_insertions() {
        let template_str = "Hello, {name}!";
        let result = template_str.template([&"Alice", &123]);
        assert_eq!(result, "Hello, Alice!");
    }

    #[test]
    fn test_escaped_literals() {
        let template_str = "This is a brace: {{ and this is another: }}";
        let result = template_str.template([]);
        assert_eq!(result, "This is a brace: { and this is another: }");
    }

    #[test]
    fn test_comments_in_placeholders() {
        let template_str = "Hello, {name: Alice}! Your object is {object: TestStruct}.";
        let alice = "Alice";
        let test_obj = TestStruct(42);
        let result = template_str.template([&alice, &test_obj]);
        assert_eq!(result, "Hello, Alice! Your object is Test(42).");
    }
}

