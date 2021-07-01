extern crate nfd2;
extern crate image;
extern crate msgbox;
extern crate zip;
extern crate zip_extensions;
extern crate directories_next;

use image::{GenericImage, GenericImageView, Rgba, DynamicImage};
use nfd2::Response;
use msgbox::IconType;
use directories_next::{BaseDirs, UserDirs};

use std::process::exit;
use std::fs;
use std::io::prelude::*;
use std::env;
use std::path::{Path, PathBuf};

const MSGBOX_TITLE: &str = "Minecraft Textures";
const MSGBOX_FOLDER_TEXT: &str = "Please select a minecraft .jar file. These are located in the version folders (ex. 1.17)";
const MSGBOX_IMAGE_TEXT: &str = "Please select the image that you wish to put on the Minecraft textures.";
const MSGBOX_FINISH_TEXT: &str = "Done!";

fn main() {
    let mc_dir = get_mc_dir();
    let pics_dir = get_pics_dir();

    msgbox::create(MSGBOX_TITLE, MSGBOX_FOLDER_TEXT, IconType::Info).unwrap();

    let result_folder = nfd2::open_file_dialog(None, Some(&mc_dir)).unwrap();
    let mut jar_path: String = "".to_string();

    match result_folder {
        Response::Okay(file_path) => { jar_path = file_path.into_os_string().into_string().unwrap() },
        Response::OkayMultiple(_files) => {},
        Response::Cancel => exit(0),
    }

    msgbox::create(MSGBOX_TITLE, MSGBOX_IMAGE_TEXT, IconType::Info).unwrap();

    let result_image = nfd2::open_file_dialog(None, Some(&pics_dir)).unwrap();
    let mut image_path: String = "".to_string();

    match result_image {
        Response::Okay(file_path) => { image_path = file_path.into_os_string().into_string().unwrap() },
        Response::OkayMultiple(_files) => {},
        Response::Cancel => exit(0),
    }

    fs::create_dir_all("out/assets/minecraft/textures/block").unwrap();

    // Extract jar file to out/temp
    let mut jar = fs::File::open(&jar_path).unwrap();
    get_to_textures(&mut jar);

    create_mcmeta();

    overlay("out/temp".to_string(), image_path);

    fs::remove_dir_all("out/temp").unwrap();

    msgbox::create(MSGBOX_TITLE, MSGBOX_FINISH_TEXT, IconType::Info).unwrap();
}

fn overlay(folder_path: String, img_path: String) {
    let mut img = image::open(img_path).unwrap();
    img = img.thumbnail_exact(128, 128);

    for entry in fs::read_dir(folder_path.clone() + "/assets/minecraft/textures/block").unwrap() {
        let entry = entry.unwrap();
        let block_path = entry.path();
        let mut block_file_name = entry.file_name().into_string().unwrap();

        if block_file_name.contains(".mcmeta") {
            continue;
        }

        block_file_name = block_file_name.replace("\"", "");

        let mut block = image::open(block_path).unwrap();
        block = block.resize_exact(128, 128, image::imageops::Nearest);
    
        let mut out = DynamicImage::new_rgba8(128, 128);
    
        for x in 0..128 {
            for y in 0..128 {
                out.put_pixel(x, y, average_pixel(
                    block.get_pixel(x, y),
                    img.get_pixel(x, y)
                ));
            }
        }
    
        out.save(format!("out/assets/minecraft/textures/block/{}", block_file_name)).unwrap();
    }
}

fn average_pixel(block: Rgba<u8>, input: Rgba<u8>) -> Rgba<u8> {
    image::Rgba([
        (block[0] / 4 * 3) + (input[0] / 4),
        (block[1] / 4 * 3) + (input[1] / 4),
        (block[2] / 4 * 3) + (input[2] / 4),
        block[3] // Keeps transparent pixels
    ])
}

fn get_to_textures<R: Read + std::io::Seek>(/*folder_name: &String, */reader: &mut R) {
    let mut zip = zip::ZipArchive::new(reader).unwrap();
    zip.extract("out/temp").unwrap();
}

fn get_mc_dir() -> PathBuf {
    match env::consts::OS {
        "windows" => {
            if let Some(base_dir) = BaseDirs::new() {
                return base_dir.config_dir().join(Path::new(".minecraft\\versions")).to_path_buf();
            }
        },
        "linux" => {
            if let Some(base_dir) = BaseDirs::new() {
                return base_dir.home_dir().join(Path::new(".minecraft/versions")).to_path_buf();
            }
        },
        "macos" => {
            if let Some(base_dir) = BaseDirs::new() {
                return base_dir.config_dir().join(Path::new("minecraft/versions"));
            }
        },
        _ => {}
    }
    PathBuf::new()
}

fn get_pics_dir() -> PathBuf {
    if let Some(user_dir) = UserDirs::new() {
        return user_dir.picture_dir().unwrap().to_path_buf();
    }
    PathBuf::new()
}

fn create_mcmeta() {
    let mut file = fs::File::create("out/pack.mcmeta").unwrap();
    file.write_all(
b"{
    \"pack\": {
        \"pack_format\": 7,
        \"description\": \"Automatically generated pack\"
    }
}"
    ).unwrap();
}