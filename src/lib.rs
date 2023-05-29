//! Documentation for [Torrust Tracker Index Backend](https://github.com/torrust/torrust-index-backend) API.
//!
//! This is the backend API for [Torrust Tracker Index](https://github.com/torrust/torrust-index).
//!
//! It is written in Rust and uses the actix-web framework. It is designed to be
//! used with by the [Torrust Tracker Index Frontend](https://github.com/torrust/torrust-index-frontend).
//!
//! If you are looking for information on how to use the API, please see the
//! [API v1](crate::web::api::v1) section of the documentation.
pub mod app;
pub mod auth;
pub mod bootstrap;
pub mod cache;
pub mod common;
pub mod config;
pub mod console;
pub mod databases;
pub mod errors;
pub mod mailer;
pub mod models;
pub mod routes;
pub mod services;
pub mod tracker;
pub mod ui;
pub mod upgrades;
pub mod utils;
pub mod web;

trait AsCSV {
    fn as_csv<T>(&self) -> Result<Option<Vec<T>>, ()>
    where
        T: std::str::FromStr;
}

impl<S> AsCSV for Option<S>
where
    S: AsRef<str>,
{
    fn as_csv<T>(&self) -> Result<Option<Vec<T>>, ()>
    where
        T: std::str::FromStr,
    {
        match self {
            Some(ref s) if !s.as_ref().trim().is_empty() => {
                let mut acc = vec![];
                for s in s.as_ref().split(',') {
                    let item = s.trim().parse::<T>().map_err(|_| ())?;
                    acc.push(item);
                }
                if acc.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(acc))
                }
            }
            _ => Ok(None),
        }
    }
}
