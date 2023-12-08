use ::serde::{Deserialize, Serialize};
use chrono::{NaiveDateTime, Utc};
use std::io::Read;
use std::{fs, fs::File, path::PathBuf};
use uuid::Uuid;

use crate::models::image_data::ImageData;
use crate::utility::{file_utilities::*, hash_password, verify_password};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub project_id: Uuid,
    pub project_name: String,
    pub password_hash: String,
    pub created_date: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectLoginInfo {
    pub project_name: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Projects {
    pub project_id: Uuid,
    pub project_name: String,
    pub created_date: NaiveDateTime,
}

#[derive(Debug)]
pub enum ProjectInfoErrors {
    FailedToCreateProjectFolder,
    ProjectAllreadyExists,
    ProjectDosentExist,
    WrongPassword,
}

impl ProjectInfo {
    pub fn new(project_name: &str, password_hash: &str) -> Self {
        ProjectInfo {
            project_id: Uuid::new_v4(),
            project_name: project_name.to_owned(),
            password_hash: password_hash.to_owned(),
            created_date: Utc::now().naive_utc(),
        }
    }
}

pub async fn get_all_project_infos(data_path: &str) -> Result<Vec<ProjectInfo>, ProjectInfoErrors> {
    read_global_project_info(data_path).await
}

pub async fn create_project_info(
    data_path: &str,
    project_creation: &ProjectLoginInfo,
) -> Result<ProjectInfo, ProjectInfoErrors> {
    let info = extract_project_info(data_path, &project_creation.project_name).await;
    if info.is_err() {
        return Err(info.err().unwrap());
    }
    let (mut projects, project) = info.ok().unwrap();

    if !project.is_none() {
        return Err(ProjectInfoErrors::ProjectAllreadyExists);
    }

    let project = ProjectInfo::new(
        &project_creation.project_name,
        &hash_password(&project_creation.password),
    );

    let image_dir = PathBuf::from(data_path.to_owned() + &project.project_id.to_string());
    let project_data =
        PathBuf::from(data_path.to_owned() + &project.project_id.to_string() + "\\project.json");
    let image_data = PathBuf::from(
        data_path.to_owned() + &project.project_id.to_string() + "\\project_images.json",
    );

    let image_dir = fs::create_dir(image_dir);
    if image_dir.is_err() {
        return Err(ProjectInfoErrors::FailedToCreateProjectFolder);
    }

    create_file_write_all(&project_data, object_to_byte_vec(&project).as_slice());
    create_file_write_all(
        &image_data,
        object_to_byte_vec(&ImageData::new_vec()).as_slice(),
    );

    projects.append(&mut vec![project.clone()]);
    let golbal_project_json = PathBuf::from(data_path.to_owned() + "project.json");
    create_file_write_all(
        &golbal_project_json,
        object_to_byte_vec(&projects).as_slice(),
    );

    Ok(project)
}

pub async fn project_login(
    data_path: &str,
    project_login_info: &ProjectLoginInfo,
) -> Result<ProjectInfo, ProjectInfoErrors> {
    let info = extract_project_info(data_path, &project_login_info.project_name).await;
    if info.is_err() {
        return Err(info.err().unwrap());
    }
    let (_, project) = info.ok().unwrap();

    if project.is_none() {
        return Err(ProjectInfoErrors::ProjectDosentExist);
    }

    let project = project.unwrap();

    if !verify_password(&project_login_info.password, &project.password_hash) {
        Err(ProjectInfoErrors::WrongPassword)
    } else {
        Ok(project)
    }
}

async fn extract_project_info(
    data_path: &str,
    project_name: &str,
) -> Result<(Vec<ProjectInfo>, Option<ProjectInfo>), ProjectInfoErrors> {
    let projects = read_global_project_info(data_path).await;

    if projects.is_err() {
        return Err(projects.err().unwrap());
    }

    let projects = projects.ok().unwrap();

    let project = search_project(&projects, project_name);
    Ok((projects, project))
}

async fn read_global_project_info(data_path: &str) -> Result<Vec<ProjectInfo>, ProjectInfoErrors> {
    let golbal_project_json = PathBuf::from(data_path.to_owned() + "project.json");
    if !golbal_project_json.exists() {
        return Ok(vec![]);
    }

    let mut file = File::open(golbal_project_json).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let projects: Vec<ProjectInfo> = serde_json::from_str(&data).unwrap();

    Ok(projects)
}

fn search_project(projects: &Vec<ProjectInfo>, project_name: &str) -> Option<ProjectInfo> {
    for project in projects {
        if project.project_name == project_name {
            return Some(project.to_owned());
        }
    }
    None
}
