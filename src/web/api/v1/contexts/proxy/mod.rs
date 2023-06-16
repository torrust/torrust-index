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
//! **NOTICE:** The proxy service is not intended to be used as a general
//! purpose proxy. It is only intended to be used for the images in the
//! torrent descriptions.
//!
//! **NOTICE:** Ununauthorized users can't see images. They will get an image
//! with the text "Sign in to see image" instead.
//!
//! # Example
//!
//! For unauthenticated clients:
//!
//! ```bash
//! curl \
//!   --header "cache-control: no-cache" \
//!   --header "pragma: no-cache" \
//!   --output mandelbrotset.jpg \
//!   http://0.0.0.0:3000/v1/proxy/image/https%3A%2F%2Fupload.wikimedia.org%2Fwikipedia%2Fcommons%2Fthumb%2F2%2F21%2FMandel_zoom_00_mandelbrot_set.jpg%2F1280px-Mandel_zoom_00_mandelbrot_set.jpg
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
//!   http://0.0.0.0:3000/v1/proxy/image/https%3A%2F%2Fupload.wikimedia.org%2Fwikipedia%2Fcommons%2Fthumb%2F2%2F21%2FMandel_zoom_00_mandelbrot_set.jpg%2F1280px-Mandel_zoom_00_mandelbrot_set.jpg
//! ```
pub mod handlers;
pub mod responses;
pub mod routes;
