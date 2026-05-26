//! Identifier and casing helpers shared across the codegen submodules.

/// Convert a string to PascalCase.
pub(super) fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Whether `s` is a valid, non-keyword Rust identifier — so it can name a
/// generated type without panicking `Ident::new`. Real-world specs sometimes
/// carry struct names that are inline type literals, not identifiers.
pub(super) fn is_valid_ident(s: &str) -> bool {
    syn::parse_str::<syn::Ident>(s).is_ok()
}

/// Convert a string to snake_case.
pub(super) fn to_snake_case(s: &str) -> String {
    let mut result = String::new();

    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else if c == '-' || c == ' ' {
            result.push('_');
        } else {
            result.push(c);
        }
    }

    result
}

/// Check if a string is a Rust keyword.
pub(super) fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "try"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
    )
}

/// Sanitize a string to be a valid Rust identifier.
pub(super) fn sanitize_identifier(s: &str) -> String {
    let sanitized: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    // Ensure it doesn't start with a digit
    if sanitized.starts_with(|c: char| c.is_ascii_digit()) {
        format!("network_{sanitized}")
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("calculator"), "Calculator");
        assert_eq!(to_pascal_case("add_liquidity"), "AddLiquidity");
        assert_eq!(to_pascal_case("myContract"), "MyContract");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("addLiquidity"), "add_liquidity");
        assert_eq!(to_snake_case("getBalance"), "get_balance");
        assert_eq!(to_snake_case("add"), "add");
    }

    #[test]
    fn test_is_rust_keyword() {
        assert!(is_rust_keyword("type"));
        assert!(is_rust_keyword("fn"));
        assert!(!is_rust_keyword("add"));
    }
}
