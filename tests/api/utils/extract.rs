pub fn links(str: &str) -> Vec<linkify::Link> {
    linkify::LinkFinder::new().links(str)
        .filter(|link| *link.kind() == linkify::LinkKind::Url)
        .collect()
}
