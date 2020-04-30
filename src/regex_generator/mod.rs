use crate::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref PATH_PARAMS_RE: Regex = { Regex::new(r"(?s)(?::([^/]+))|(?:\*)").unwrap() };
}

fn generate_common_regex_str(path: &str) -> crate::Result<(String, Vec<String>)> {
    let mut regex_str = String::with_capacity(path.len());
    let mut param_names: Vec<String> = Vec::new();

    let mut pos: usize = 0;

    for caps in PATH_PARAMS_RE.captures_iter(path) {
        let whole = caps.get(0).unwrap();

        let path_s = &path[pos..whole.start()];
        regex_str += &regex::escape(path_s);

        if whole.as_str() == "*" {
            regex_str += r"(.*)";
            param_names.push("*".to_owned());
        } else {
            regex_str += r"([^/]+)";
            param_names.push(caps.get(1).unwrap().as_str().to_owned());
        }

        pos = whole.end();
    }

    let left_over_path_s = &path[pos..];
    regex_str += &regex::escape(left_over_path_s);

    Ok((regex_str, param_names))
}

pub(crate) fn generate_exact_match_regex(path: &str) -> crate::Result<(Regex, Vec<String>)> {
    let (common_regex_str, params) = generate_common_regex_str(path)?;
    let re_str = format!("{}{}{}", r"(?s)^", common_regex_str, "$");
    let re = Regex::new(re_str.as_str()).wrap()?;
    Ok((re, params))
}

pub(crate) fn generate_prefix_match_regex(path: &str) -> crate::Result<(Regex, Vec<String>)> {
    let (common_regex_str, params) = generate_common_regex_str(path)?;
    let re_str = format!("{}{}", r"(?s)^", common_regex_str);
    let re = Regex::new(re_str.as_str()).wrap()?;
    Ok((re, params))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_common_regex_str_normal() {
        let path = "/";
        let r = generate_common_regex_str(path).unwrap();
        assert_eq!(r, (r"/".to_owned(), Vec::<String>::new()));

        let path = "/api/v1/services/get_ip";
        let r = generate_common_regex_str(path).unwrap();
        assert_eq!(r, (r"/api/v1/services/get_ip".to_owned(), Vec::<String>::new()))
    }

    #[test]
    fn test_generate_common_regex_str_special_character() {
        let path = "/users/user-data/view";
        let r = generate_common_regex_str(path).unwrap();
        assert_eq!(r, (r"/users/user\-data/view".to_owned(), Vec::<String>::new()))
    }

    #[test]
    fn test_generate_common_regex_str_params() {
        let path = "/users/:username/data";
        let r = generate_common_regex_str(path).unwrap();
        assert_eq!(r, (r"/users/([^/]+)/data".to_owned(), vec!["username".to_owned()]));

        let path = "/users/:username/data/:attr/view";
        let r = generate_common_regex_str(path).unwrap();
        assert_eq!(
            r,
            (
                r"/users/([^/]+)/data/([^/]+)/view".to_owned(),
                vec!["username".to_owned(), "attr".to_owned()]
            )
        );

        let path = "/users/:username";
        let r = generate_common_regex_str(path).unwrap();
        assert_eq!(r, (r"/users/([^/]+)".to_owned(), vec!["username".to_owned()]));

        let path = ":username";
        let r = generate_common_regex_str(path).unwrap();
        assert_eq!(r, (r"([^/]+)".to_owned(), vec!["username".to_owned()]));
    }

    #[test]
    fn test_generate_common_regex_str_star_globe() {
        let path = "*";
        let r = generate_common_regex_str(path).unwrap();
        assert_eq!(r, (r"(.+)".to_owned(), vec!["*".to_owned()]));

        let path = "/users/*";
        let r = generate_common_regex_str(path).unwrap();
        assert_eq!(r, (r"/users/(.+)".to_owned(), vec!["*".to_owned()]));

        let path = "/users/*/data";
        let r = generate_common_regex_str(path).unwrap();
        assert_eq!(r, (r"/users/(.+)/data".to_owned(), vec!["*".to_owned()]));

        let path = "/users/*/data/*";
        let r = generate_common_regex_str(path).unwrap();
        assert_eq!(
            r,
            (
                r"/users/(.+)/data/(.+)".to_owned(),
                vec!["*".to_owned(), "*".to_owned()]
            )
        );

        let path = "/users/**";
        let r = generate_common_regex_str(path).unwrap();
        assert_eq!(r, (r"/users/(.+)(.+)".to_owned(), vec!["*".to_owned(), "*".to_owned()]));
    }
}
