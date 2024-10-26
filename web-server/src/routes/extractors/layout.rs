use std::ops::Deref;

use axum::{
    async_trait,
    extract::{FromRequestParts, OriginalUri},
    http::request::Parts,
};
use maud::{html, Markup, DOCTYPE};

use crate::components::{css::STYLESHEET, navbar::Navbar};
//use crate::components::{navbar::Navbar, theme_selector::ThemeSelector};

/// Defines the base layout of a page that will wrap its contents with container
/// divs, headers, footers.
///
/// Usage:
/// ```ignore
/// async fn endpoint(layout: Layout) -> Markup {
///    layout.render(html! { "Hello, World!" })
/// }
/// ```
pub struct Layout {
    uri: OriginalUri,
}

#[async_trait]
impl<S> FromRequestParts<S> for Layout
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            uri: OriginalUri::from_request_parts(parts, state).await.unwrap(),
        })
    }
}

impl Layout {
    pub fn render(self, content: Markup) -> Markup {
        html! {
            (DOCTYPE)
            head {
                ( STYLESHEET )
                meta name="viewport" content="width=device-width, initial-scale=1";
                script src="/public/htmx.min.js" {}
            }
            div id="theme-container" class="light" {
                div class="container" {
                    (Navbar::from_uri(self.uri.deref()))
                    div class="content-container" {
                        (content)
                    }
                }
            }
        }
    }
}
