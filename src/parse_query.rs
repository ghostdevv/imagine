#[derive(Debug, Clone)]
pub struct QueryResult {
    pub key_path: String,
    pub key: String,
    pub name: String,
}

pub fn parse_query(pathname: &str) -> QueryResult {
    if pathname == "/base.gif" {
        return QueryResult {
            key_path: "base.gif".to_string(),
            key: "base".to_string(),
            name: "base".to_string(),
        };
    }

    // Process the key
    let key: String = pathname
        .chars()
        .skip(1) // Skip first character
        .take(pathname.len().saturating_sub(5)) // Remove last 4 characters (accounting for the first char already removed)
        .collect::<String>()
        .trim()
        .replace(' ', "_")
        .to_lowercase()
        .chars()
        .take(30)
        .collect();

    // Create name from key
    let name = key.replace('_', " ").to_uppercase();

    QueryResult {
        key_path: format!("generated/{}.gif", key),
        key,
        name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query() {
        let result = parse_query("/hello world test.png");
        assert_eq!(result.key, "hello_world_test");
        assert_eq!(result.name, "HELLO WORLD TEST");
        assert_eq!(result.key_path, "generated/hello_world_test.gif");
    }

    #[test]
    fn test_parse_query_with_long_string() {
        let result = parse_query("/this is a very long string that should be truncated.png");
        assert_eq!(result.key.len(), 30);
        assert_eq!(result.key, "this_is_a_very_long_string_tha");
    }

    #[test]
    fn test_parse_query_with_extra_spaces() {
        let result = parse_query("/  hello   world  .png");
        assert_eq!(result.key, "hello___world");
        assert_eq!(result.name, "HELLO   WORLD");
    }
}
