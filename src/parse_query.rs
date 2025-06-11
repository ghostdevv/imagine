#[derive(Debug, Clone)]
pub struct GifConfig {
    pub bucket_path: String,
    pub file_name: String,
    pub text: String,
}

pub fn parse_gif_path(pathname: &str) -> GifConfig {
    if pathname == "/base.gif" {
        return GifConfig {
            bucket_path: "base.gif".to_string(),
            file_name: "base".to_string(),
            text: "base".to_string(),
        };
    }

    // Process the key
    let chars: Vec<char> = pathname.chars().collect();
    let char_count = chars.len();

    // Skip first character ("/") and remove last 4 characters (".gif")
    let file_name: String = chars
        .into_iter()
        .skip(1)
        .take(char_count.saturating_sub(5))
        .collect::<String>()
        .trim()
        .replace(' ', "_")
        .to_lowercase()
        .chars()
        .take(30)
        .collect();

    // Create name from key
    let text = file_name.replace('_', " ").trim().to_uppercase();

    GifConfig {
        bucket_path: format!("generated/{}.gif", file_name),
        file_name,
        text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_base_case() {
        let result = parse_gif_path("/base.gif");
        assert_eq!(result.file_name, "base");
        assert_eq!(result.text, "base");
        assert_eq!(result.bucket_path, "base.gif");
    }

    #[test]
    fn test_parse_query() {
        let result = parse_gif_path("/hello world test.png");
        assert_eq!(result.file_name, "hello_world_test");
        assert_eq!(result.text, "HELLO WORLD TEST");
        assert_eq!(result.bucket_path, "generated/hello_world_test.gif");
    }

    #[test]
    fn test_parse_query_with_long_string() {
        let result = parse_gif_path("/this is a very long string that should be truncated.png");
        assert_eq!(result.file_name.len(), 30);
        assert_eq!(result.file_name, "this_is_a_very_long_string_tha");
    }

    #[test]
    fn test_parse_query_with_extra_spaces() {
        let result = parse_gif_path("/  hello   world  .png");
        assert_eq!(result.file_name, "hello___world");
        assert_eq!(result.text, "HELLO   WORLD");
    }

    #[test]
    fn test_correctly_handles_empty() {
        let result = parse_gif_path("/  .gif");
        assert_eq!(result.file_name, "");
        assert_eq!(result.text, "");
        assert_eq!(result.bucket_path, "generated/.gif");
    }

    #[test]
    fn test_handles_emoji() {
        let result = parse_gif_path("/😀.gif");
        assert_eq!(result.file_name, "😀");
        assert_eq!(result.text, "😀");
        assert_eq!(result.bucket_path, "generated/😀.gif");
    }

    #[test]
    fn test_handles_mixed_emoji_and_text() {
        let result = parse_gif_path("/hello 😀 world.gif");
        assert_eq!(result.file_name, "hello_😀_world");
        assert_eq!(result.text, "HELLO 😀 WORLD");
        assert_eq!(result.bucket_path, "generated/hello_😀_world.gif");
    }
}
