//! Templates for "about" static pages.

#[derive(Default)]
pub struct Service;

impl Service {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Returns the html with the about page.
    ///
    /// # Errors
    ///
    /// This method is infallible but returns `Result` to keep the
    /// handler signature uniform.
    #[must_use]
    pub fn get_about_page(&self) -> Result<String, std::convert::Infallible> {
        let html = r#"
    <html>
        <head>
            <title>About</title>
        </head>
        <body style="margin-left: auto;margin-right: auto;max-width: 30em;">
            <h1>Torrust Index</h1>

            <h2>About</h2>

            <p>Hi! This is a running <a href="https://github.com/torrust/torrust-index">torrust-index</a>.</p>
        </body>
        <footer style="padding: 1.25em 0;border-top: dotted 1px;">
            <a href="./about/license">license</a>
        </footer>
    </html>
"#;

        Ok(html.to_string())
    }

    /// Returns the html with the license page.
    ///
    /// # Errors
    ///
    /// This method is infallible but returns `Result` to keep the
    /// handler signature uniform.
    #[must_use]
    pub fn get_license_page(&self) -> Result<String, std::convert::Infallible> {
        let html = r#"
        <html>
            <head>
                <title>Licensing</title>
            </head>
            <body style="margin-left: auto;margin-right: auto;max-width: 30em;">
                <h1>Torrust Index</h1>
    
                <h2>Licensing</h2>
    
                <h3>Multiple Licenses</h3>
    
                <p>This repository has multiple licenses depending on the content type, the date of contributions or stemming from external component licenses that were not developed by any of Torrust team members or Torrust repository contributors.</p>
    
                <p>The two main applicable license to most of its content are:</p>
    
                <p>- For Code -- <a href="https://github.com/torrust/torrust-index/blob/main/licensing/agpl-3.0.md">agpl-3.0</a></p>
    
                <p>- For Media (Images, etc.) -- <a href="https://github.com/torrust/torrust-index/blob/main/licensing/cc-by-sa.md">cc-by-sa</a></p>
    
                <p>If you want to read more about all the licenses and how they apply please refer to the <a href="https://github.com/torrust/torrust-index/blob/develop/licensing/contributor_agreement_v01.md">contributor agreement</a>.</p>
            </body>
            <footer style="padding: 1.25em 0;border-top: dotted 1px;">
                <a href="../about">about</a>
            </footer>
        </html>
    "#;

        Ok(html.to_string())
    }
}
