use actix_web::{
    get, post,
    web::{self, ReqData},
    HttpResponse, Responder,
};

use crate::{app_data::AppData, models::image_data::*, utility::jwt_token::Claims};

pub fn image_routes(config: &mut web::ServiceConfig) {
    let scope = web::scope("")
        .service(save_image)
        .service(get_image)
        .service(get_project_info);

    config.service(scope);
}

#[post("/save")]
pub async fn save_image(
    data: web::Data<AppData>,
    req_user: Option<ReqData<Claims>>,
    form: web::Json<UploadImage>,
) -> impl Responder {
    /*
      part of the solution was referenced
      from a post on stackoverflow

      post : https://stackoverflow.com/a/75849261/13026811

      refere for more information
    */

    // 10 MB
    // const MAX_FILE_SIZE: usize = 1024 * 1024 * 10;

    // match form.image.size {
    //     0 => return HttpResponse::BadRequest().finish(),
    //     length if length > MAX_FILE_SIZE => {
    //         return HttpResponse::BadRequest().body(format!(
    //             "The uploaded file is too large. Maximum size is {} bytes.",
    //             MAX_FILE_SIZE
    //         ));
    //     }
    //     _ => {}
    // }

    let project_id = req_user.unwrap().project_id;

    println!("{:#?}", form.0);

    let img = upload_image(&data.data_path, &data.input_path, form.0, project_id).await;

    match img {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}

#[get("/get")]
pub async fn get_image(
    data: web::Data<AppData>,
    req_user: Option<ReqData<Claims>>,
    image_req: web::Json<ReqImageData>,
) -> HttpResponse {
    let project_id = req_user.unwrap().project_id;

    let project_images = get_saved_image(&data.data_path, &project_id, &image_req.0.image_id).await;

    match project_images {
        Ok(images) => HttpResponse::Ok().body(images.data),
        Err(err) => HttpResponse::NotFound().body(format!("{:?}", err)),
    }
}

#[get("/info")]
pub async fn get_project_info(
    data: web::Data<AppData>,
    req_user: Option<ReqData<Claims>>,
) -> HttpResponse {
    let project_id = req_user.unwrap().project_id;

    HttpResponse::Ok().body("")
}
