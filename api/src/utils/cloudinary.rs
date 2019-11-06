pub fn optimize_cloudinary(url: &Option<String>) -> Option<String> {
    let size = "f_auto";
    let quality = "low";
    match url {
        Some(url) => {
            //If it is a cloudinary url and it hasn't already been optimized
            if url.contains("res.cloudinary.com") && !url.contains("/q_auto") {
                let insert_after_string = "/image/upload/";
                match url.find(insert_after_string) {
                    Some(index) => {
                        let quality_params = format!("{}/q_auto:{}/", size, quality);
                        let index_to_insert = index + insert_after_string.len();
                        let (first, last) = url.split_at(index_to_insert);
                        return Some(format!("{}{}{}", first, quality_params, last).to_string());
                    }
                    None => Some(url.clone()),
                }
            } else {
                return Some(url.clone());
            }
        }
        None => None,
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_optimize_cloudinary() {
        let original_url =
            "https://res.cloudinary.com/dyro9cwim/image/upload/v1569351595/h66oolnynlnb7m93plog.png".to_string();
        let optimized_url =
            "https://res.cloudinary.com/dyro9cwim/image/upload/f_auto/q_auto:low/v1569351595/h66oolnynlnb7m93plog.png"
                .to_string();
        let generated_optimized_url = optimize_cloudinary(&Some(original_url));
        //Optimized url
        assert_eq!(Some(optimized_url), generated_optimized_url);
        //If None return None
        assert_eq!(None, optimize_cloudinary(&None));
        //If it's not cloudinary, return the original
        assert_eq!(
            Some("https://google.com".to_string()),
            optimize_cloudinary(&Some("https://google.com".to_string()))
        );
    }
}
