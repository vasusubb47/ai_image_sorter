use actix_files::NamedFile;
use actix_multipart::form::MultipartForm;
use actix_web::{
    delete, get, post,
    web::{self, ReqData},
    HttpResponse, Responder,
};
use serde_json::json;

use crate::{app_data::AppData, models::image_data::*, utility::jwt_token::Claims};

#[post("/save")]
pub async fn save_image(
    data: web::Data<AppData>,
    req_user: Option<ReqData<Claims>>,
    form: MultipartForm<UploadImage>,
) -> impl Responder {
    /*
      part of the solution was referenced
      from a post on stackoverflow

      post : https://stackoverflow.com/a/75849261/13026811

      refere for more information
    */

    // 10 MB
    const MAX_FILE_SIZE: usize = 1024 * 1024 * 10;

    match form.image.size {
        0 => return HttpResponse::BadRequest().finish(),
        length if length > MAX_FILE_SIZE => {
            return HttpResponse::BadRequest().body(format!(
                "The uploaded file is too large. Maximum size is {} bytes.",
                MAX_FILE_SIZE
            ));
        }
        _ => {}
    }

    let project_id = req_user.unwrap().project_id;

    let img = upload_image(&data.data_path, form.0, project_id).await;

    match img {
        Ok(_) => HttpResponse::Ok().into(),
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}
