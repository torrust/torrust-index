use actix_web::{web, Responder, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::common::WebAppData;
use crate::errors::ServiceResult;
use crate::models::response::OkResponse;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/feedback")
            .service(web::resource("")
                .route(web::post().to(feedback)))
    );
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Feedback {
    pub username: Option<String>,
    pub emil : Option<String>,
    pub description: String
}

pub async fn feedback(req: HttpRequest, payload: web::Json<Feedback>, app_data: WebAppData) -> ServiceResult<impl Responder> {
            let user = app_data.auth.get_user_from_request(&req).await?;

            Ok(HttpResponse::Ok())
      // None => Err(ServiceError::WrongPasswordOrUsername)
}




