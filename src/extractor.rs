use scraper::{Html, Selector};
use url::Url;

pub fn extract_links(body: &str, base: &Url, allowed_host: &str) -> Vec<Url> {
    let document = Html::parse_document(body);
    let selector = Selector::parse("a[href]").expect("valid CSS selector");

    document
        .select(&selector)
        .filter_map(|el| el.value().attr("href"))
        .filter_map(|href| resolve_url(href, base))
        .filter(|url| is_same_domain(url, allowed_host))
        .map(|mut url| {
            url.set_fragment(None);
            url
        })
        .collect()
}

fn resolve_url(href: &str, base: &Url) -> Option<Url> {
    if href.starts_with("mailto:")
        || href.starts_with("javascript:")
        || href.starts_with("tel:")
        || href.starts_with('#')
    {
        return None;
    }
    base.join(href).ok()
}

fn is_same_domain(url: &Url, allowed_host: &str) -> bool {
    url.host_str()
        .map(|h| h == allowed_host)
        .unwrap_or(false)
}
