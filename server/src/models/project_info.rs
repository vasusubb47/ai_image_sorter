use ::serde::{Deserialize, Serialize};
use chrono::{NaiveDateTime, Utc};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::{fs::File, path::PathBuf};
use uuid::Uuid;

use crate::utility::create_file_write_all;
use crate::utility::genarate_salt;

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
pub struct ProjectCreationInfo {
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
    ProjectAllreadyExists,
}

pub async fn get_all_project_infos(data_path: &str) -> Result<Vec<ProjectInfo>, ProjectInfoErrors> {
    read_global_project_info(data_path).await
}

pub async fn create_project_info(
    data_path: &str,
    project_creation: &ProjectCreationInfo,
) -> Result<ProjectInfo, ProjectInfoErrors> {
    let projects = read_global_project_info(data_path).await;

    if projects.is_err() {
        return Err(projects.err().unwrap());
    }

    let mut projects = projects.ok().unwrap();

    let project = search_project(&projects, &project_creation.project_name);
    if !project.is_none() {
        return Err(ProjectInfoErrors::ProjectAllreadyExists);
    }

    let mut sha = Sha256::new();
    let salt = genarate_salt(64);

    sha.update(project_creation.password.to_owned() + &salt.to_owned());
    let passcode_hash = sha.finalize();

    let passcode_hash = format!("{:X}", passcode_hash) + ":" + &salt;

    let project = ProjectInfo {
        project_id: Uuid::new_v4(),
        project_name: project_creation.project_name.to_owned(),
        password_hash: passcode_hash,
        created_date: Utc::now().naive_utc(),
    };

    projects.append(&mut vec![project.clone()]);
    let golbal_project_json = PathBuf::from(data_path.to_owned() + "project.json");
    create_file_write_all(
        &golbal_project_json,
        serde_json::to_string(&projects).unwrap().as_bytes(),
    );

    Ok(project)
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
