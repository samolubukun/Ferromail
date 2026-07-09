use serde_json::Value;

/// Render email template by replacing variables.
/// Supports `{{variable}}` and `{{variable ?? defaultValue}}` syntax.
/// Also supports nested access like `{{data.firstName}}`.
///
/// If a value is an array, it renders as a list: `<li>value</li>` separated by newlines.
pub fn render_template(template: &str, variables: &Value) -> String {
    let mut result = String::new();
    let mut remaining = template;

    while let Some(start_idx) = remaining.find("{{") {
        result.push_str(&remaining[..start_idx]);
        let tail = &remaining[start_idx + 2..];

        if let Some(end_idx) = tail.find("}}") {
            let key_expr = tail[..end_idx].trim();
            remaining = &tail[end_idx + 2..];

            // Split key expression by ??
            let parts: Vec<&str> = key_expr.split("??").map(|s| s.trim()).collect();
            let main_key = parts[0];
            let default_val = parts.get(1).copied().unwrap_or("");

            // Resolve value
            let resolved_value = resolve_value(variables, main_key);

            match resolved_value {
                Some(Value::Array(arr)) => {
                    let mut list_html = String::new();
                    for (i, item) in arr.iter().enumerate() {
                        let item_str = match item {
                            Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        list_html.push_str(&format!("<li>{}</li>", item_str));
                        if i < arr.len() - 1 {
                            list_html.push('\n');
                        }
                    }
                    result.push_str(&list_html);
                }
                Some(Value::String(s)) => {
                    result.push_str(s);
                }
                Some(Value::Null) | None => {
                    result.push_str(default_val);
                }
                Some(other) => {
                    // numbers, booleans, etc.
                    result.push_str(&other.to_string());
                }
            }
        } else {
            // Unclosed {{
            result.push_str("{{");
            remaining = tail;
        }
    }
    result.push_str(remaining);
    result
}

// Helper to look up a path in a JSON object
fn get_nested_value<'a>(val: &'a Value, path: &str) -> Option<&'a Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = val;
    for part in parts {
        match current {
            Value::Object(map) => {
                current = map.get(part)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

fn resolve_value<'a>(variables: &'a Value, main_key: &str) -> Option<&'a Value> {
    // 1. Try as nested path (e.g., data.firstName)
    if let Some(v) = get_nested_value(variables, main_key) {
        if !v.is_null() {
            return Some(v);
        }
    }

    // 2. Try as top-level property
    if let Value::Object(ref map) = variables {
        if let Some(v) = map.get(main_key) {
            if !v.is_null() {
                return Some(v);
            }
        }
    }

    // 3. Try in data object: variables["data"][main_key]
    if let Some(data_val) = variables.get("data") {
        if let Some(v) = data_val.get(main_key) {
            if !v.is_null() {
                return Some(v);
            }
        }
    }

    None
}

/// Detects whether an email body contains an unsubscribe signal.
///
/// Returns true if any of the following are present:
/// - Plunk template variables: {{unsubscribeUrl}} or {{manageUrl}}
/// - An <a> tag whose href contains unsubscribe-related keywords
/// - An <a> tag whose visible text contains unsubscribe-related keywords
pub fn detect_unsubscribe_signal(body: &str) -> bool {
    if body.is_empty() {
        return false;
    }

    // 1. Plunk's own managed unsubscribe variables
    if body.contains("{{unsubscribeUrl}}") || body.contains("{{manageUrl}}") {
        return true;
    }

    // 2. href containing unsubscribe keywords
    // Equivalent regex: /href=["'][^"']*(?:unsubscribe|opt[_-]?out|remove)[^"']*["']/i
    // We can do a case-insensitive search to be fast and safe.
    // Instead of regex compile-time overhead, we can write a simple stateful parser or search.
    // Let's implement a robust tag check.
    let lower_body = body.to_lowercase();
    
    // We check for `<a ` tags
    let mut cursor = 0;
    while let Some(a_start) = lower_body[cursor..].find("<a") {
        let abs_a_start = cursor + a_start;
        cursor = abs_a_start + 2;

        // Find closing tag `>`
        if let Some(a_end) = lower_body[abs_a_start..].find('>') {
            let abs_a_end = abs_a_start + a_end;
            let a_tag_attrs = &lower_body[abs_a_start..abs_a_end];

            // Check if there is href="..."
            if let Some(href_idx) = a_tag_attrs.find("href=") {
                let after_href = &a_tag_attrs[href_idx + 5..].trim_start();
                let quote_char = after_href.chars().next();
                if quote_char == Some('"') || quote_char == Some('\'') {
                    let quote = quote_char.unwrap();
                    let val_start = &after_href[1..];
                    if let Some(val_end_idx) = val_start.find(quote) {
                        let href_val = &val_start[..val_end_idx];
                        if href_val.contains("unsubscribe") 
                            || href_val.contains("opt-out") 
                            || href_val.contains("optout") 
                            || href_val.contains("opt_out") 
                            || href_val.contains("remove") 
                        {
                            return true;
                        }
                    }
                }
            }

            // Find closing `</a>` tag to check text content
            if let Some(close_a) = lower_body[abs_a_end..].find("</a>") {
                let abs_close_a = abs_a_end + close_a;
                let a_text = &lower_body[abs_a_end + 1..abs_close_a];
                if a_text.contains("unsubscribe")
                    || a_text.contains("opt-out")
                    || a_text.contains("optout")
                    || a_text.contains("opt out")
                    || a_text.contains("manage preferences")
                    || a_text.contains("email preferences")
                    || a_text.contains("remove me")
                {
                    return true;
                }
            }
        }
    }

    false
}

/// Appends a 1x1 transparent tracking pixel before the closing `</body>` tag of an HTML email body,
/// or at the end if the tag is not present.
pub fn inject_open_tracking_pixel(html_body: &str, api_url: &str, email_id: &str) -> String {
    let pixel_tag = format!(
        "<img src=\"{}/v1/track/open/{}\" width=\"1\" height=\"1\" style=\"display:none !important; width:1px !important; height:1px !important;\" border=\"0\" alt=\"\" />",
        api_url.trim_end_matches('/'),
        email_id
    );

    if let Some(body_close_idx) = html_body.to_lowercase().rfind("</body>") {
        let mut new_body = String::new();
        new_body.push_str(&html_body[..body_close_idx]);
        new_body.push_str(&pixel_tag);
        new_body.push_str(&html_body[body_close_idx..]);
        new_body
    } else {
        format!("{}{}", html_body, pixel_tag)
    }
}

/// Parses HTML `<a href="...">` links and wraps eligible ones with a click-tracking redirection URL.
pub fn wrap_click_tracking_links(html_body: &str, api_url: &str, email_id: &str) -> String {
    let mut result = String::new();
    let mut remaining = html_body;
    let base_track_url = format!("{}/v1/track/click/{}", api_url.trim_end_matches('/'), email_id);

    while let Some(start_idx) = remaining.to_lowercase().find("<a") {
        result.push_str(&remaining[..start_idx]);
        let tail = &remaining[start_idx..];
        remaining = &tail[2..]; // Advance past `<a`

        if let Some(close_idx) = tail.find('>') {
            let tag_content = &tail[..close_idx + 1];
            remaining = &tail[close_idx + 1..];

            // Parse href attribute
            let mut rewritten_tag = tag_content.to_string();
            let tag_lower = tag_content.to_lowercase();
            if let Some(href_idx) = tag_lower.find("href=") {
                let after_href = &tag_content[href_idx + 5..].trim_start();
                let quote_char = after_href.chars().next();
                if quote_char == Some('"') || quote_char == Some('\'') {
                    let quote = quote_char.unwrap();
                    let val_start = &after_href[1..];
                    if let Some(val_end_idx) = val_start.find(quote) {
                        let href_val = &val_start[..val_end_idx];
                        
                        // Check if we should wrap this link
                        let should_skip = href_val.starts_with('#')
                            || href_val.starts_with("mailto:")
                            || href_val.starts_with("tel:")
                            || href_val.starts_with("javascript:")
                            || href_val.contains("unsubscribe")
                            || href_val.contains("{{unsubscribeurl}}")
                            || href_val.contains("{{manageurl}}");

                        if !should_skip && !href_val.trim().is_empty() {
                            // URL encode the original URL
                            let encoded_url = urlencoding::encode(href_val);
                            let new_href_val = format!("{}?url={}", base_track_url, encoded_url);
                            
                            // Reconstruct the `<a ... href="...">` tag
                            let before_href_val = &tag_content[..href_idx + 5 + (after_href.len() - val_start.len())];
                            let after_href_val = &val_start[val_end_idx..];
                            rewritten_tag = format!("{}{}{}", before_href_val, new_href_val, after_href_val);
                        }
                    }
                }
            }
            result.push_str(&rewritten_tag);
        } else {
            result.push_str("<a");
        }
    }
    result.push_str(remaining);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_render_template() {
        let vars = json!({
            "name": "Alice",
            "data": {
                "firstName": "John",
                "hobbies": ["rust", "coding"]
            }
        });

        assert_eq!(render_template("Hello {{name}}!", &vars), "Hello Alice!");
        assert_eq!(render_template("Hello {{data.firstName}}!", &vars), "Hello John!");
        assert_eq!(render_template("Hello {{missing ?? Guest}}!", &vars), "Hello Guest!");
        assert_eq!(
            render_template("Hobbies: {{data.hobbies}}", &vars),
            "Hobbies: <li>rust</li>\n<li>coding</li>"
        );
    }

    #[test]
    fn test_detect_unsubscribe() {
        assert!(detect_unsubscribe_signal("Hello {{unsubscribeUrl}}"));
        assert!(detect_unsubscribe_signal("<a href=\"/unsubscribe/123\">Link</a>"));
        assert!(detect_unsubscribe_signal("<a href=\"/xyz\">Unsubscribe here</a>"));
        assert!(!detect_unsubscribe_signal("<a href=\"/xyz\">Click me</a>"));
    }

    #[test]
    fn test_inject_pixel() {
        let html = "<html><body>Hello!</body></html>";
        let injected = inject_open_tracking_pixel(html, "http://api.test", "123");
        assert!(injected.contains("<img src=\"http://api.test/v1/track/open/123\""));
        assert!(injected.contains("</body>"));
    }

    #[test]
    fn test_wrap_links() {
        let html = "Click <a href=\"https://google.com\">here</a> or unsubscribe <a href=\"/unsubscribe\">now</a>";
        let wrapped = wrap_click_tracking_links(html, "http://api.test", "123");
        assert!(wrapped.contains("href=\"http://api.test/v1/track/click/123?url=https%3A%2F%2Fgoogle.com\""));
        assert!(wrapped.contains("href=\"/unsubscribe\""));
    }
}
