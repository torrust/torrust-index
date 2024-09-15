use serde::{Deserialize, Serialize};

/// Information displayed to the user in the website.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Website {
    /// The name of the website.
    #[serde(default = "Website::default_name")]
    pub name: String,

    /// The demo settings when the app runs in `demo` mode.
    #[serde(default = "Website::default_demo")]
    pub demo: Option<Demo>,

    /// The legal information.
    #[serde(default = "Website::default_terms")]
    pub terms: Terms,
}

impl Default for Website {
    fn default() -> Self {
        Self {
            name: Self::default_name(),
            demo: Self::default_demo(),
            terms: Self::default_terms(),
        }
    }
}

impl Website {
    fn default_name() -> String {
        "Torrust".to_string()
    }

    fn default_demo() -> Option<Demo> {
        None
    }

    fn default_terms() -> Terms {
        Terms::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Demo {
    /// The fixed message to show when the index is running in demo mode.
    #[serde(default = "Demo::default_warning")]
    pub warning: String,
}

impl Demo {
    fn default_warning() -> String {
        "⚠️ Please be aware: This demo resets all data weekly. Torrents not complying with our Usage Policies will be removed immediately without notice. We encourage the responsible use of this software in compliance with all legal requirements.".to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Terms {
    /// The terms page info.
    #[serde(default = "Terms::default_page")]
    pub page: TermsPage,

    /// The upload agreement info.
    #[serde(default = "Terms::default_upload")]
    pub upload: TermsUpload,
}

impl Terms {
    fn default_page() -> TermsPage {
        TermsPage::default()
    }

    fn default_upload() -> TermsUpload {
        TermsUpload::default()
    }
}

impl Default for Terms {
    fn default() -> Self {
        Self {
            page: Self::default_page(),
            upload: Self::default_upload(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TermsPage {
    /// The terms page title.
    #[serde(default = "TermsPage::default_title")]
    pub title: String,

    /// The terms page content.
    #[serde(default = "TermsPage::default_content")]
    pub content: Markdown,
}

impl TermsPage {
    fn default_title() -> String {
        "Usage Policies and Content Restrictions".to_string()
    }

    fn default_content() -> Markdown {
        Markdown::new(
            r"
# Usage Policies and Content Restrictions

Our software is designed to support the distribution of legal, authorized content only. Users may only upload or share files that fall under the following categories:

- **Open-Source Licenses:** Content licensed under recognized open-source licenses, allowing for free distribution and modification.
- **Creative Commons Licenses:** Content released under Creative Commons licenses that permit sharing and distribution.
- **Public Domain:** Content that is free of copyright restrictions and available for public use.

**Prohibited Content:** Any content that infringes copyright, is subject to copyright protection, or is illegal under applicable laws is strictly prohibited. This includes but is not limited to copyrighted movies, music, software, books, and any other media.

**Enforcement:** We reserve the right to remove any content that does not comply with these policies without notice. We may also take additional steps, including reporting violations to the relevant authorities, if necessary.

",
        )
    }
}

impl Default for TermsPage {
    fn default() -> Self {
        Self {
            title: Self::default_title(),
            content: Self::default_content(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TermsUpload {
    /// The terms page content.
    #[serde(default = "TermsUpload::default_content_upload_agreement")]
    pub content_upload_agreement: Markdown,
}

impl TermsUpload {
    fn default_content_upload_agreement() -> Markdown {
        Markdown::new("I confirm that the content I am uploading is authorized, and I have read and agree to the terms.")
    }
}

impl Default for TermsUpload {
    fn default() -> Self {
        Self {
            content_upload_agreement: Self::default_content_upload_agreement(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Markdown(String);

impl Markdown {
    #[must_use]
    pub fn new(content: &str) -> Self {
        Self(content.to_owned())
    }

    #[must_use]
    pub fn source(&self) -> String {
        self.0.clone()
    }
}
