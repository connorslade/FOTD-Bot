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

/// Decode a url encoded string
pub fn decode_url_chars(url: &str) -> String {
    // Convert input to Char array
    let url = url.chars().collect::<Vec<char>>();

    let mut res = String::new();
    let mut i = 0;
    while i < url.len() {
        if url[i] == '%' {
            let mut hex = String::new();
            try_push(&mut hex, url.get(i + 1));
            try_push(&mut hex, url.get(i + 2));
            res.push(u8::from_str_radix(&hex, 16).unwrap_or_default() as char);
            i += 3;
            continue;
        }
        try_push(&mut res, url.get(i));
        i += 1;
    }
    res
}

fn try_push(vec: &mut String, c: Option<&char>) {
    if let Some(c) = c {
        vec.push(*c);
    }
}
