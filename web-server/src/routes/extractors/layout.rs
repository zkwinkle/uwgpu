use axum::{
    async_trait,
    extract::{FromRequestParts, OriginalUri},
    http::request::Parts,
};
use maud::{html, Markup, DOCTYPE};

use crate::components::css::STYLESHEET;
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
    #[expect(
        unused,
        reason = "Will probably get used once I have a navbar or something"
    )]
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

            }
            div id="theme-container" {
                div class="container" {
                    header {
                        h1 { "wgpu experiments WIP" }
                    }
                    (content)
                }
            }
        }
    }
}
