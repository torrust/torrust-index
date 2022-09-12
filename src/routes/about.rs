use actix_web::{Responder, web, HttpResponse};
use actix_web::http::StatusCode;

use crate::errors::ServiceResult;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/about")
            .service(web::resource("/license")
                .route(web::get().to(get_license))
            )
    );
}

const LICENSE: &str = r#"
<html>
    <head>
        <title>Licensing</title>
    </head>
    <body style="margin-left: auto;margin-right: auto;max-width: 30em;">
        <h1>Torrust Developers</h1>

        <h2>Licensing</h2>

        <h3>Multiple Licenses</h3>

        <p>This repository has multiple licenses depending on the content type, the date of contributions or stemming from external component licenses that were not developed by any of Torrust team members or Torrust repository contributors.</p>

        <p>The two main applicable license to most of its content are:</p>

        <p>- For Code -- <a href="https://github.com/torrust/torrust-index-backend/blob/main/licensing/agpl-3.0.md">agpl-3.0</a></p>

        <p>- For Media (Images, etc.) -- <a href="https://github.com/torrust/torrust-index-backend/blob/main/licensing/cc-by-sa.md">cc-by-sa</a></p>

        <p>If you want to read more about all the licenses and how they apply please refer to the <a href="https://github.com/torrust/torrust-index-backend/blob/develop/licensing/contributor_agreement_v01.md">contributor agreement</a>.</p>
    </body>
</html>
"#;

pub async fn get_license() -> ServiceResult<impl Responder> {
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(LICENSE)
    )
}
