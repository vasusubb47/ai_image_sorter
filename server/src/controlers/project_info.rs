use actix_web::{
    delete, get, post,
    web::{self, ReqData},
    HttpResponse, Responder,
};
use serde_json::json;

use crate::{
    app_data::AppData,
    models::project_info::{
        create_project_info, get_all_project_infos, ProjectCreationInfo, ProjectInfo,
    },
    utility::jwt_token::Claims,
};

#[get("/")]
pub async fn get_all_project_info(data: web::Data<AppData>) -> impl Responder {
    let projects = get_all_project_infos(&data.data_path).await;

    match projects {
        Ok(projects) => HttpResponse::Ok().json(json!(projects)),
        Err(err) => {
            println!("{:#?}", err);
            HttpResponse::InternalServerError().into()
        }
    }
}

#[post("/create")]
pub async fn create_project(
    data: web::Data<AppData>,
    new_project: web::Json<ProjectCreationInfo>,
) -> impl Responder {
    let project = create_project_info(&data.data_path, &new_project).await;

    match project {
        Ok(project) => HttpResponse::Ok().json(json!(project)),
        Err(err) => {
            println!("{:#?}", err);
            HttpResponse::InternalServerError().into()
        }
    }
}
