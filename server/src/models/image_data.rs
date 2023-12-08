use ::serde::{Deserialize, Serialize};
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use chrono::{NaiveDateTime, Utc};
use image::{open as openImage, GenericImage, GenericImageView, ImageBuffer};
use std::io::Read;
use std::{fs::File, path::PathBuf};
use uuid::Uuid;

use crate::utility::file_utilities::{
    create_file_write_all, get_file_type_from_mime, object_to_byte_vec,
};
use crate::utility::genarate_salt;

use super::project_info::ProjectInfo;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageData {
    pub image_id: Uuid,
    pub image_name: String,
    pub original_image_name: String,
    pub image_size: u32,
    pub created_date: NaiveDateTime,
    pub is_encrypted: bool,
    pub tags: Vec<String>,
}

#[derive(MultipartForm, Debug)]
pub struct UploadImage {
    pub image: TempFile,
    pub image_name: Option<Text<String>>,
    pub image_tags: Text<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TempImage {
    pub temp_file_path: String,
    pub temp_file_size: u32,
    pub temp_image_name: String,
    pub temp_image_mime: String,
    pub image_name: String,
    pub image_tags: Vec<String>,
}

pub enum ImageDataError {
    ProjectDosentExists,
    FailedToSaveImage,
}

impl ImageData {
    fn new(temp_image: TempImage, is_encrypted: bool) -> Self {
        ImageData {
            image_id: Uuid::new_v4(),
            image_name: temp_image.image_name,
            original_image_name: temp_image.temp_image_name,
            image_size: temp_image.temp_file_size,
            created_date: Utc::now().naive_utc(),
            is_encrypted,
            tags: temp_image.image_tags,
        }
    }

    pub fn new_vec() -> Vec<Self> {
        vec![]
    }
}

impl TempImage {
    fn from_upload_image(upload_image: UploadImage) -> Self {
        TempImage {
            temp_file_path: upload_image.image.file.path().to_str().unwrap().to_owned(),
            temp_file_size: upload_image.image.size as u32,
            temp_image_name: upload_image.image.file_name.unwrap(),
            temp_image_mime: get_file_type_from_mime(
                &upload_image.image.content_type.unwrap().to_string(),
            ),
            image_name: upload_image.image_name.unwrap_or(Text(genarate_salt(7))).0,
            image_tags: upload_image
                .image_tags
                .0
                .split(';')
                .collect::<Vec<&str>>()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

pub async fn upload_image(
    data_path: &str,
    temp_img: UploadImage,
    project_id: Uuid,
) -> Result<(), ImageDataError> {
    let images = read_project_images(data_path, &project_id).await;

    if images.is_err() {
        return Err(images.err().unwrap());
    }
    let mut images = images.ok().unwrap();

    let project_info = read_project_info(data_path, &project_id).await;
    if project_info.is_err() {
        return Err(project_info.err().unwrap());
    }
    let project_info = project_info.ok().unwrap();

    let temp_img = TempImage::from_upload_image(temp_img);
    let mime = temp_img.temp_image_mime.to_owned();
    let temp_path = temp_img.temp_file_path.to_owned();
    println!("{:#?}", temp_img);
    let img_data = ImageData::new(temp_img, true);
    println!("{:#?}", img_data);

    let image_path = PathBuf::from(
        data_path.to_owned()
            + "\\"
            + &project_id.to_string()
            + "\\"
            + &img_data.image_name
            + "."
            + &mime,
    );

    match image_path.exists() {
        true => Err(ImageDataError::FailedToSaveImage),
        false => {
            println!("{:#?}", image_path);

            let enc_key = if img_data.is_encrypted {
                Some(project_info.password_hash.split(':').collect::<Vec<&str>>()[1].to_owned())
            } else {
                None
            };
            save_temp_image(temp_path, image_path.to_str().unwrap().to_owned(), enc_key).await;

            images.append(&mut vec![img_data.clone()]);
            let project_path = PathBuf::from(
                data_path.to_owned() + "\\" + &project_id.to_string() + "\\project_images.json",
            );
            create_file_write_all(&project_path, object_to_byte_vec(&images).as_slice());

            Ok(())
        }
    }
}

async fn read_project_images(
    data_path: &str,
    project_id: &Uuid,
) -> Result<Vec<ImageData>, ImageDataError> {
    let project_path = PathBuf::from(
        data_path.to_owned() + "\\" + &project_id.to_string() + "\\project_images.json",
    );
    if !project_path.exists() {
        return Err(ImageDataError::ProjectDosentExists);
    }

    let mut file = File::open(project_path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let images: Vec<ImageData> = serde_json::from_str(&data).unwrap();
    Ok(images)
}

async fn read_project_info(
    data_path: &str,
    project_id: &Uuid,
) -> Result<ProjectInfo, ImageDataError> {
    let project_path =
        PathBuf::from(data_path.to_owned() + "\\" + &project_id.to_string() + "\\project.json");
    if !project_path.exists() {
        return Err(ImageDataError::ProjectDosentExists);
    }

    let mut file = File::open(project_path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let project_info: ProjectInfo = serde_json::from_str(&data).unwrap();
    Ok(project_info)
}

async fn save_temp_image(temp_path: String, image_path: String, _encryption_key: Option<String>) {
    let temp_img = openImage(temp_path).unwrap().into_rgb8();
    let _ = temp_img.save(image_path.to_owned());
    println!("saved image to {}", image_path);
}
