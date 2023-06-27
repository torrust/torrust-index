//! API context: `proxy`.
//!
//! This context contains the API routes for the proxy service.
//!
//! The torrent descriptions can contain images. These images are proxied
//! through the backend to:
//!
//! - Prevent leaking the user's IP address.
//! - Avoid storing images on the server.
//!
//! The proxy service is a simple cache that stores the images in memory.
//!
//! **NOTICE:** For now, it only supports PNG images.
//!
//! **NOTICE:** The proxy service is not intended to be used as a general
//! purpose proxy. It is only intended to be used for the images in the
//! torrent descriptions.
//!
//! **NOTICE:** Ununauthorized users can't see images. They will get an image
//! with the text "Sign in to see image" instead.
//!
//! # Example
//!
//! The PNG image:
//!
//! <https://raw.githubusercontent.com/torrust/torrust-index-backend/develop/docs/media/torrust_logo.png>
//!
//! The percent encoded image URL:
//!
//! ```text
//! https%3A%2F%2Fraw.githubusercontent.com%2Ftorrust%2Ftorrust-index-backend%2Fdevelop%2Fdocs%2Fmedia%2Ftorrust_logo.png
//! ```
//!
//! For unauthenticated clients:
//!
//! ```bash
//! curl \
//!   --header "cache-control: no-cache" \
//!   --header "pragma: no-cache" \
//!   --output mandelbrotset.jpg \
//!   http://0.0.0.0:3000/v1/proxy/image/https%3A%2F%2Fraw.githubusercontent.com%2Ftorrust%2Ftorrust-index-backend%2Fdevelop%2Fdocs%2Fmedia%2Ftorrust_logo.png
//! ```
//!
//! You will receive an image with the text "Sign in to see image" instead.
//!
//! For authenticated clients:
//!
//! ```bash
//! curl \
//!   --header "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI" \
//!   --header "cache-control: no-cache" \
//!   --header "pragma: no-cache" \
//!   --output mandelbrotset.jpg \
//!   http://0.0.0.0:3000/v1/proxy/image/https%3A%2F%2Fraw.githubusercontent.com%2Ftorrust%2Ftorrust-index-backend%2Fdevelop%2Fdocs%2Fmedia%2Ftorrust_logo.png
//! ```
pub mod handlers;
pub mod responses;
pub mod routes;
