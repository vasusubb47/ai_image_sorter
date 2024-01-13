use ::serde::{Deserialize, Serialize};
use chrono::{NaiveDateTime, Utc};
use image::open as openImage;
use std::io::Read;
use std::{fs::File, path::PathBuf};
use uuid::Uuid;

use crate::utility::encryption::{decrypt_bytes, encrypt_bytes};
use crate::utility::file_utilities::{create_file_write_all, object_to_byte_vec, op_osstr_to_str};
use crate::utility::genarate_salt;

use super::project_info::ProjectInfo;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageData {
    pub image_id: Uuid,
    pub image_name: String,
    pub mime: String,
    pub original_image_name: String,
    pub image_size: u64,
    pub created_date: NaiveDateTime,
    pub is_encrypted: bool,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReqImageData {
    pub image_id: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResponseImageData {
    pub data: Vec<u8>,
    pub metadata: ImageData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadImage {
    pub image_path: String,
    pub image_name: Option<String>,
    pub image_tags: String,
    pub encrypt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TempImage {
    pub temp_file_path: String,
    pub temp_file_size: u64,
    pub temp_image_name: String,
    pub temp_image_mime: String,
    pub image_name: String,
    pub image_tags: Vec<String>,
    pub encrypt: bool,
}

#[derive(Debug)]
pub enum ImageDataError {
    ProjectDosentExists,
    FailedToSaveImage,
    ImageNotFound,
    DecryptionError(String),
}

impl ImageData {
    fn new(temp_image: TempImage) -> Self {
        ImageData {
            image_id: Uuid::new_v4(),
            image_name: temp_image.image_name,
            mime: temp_image.temp_image_mime.to_owned(),
            original_image_name: temp_image.temp_image_name,
            image_size: temp_image.temp_file_size,
            created_date: Utc::now().naive_utc(),
            is_encrypted: temp_image.encrypt,
            tags: temp_image.image_tags,
        }
    }

    pub fn new_vec() -> Vec<Self> {
        vec![]
    }
}

impl TempImage {
    fn from_upload_image(upload_image: UploadImage, input_path: &str) -> Self {
        let binding = PathBuf::from(format!("{}//{}", input_path, upload_image.image_path));
        let temp_img_path = binding.as_path();
        let temp_img_metadata = temp_img_path.metadata().unwrap();

        TempImage {
            temp_file_path: temp_img_path.to_str().unwrap().to_owned(),
            temp_file_size: temp_img_metadata.len(),
            encrypt: upload_image.encrypt,
            temp_image_name: op_osstr_to_str(temp_img_path.file_name()),
            temp_image_mime: op_osstr_to_str(temp_img_path.extension()),
            image_name: upload_image.image_name.unwrap_or(genarate_salt(7)),
            image_tags: upload_image
                .image_tags
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
    image_path: &str,
    temp_img: UploadImage,
    project_id: Uuid,
) -> Result<(), ImageDataError> {
    let images = read_project_images(data_path, &project_id).await;

    if images.is_err() {
        return Err(images.err().unwrap());
    }
    let mut images = images?;

    let project_info = get_project_info(data_path, &project_id).await;
    if project_info.is_err() {
        return Err(project_info.err().unwrap());
    }
    let project_info = project_info?;

    let temp_img = TempImage::from_upload_image(temp_img, image_path);
    let temp_path = temp_img.temp_file_path.to_owned();
    println!("{:#?}", temp_img);
    let img_data = ImageData::new(temp_img);
    println!("{:#?}", img_data);

    let image_path = PathBuf::from(format!(
        "{}\\{}\\{}.{}",
        data_path,
        project_id.to_string(),
        img_data.image_name,
        img_data.mime
    ));

    match image_path.exists() {
        true => Err(ImageDataError::FailedToSaveImage),
        false => {
            println!("{:#?}", image_path);

            let enc_key = if img_data.is_encrypted {
                Some(get_encryption_key(&project_info))
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

pub async fn get_saved_image(
    data_path: &str,
    project_id: &Uuid,
    image_id: &Uuid,
) -> Result<ResponseImageData, ImageDataError> {
    let image_data = get_project_image(data_path, project_id, image_id).await;

    if image_data.is_err() {
        return Err(image_data.err().unwrap());
    }
    let image_data = image_data?;

    let project_info = get_project_info(data_path, project_id).await;

    let mut res_img = ResponseImageData {
        data: Vec::new(),
        metadata: image_data.clone(),
    };

    if project_info.is_err() {
        return Err(project_info.err().unwrap());
    }

    let project_info = project_info?;

    let image_path = format!(
        "{}{}\\{}.{}",
        data_path, project_info.project_id, image_data.image_name, image_data.mime
    );
    println!("image path : {}", image_path);
    let mut file = File::open(image_path).unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    let err = file.read_to_end(&mut buffer);
    println!("{:#?}", err);

    if err.is_err() {
        return Err(ImageDataError::DecryptionError(err.unwrap().to_string()));
    }

    match image_data.is_encrypted {
        false => {
            res_img.data = buffer;
        }
        true => {
            // println!("buffer data : {}", buffer);

            let dec_bytes = decrypt_bytes(
                std::str::from_utf8(&buffer).unwrap(),
                &get_encryption_key(&project_info),
            );

            if dec_bytes.is_err() {
                return Err(ImageDataError::DecryptionError(
                    dec_bytes.unwrap_err().to_string(),
                ));
            }
            res_img.data = dec_bytes.ok().unwrap();
        }
    }

    Ok(res_img)
}

async fn get_project_image(
    data_path: &str,
    project_id: &Uuid,
    image_id: &Uuid,
) -> Result<ImageData, ImageDataError> {
    let project_images = read_project_images(data_path, project_id).await;

    if project_images.is_err() {
        return Err(project_images.err().unwrap());
    }

    let project_imges = project_images?;

    for image_data in project_imges.iter() {
        if image_data.image_id == *image_id {
            return Ok(image_data.clone());
        }
    }

    Err(ImageDataError::ImageNotFound)
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

async fn get_project_info(
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

async fn save_temp_image(temp_path: String, image_path: String, encryption_key: Option<String>) {
    match encryption_key {
        Some(encryption_key) => {
            let mut file = File::open(temp_path).unwrap();
            let mut buffer: Vec<u8> = Vec::new();
            let err = file.read_to_end(&mut buffer);
            println!("{:#?}", err);

            let encrypted_image = encrypt_bytes(buffer, &encryption_key);
            create_file_write_all(&PathBuf::from(&image_path), &encrypted_image);
        }
        None => {
            let temp_img = openImage(temp_path).unwrap().into_rgb8();
            let _ = temp_img.save(image_path.to_owned());
        }
    }

    println!("saved image to {}", image_path);
}

fn get_encryption_key(project_info: &ProjectInfo) -> String {
    project_info.password_hash.split(':').collect::<Vec<&str>>()[1].to_owned()
}
