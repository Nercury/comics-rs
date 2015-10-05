use std::collections::HashMap;

pub struct Globals {
    js_links: Vec<u8>,
    css_links: Vec<u8>,
    values: HashMap<&'static [u8], String>,
}

impl Globals {
    pub fn new() -> Globals {
        Globals {
            js_links: {
                let mut res = Vec::new();

                for link in Globals::js_links() {
                    res.extend(b"<script src=\"");
                    res.extend(link);
                    Self::apply_cache_buster(&mut res);
                    res.extend(b"\"></script>");
                }

                res
            },
            css_links: {
                let mut res = Vec::new();

                for link in Globals::css_links() {
                    res.extend(br#"<link rel="stylesheet" type="text/css" href=""#.iter());
                    res.extend(link);
                    Self::apply_cache_buster(&mut res);
                    res.extend(br#"" />"#);
                }

                res
            },
            values: HashMap::new(),
        }
    }

    pub fn with(mut self, key: &'static str, value: String) -> Globals {
        self.amend(key, value);

        self
    }

    pub fn amend(&mut self, key: &'static str, value: String) {
        self.values.insert(key.as_bytes(), value);
    }

    pub fn get<'g, 'r>(&'g self, key: &'r [u8]) -> Option<&'g str> {
        match self.values.get(key) {
            Some(v) => Some(&v),
            None => None,
        }
    }

    #[cfg(feature = "prod")]
    fn apply_cache_buster(res: &mut Vec<u8>) {
        use release;

        res.extend(b"?");
        res.extend(release::version().bytes());
    }

    #[cfg(not(feature = "prod"))]
    fn apply_cache_buster(_: &mut Vec<u8>) {
    }

    #[cfg(feature = "prod")]
    fn js_links() -> Vec<&'static [u8]> {
        vec![
            br#"/js/compiled/prod.js"#,
        ]
    }

    #[cfg(not(feature = "prod"))]
    fn js_links() -> Vec<&'static [u8]> {
        vec![
            br#"/js/compiled/require.js"#,
            br#"/js/config.js"#,
        ]
    }

    pub fn get_js_links(&self) -> &[u8] {
        &self.js_links
    }

    #[cfg(feature = "prod")]
    fn css_links() -> Vec<&'static [u8]> {
        vec![
            br#"/css/compiled/prod.css"#,
        ]
    }

    #[cfg(not(feature = "prod"))]
    fn css_links() -> Vec<&'static [u8]> {
        vec![
            br#"/css/compiled/dev.css"#,
            br#"/css/style.css"#,
            br#"/css/comics.css"#,
        ]
    }

    pub fn get_css_links(&self) -> &[u8] {
        &self.css_links
    }
}
