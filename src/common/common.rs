/// Get the type MMIE content type of a file from its extension
pub fn get_type(path: &str) -> &str {
    match path.split('.').last() {
        Some(ext) => match ext {
            "html" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "png" => "image/png",
            "jpg" => "image/jpeg",
            "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "ico" => "image/x-icon",
            "svg" => "image/svg+xml",
            _ => "application/octet-stream",
        },

        None => "application/octet-stream",
    }
}

/// i dont even know...
fn append_frag(text: &mut String, frag: &mut String) {
    if !frag.is_empty() {
        let encoded = frag
            .chars()
            .collect::<Vec<char>>()
            .chunks(2)
            .map(|ch| u8::from_str_radix(&ch.iter().collect::<String>(), 16).unwrap())
            .collect::<Vec<u8>>();
        text.push_str(&std::str::from_utf8(&encoded).unwrap());
        frag.clear();
    }
}

/// Decode URL encoded strings
pub fn decode_url_chars(text: &str) -> String {
    let mut output = String::new();
    let mut encoded_ch = String::new();
    let mut iter = text.chars();
    while let Some(ch) = iter.next() {
        if ch == '%' {
            encoded_ch.push_str(&format!("{}{}", iter.next().unwrap(), iter.next().unwrap()));
            continue;
        }
        append_frag(&mut output, &mut encoded_ch);
        output.push(ch);
    }
    append_frag(&mut output, &mut encoded_ch);
    output
}

/// Holds key value pairs from Query Strings
pub struct Query {
    data: Vec<[String; 2]>,
}

impl Query {
    /// Create a new Query from a Form POST body
    pub fn from_body(body: String) -> Query {
        let mut data = Vec::new();

        let parts: Vec<&str> = body.split('&').collect();
        for i in parts {
            let sub: Vec<&str> = i.split('=').collect();
            if sub.len() < 2 {
                continue;
            }

            let key: String = sub[0].to_string();
            let value: String = sub[1].to_string();

            data.push([key, value])
        }

        Query { data }
    }

    /// Get a value from a key
    pub fn get(&self, key: &str) -> Option<String> {
        for i in self.data.clone() {
            if i[0] == key {
                return Some(i[1].clone());
            }
        }
        None
    }
}
