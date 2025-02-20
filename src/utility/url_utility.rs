


pub fn is_valid_remote_resource(s: &str) -> bool {
        
    is_valid_url(s)
}

fn is_valid_url(s: &str) -> bool {
    match reqwest::Url::parse(s) {
        Ok(_) => true,
        Err(_) => false,
    }
}

