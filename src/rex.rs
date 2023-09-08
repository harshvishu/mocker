use regex::{Error, Regex};

pub fn generate_regex_from_route(route: &str) -> Result<Regex, Error> {
    let mut regex_pattern = String::new();

    for part in route.split('/') {
        if part.starts_with('{') && part.ends_with('}') {
            // Replace {id} with a regex capture group
            regex_pattern.push_str(r"(\w+)");
        } else {
            regex_pattern.push_str(part);
        }
        regex_pattern.push('/');
    }

    // Remove the trailing slash and add anchors to the pattern
    regex_pattern.pop();
    regex_pattern = format!("^{}$", regex_pattern);

    Regex::new(&regex_pattern)
}

pub fn contains_curly_braces(url: &str) -> bool {
    let re = Regex::new(r#"\{[^\}]+\}"#).unwrap();
    re.is_match(url)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_generate_regex_from_route() {
        let routes = vec![
            ("account/v1/user/{id}", "^account/v1/user/(\\w+)$"),
            (
                "account/v1/user/{id}/balance",
                "^account/v1/user/(\\w+)/balance$",
            ),
            ("product/v2/main/{id}", "^product/v2/main/(\\w+)$"),
        ];

        for (route, expected_regex) in routes {
            let regex_pattern = generate_regex_from_route(route).unwrap();
            assert_eq!(regex_pattern.as_str(), expected_regex);
        }
    }

    #[test]
    fn test_routes_regex() {
        let routes = vec![
            ("account/v1/user/12345", "account/v1/user/{id}"),
            (
                "account/v1/user/harsh/balance",
                "account/v1/user/{id}/balance",
            ),
            ("product/v2/main/earphone", "product/v2/main/{id}"),
        ];

        for (url, route) in routes {
            let regex_pattern = generate_regex_from_route(route).unwrap();

            assert!(regex_pattern.is_match(url));
        }
    }
}
