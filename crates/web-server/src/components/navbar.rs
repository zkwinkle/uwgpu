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
    current_uri: &'a Uri,
}

impl Render for Navbar<'_> {
    fn render(&self) -> Markup {
        html! {
            nav class="navbar" {
                ul {
                    li class="header" { a
                        class=@if self.current_uri.path().eq("/")
                            { "active" }
                        href=("/")
                        { ("µwgpu") }}
                    @for link in self.links {
                        li { a
                            class=@if self.current_uri.path().starts_with(link.link)
                                { "active" }
                            href=(link.link)
                            { (link.name) }}
                    }
                }
            }
        }
    }
}

impl Navbar<'_> {
    /// Generate the site's navbar
    pub fn from_uri(uri: &Uri) -> Navbar {
        Navbar {
            links: &TABS,
            current_uri: uri,
        }
    }
}
