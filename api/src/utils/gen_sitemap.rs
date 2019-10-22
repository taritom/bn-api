use errors::{ApplicationError, ApplicationErrorType, BigNeonError};
use sitemap::structs::UrlEntry;
use sitemap::writer::SiteMapWriter;
use std::io::{Cursor, Read};

pub fn create_sitemap(urls: &[String]) -> Result<String, BigNeonError> {
    let mut output = Cursor::new(Vec::new());
    {
        let sitemap_writer = SiteMapWriter::new(&mut output);

        let mut urlwriter = sitemap_writer.start_urlset().map_err(|_e| {
            ApplicationError::new_with_type(
                ApplicationErrorType::Internal,
                "fn create_sitemap: Unable to write urlset".to_string(),
            )
        })?;

        for url in urls.iter() {
            urlwriter
                .url(UrlEntry::builder().loc(url.clone()))
                .map_err(|_e| {
                    ApplicationError::new_with_type(
                        ApplicationErrorType::Internal,
                        format!("fn create_sitemap: Unable to write url, {}", url),
                    )
                })?;
        }
        urlwriter.end().map_err(|_e| {
            ApplicationError::new_with_type(
                ApplicationErrorType::Internal,
                "fn create_sitemap: Unable to write close tags".to_string(),
            )
        })?;
    }
    let mut buffer = String::new();
    output.set_position(0);
    output.read_to_string(&mut buffer)?;
    Ok(buffer)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_sitemap_test() {
        let v = vec![
            "http://github.com".to_string(),
            "http://google.com".to_string(),
            "http://yandex.ru".to_string(),
        ];
        let buffer = create_sitemap(&v).unwrap();
        println!("result: {}", buffer);
        let result = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n  <url>\n    <loc>http://github.com/</loc>\n  </url>\n  <url>\n    <loc>http://google.com/</loc>\n  </url>\n  <url>\n    <loc>http://yandex.ru/</loc>\n  </url>\n</urlset>";
        assert_eq!(buffer, result);
    }
}
