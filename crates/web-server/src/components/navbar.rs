use axum::http::Uri;
use maud::{html, Markup, Render};

use crate::api_types::MicrobenchmarkKind::*;

/// Tab / Link in the navbar
struct NavLink {
    name: &'static str,
    link: &'static str,
}

const TABS: &'static [NavLink] = &[
    NavLink {
        name: "Matrix Multiplication",
        link: Matmul.path(),
    },
    NavLink {
        name: "Buffer To Buffer",
        link: BufferToBuffer.path(),
    },
];

/// A navbar with certain navlinks
pub struct Navbar<'a> {
    links: &'static [NavLink],
    public_server_url: &'a str,
    current_uri: &'a Uri,
}

impl Render for Navbar<'_> {
    fn render(&self) -> Markup {
        let url = |path: &str| format!("{}{}", self.public_server_url, path);
        html! {
            nav class="navbar" {
                ul {
                    li class="header" { a
                        class=@if self.current_uri.path().eq("/")
                            { "active" }
                        href=(url("/"))
                        { ("µwgpu") }}
                    @for tab in self.links {
                        li { a
                            class=@if self.current_uri.path().starts_with(tab.link)
                                { "active" }
                            href=(url(tab.link))
                            { (tab.name) }}
                    }
                }
            }
        }
    }
}

impl Navbar<'_> {
    /// Generate the site's navbar
    pub fn from_urls<'a>(
        public_server_url: &'a str,
        current_uri: &'a Uri,
    ) -> Navbar<'a> {
        Navbar {
            links: &TABS,
            public_server_url,
            current_uri,
        }
    }
}
