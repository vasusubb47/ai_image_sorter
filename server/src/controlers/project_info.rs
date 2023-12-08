use actix_web::{
    delete, get, post,
    web::{self, ReqData},
    HttpResponse, Responder,
};
use serde_json::json;

use crate::{app_data::AppData, models::project_info::*, utility::jwt_token::generate_token};

pub fn project_pre_auth(config: &mut web::ServiceConfig) {
    let scope = web::scope("")
        .service(get_all_project_info)
        .service(create_project)
        .service(login_project);

    config.service(scope);
}

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
    new_project: web::Json<ProjectLoginInfo>,
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

#[post("/login")]
pub async fn login_project(
    data: web::Data<AppData>,
    project_info: web::Json<ProjectLoginInfo>,
) -> impl Responder {
    let project = project_login(&data.data_path, &project_info).await;

    match project {
        Ok(project) => HttpResponse::Ok().body(generate_token(&project)),
        Err(err) => {
            println!("{:#?}", err);
            match err {
                ProjectInfoErrors::ProjectDosentExist => HttpResponse::NotFound().into(),
                ProjectInfoErrors::WrongPassword => HttpResponse::Unauthorized().into(),
                _ => {
                    // should never reach here
                    HttpResponse::InternalServerError().into()
                }
            }
        }
    }
}
