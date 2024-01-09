use iced::widget::{button, column, row, scrollable, text, text_input};
use iced::{Color, Element, Length, Sandbox};
use regex::Captures;
use rfd::{FileDialog, MessageDialog};
use std::path::Path;

pub struct App {
    // md file path
    md_file_path: Option<String>,
    // image directory path
    image_dir_path: Option<String>,
    // images used in the md file
    images: Vec<Image>,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    // select a md file
    SelectMdFile,
    // select a directory which save the images used in the md file
    SelectImageDir,
    // fresh the images used in the md file
    FreshImages,
    // remove the images which are not used in the md file
    RemoveImages,
    // rename the images with 1, 2, 3, ...
    RenameImages,
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        App {
            md_file_path: None,
            image_dir_path: None,
            images: vec![],
        }
    }

    fn title(&self) -> String {
        String::from("Markdown Image Cleaner")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::SelectMdFile => {
                let files = FileDialog::new()
                    .add_filter("markdown", &["md"])
                    .set_directory("/")
                    .pick_file();
                if let Some(file_path) = files {
                    self.md_file_path = Some(file_path.to_str().unwrap().to_string());
                    // find if the same directory have a same name image directory
                    // if have, set the image directory path
                    // xx/xx/xx.md -> xx/xx/xx
                    let file_path_string =
                        file_path.to_str().unwrap().to_string().replace(".md", "");
                    let image_dir_path_strings =
                        vec![file_path_string.clone(), file_path_string + ".assets"];
                    for image_dir_path_string in image_dir_path_strings {
                        let image_dir_path = Path::new(&image_dir_path_string);
                        if image_dir_path.exists() && image_dir_path.is_dir() {
                            self.image_dir_path =
                                Some(image_dir_path.to_str().unwrap().to_string());
                            self.fresh_images();
                            break;
                        }
                    }
                }
            }
            Message::SelectImageDir => {
                let files = FileDialog::new()
                    .add_filter("directory", &[""])
                    .set_directory("/")
                    .pick_folder();
                if let Some(file) = files {
                    self.image_dir_path = Some(file.to_str().unwrap().to_string());
                    self.fresh_images();
                }
            }
            Message::FreshImages => {
                self.fresh_images();
            }
            Message::RemoveImages => {
                let remove_images = self.find_remove_images();
                if remove_images.is_empty() {
                    MessageDialog::new()
                        .set_title("Remove Images")
                        .set_description("No images need to be removed!")
                        .set_level(rfd::MessageLevel::Info)
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show();
                    return;
                }
                let remove_images_string = Self::get_description(remove_images.clone());
                if MessageDialog::new()
                    .set_title("Remove Images")
                    .set_description(&*format!(
                        "Are you sure to remove these {} images?\n\n{}",
                        remove_images.len(),
                        remove_images_string
                    ))
                    .set_level(rfd::MessageLevel::Warning)
                    .set_buttons(rfd::MessageButtons::YesNo)
                    .show()
                {
                    for image in &remove_images {
                        std::fs::remove_file(image).unwrap();
                    }
                    MessageDialog::new()
                        .set_title("Remove Images")
                        .set_description(&*format!(
                            "Remove {} images successfully!\n\n{}",
                            remove_images.len(),
                            remove_images_string
                        ))
                        .set_level(rfd::MessageLevel::Info)
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show();
                    self.fresh_images();
                }
            }
            Message::RenameImages => {
                // rename the images with 1.xxx, 2.xxx, 3.xxx, ...
                // from back to front to avoid the offset change
                // 1. remove the images which are not used in the md file
                // 2. rename the images
                let rename_images = self
                    .images
                    .iter()
                    .filter(|image| image.used)
                    .collect::<Vec<&Image>>();
                if rename_images.is_empty() {
                    MessageDialog::new()
                        .set_title("Rename Images")
                        .set_description("No images need to be renamed!")
                        .set_level(rfd::MessageLevel::Info)
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show();
                    return;
                }
                let rename_images = rename_images
                    .into_iter()
                    .enumerate()
                    .map(|(idx, image)| (idx + 1, image))
                    .collect::<Vec<(usize, &Image)>>();

                let rename_images_str_vec = rename_images
                    .iter()
                    .map(|(idx, image)| {
                        let path = Path::new(&image.abs_path);
                        let extension = path.extension().unwrap().to_str().unwrap();
                        let file_name = path.file_name().unwrap().to_str().unwrap();
                        format!("{} -> {}", file_name, format!("{}.{}", idx, extension))
                    })
                    .collect::<Vec<String>>();

                let rename_images_str = Self::get_description(rename_images_str_vec);

                if MessageDialog::new()
                    .set_title("Rename Images")
                    .set_description(&*format!(
                        "Are you sure to rename these {} images?\n\n{}",
                        rename_images.len(),
                        rename_images_str
                    ))
                    .set_level(rfd::MessageLevel::Warning)
                    .set_buttons(rfd::MessageButtons::YesNo)
                    .show()
                {
                    let new_file_path = self.rename_images(&rename_images);
                    MessageDialog::new()
                        .set_title("Rename Images")
                        .set_description(&*format!(
                            "Rename {} images successfully!\n\n{}\n\nNew md file save to:\n{}",
                            rename_images.len(),
                            rename_images_str,
                            new_file_path
                        ))
                        .set_level(rfd::MessageLevel::Info)
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show();
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let file_path_label = text("Md File Path:");
        let file_path_select = row![
            text_input(
                "Select a Md File",
                self.md_file_path.as_ref().unwrap_or(&String::from(""))
            )
            .width(Length::Fill),
            button(text("Select Md File")).on_press(Message::SelectMdFile),
        ]
        .spacing(10)
        .width(Length::Fill)
        .padding([0, 0, 10, 0]);
        let image_dir_path_label = text("Image Directory Path:");
        let image_dir_path_select = row![
            text_input(
                "Select a Directory",
                self.image_dir_path.as_ref().unwrap_or(&String::from(""))
            )
            .width(Length::Fill),
            button(text("Select Image Directory")).on_press(Message::SelectImageDir),
        ]
        .spacing(10)
        .width(Length::Fill)
        .padding([0, 0, 10, 0]);
        let images_label = text("Images:");
        let images = scrollable(
            column(
                self.images
                    .iter()
                    .map(|image| {
                        Element::from(text(image.abs_path.clone()).style(if image.used {
                            Color::from_rgb(0.0, 1.0, 0.0)
                        } else {
                            Color::from_rgb(1.0, 0.0, 0.0)
                        }))
                    })
                    .collect(),
            )
            .width(Length::Fill),
        )
        .height(Length::Fill);
        let buttons = row![
            button(text("Fresh Images")).on_press(Message::FreshImages),
            button(text("Remove Images")).on_press(Message::RemoveImages),
            button(text("Rename Images")).on_press(Message::RenameImages)
        ]
        .spacing(20)
        .width(Length::Fill)
        .padding([10, 0, 0, 0]);

        column![
            file_path_label,
            file_path_select,
            image_dir_path_label,
            image_dir_path_select,
            images_label,
            images,
            buttons,
        ]
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

#[derive(Debug)]
struct Image {
    abs_path: String,
    raw_path: Option<String>,
    used: bool,
}

impl App {
    // return (absolute_path, raw_path, offset)
    fn find_md_images(md_path: &str) -> Vec<(String, String)> {
        let file = std::fs::read_to_string(md_path).unwrap();
        let mut images = vec![];
        let md_format = regex::Regex::new(r"!\[.*?\]\((.*?)\)").unwrap();
        let html_format = regex::Regex::new(r#"<img[^>]*?src\s*=\s*["']?([^"'>]+)[^>]*>"#).unwrap();

        for cap in md_format
            .captures_iter(&file)
            .chain(html_format.captures_iter(&file))
            .collect::<Vec<Captures>>()
        {
            let match_str = cap.get(1).unwrap();
            let absolute_path = Path::new(md_path)
                .parent()
                .unwrap()
                .join(cap[1].to_string().replace("/", "\\"));
            images.push((
                absolute_path.to_str().unwrap().to_string(),
                match_str.as_str().to_string(),
            ));
        }

        images
    }

    // return absolute_path
    fn find_dir_images(dir_path: &str) -> Vec<String> {
        let mut images = vec![];
        let dir = std::fs::read_dir(dir_path).unwrap();
        for entry in dir {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                images.append(&mut Self::find_dir_images(path.to_str().unwrap()));
            } else {
                // check if the file is a image
                let extension = path.extension().unwrap().to_str().unwrap().to_lowercase();
                if !["png", "jpg", "jpeg", "gif", "webp"].contains(&extension.as_str()) {
                    continue;
                }
                images.push(path.to_str().unwrap().to_string());
            }
        }
        images
    }

    fn fresh_images(&mut self) {
        let dir_images = if self.image_dir_path.is_some() {
            Self::find_dir_images(self.image_dir_path.as_ref().unwrap())
        } else {
            vec![]
        };
        let md_images = if self.md_file_path.is_some() {
            Self::find_md_images(self.md_file_path.as_ref().unwrap())
        } else {
            vec![]
        };
        let md_images_map = md_images
            .iter()
            .map(|(abs_path, raw_path)| (abs_path, raw_path))
            .collect::<std::collections::HashMap<&String, &String>>();
        let mut images = vec![];
        for image in dir_images {
            images.push(Image {
                used: md_images_map.contains_key(&image),
                abs_path: image.clone(),
                raw_path: md_images_map.get(&image).map(|s| s.to_string()),
            });
        }
        self.images = images;
    }

    fn find_remove_images(&mut self) -> Vec<String> {
        let mut images = vec![];
        for image in &self.images {
            if !image.used {
                images.push(image.abs_path.clone());
            }
        }
        images
    }

    fn get_description(content: Vec<String>) -> String {
        if content.len() > 10 {
            content[0..5].join("\n") + "\n...\n" + content[content.len() - 5..].join("\n").as_str()
        } else {
            content.join("\n")
        }
    }

    fn rename_images(&self, images: &Vec<(usize, &Image)>) -> String {
        let md_file_path = self.md_file_path.as_ref().unwrap();
        let mut md_file_content = std::fs::read_to_string(md_file_path).unwrap();
        for (idx, image) in images.iter().rev() {
            let path = Path::new(&image.abs_path);
            let extension = path.extension().unwrap().to_str().unwrap();
            let raw_file_name = path.file_name().unwrap().to_str().unwrap();
            let new_file_name = format!("{}.{}", idx, extension);
            let new_path = path.parent().unwrap().join(&new_file_name);
            let new_raw_path = image
                .raw_path
                .as_ref()
                .unwrap()
                .replace(raw_file_name, &new_file_name);
            std::fs::copy(path, new_path).unwrap();
            md_file_content =
                md_file_content.replace(image.raw_path.as_ref().unwrap(), &new_raw_path);
        }

        // write to a new file like xx_new.md
        let new_md_file_path = Path::new(md_file_path);
        let file_name = new_md_file_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .split(".")
            .collect::<Vec<&str>>()[0]
            .to_string();
        let new_md_file_path = new_md_file_path
            .parent()
            .unwrap()
            .join(format!("{}_new.md", file_name))
            .to_str()
            .unwrap()
            .to_string();
        std::fs::write(&new_md_file_path, md_file_content).unwrap();
        new_md_file_path
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_regex_images() {
        let md_format = regex::Regex::new(r"!\[.*?\]\((.*?)\)").unwrap();
        let html_format = regex::Regex::new(r#"<img[^>]*?src\s*=\s*["']?([^"'>]+)[^>]*>"#).unwrap();
        let text = r#"
# Test Regex Images
![image](./image.png)
<img src="./image.png" />
<img src="Android XXXX.assets/image-20230523174346232.png" alt="image-20230523174346232" style="zoom: 67%;" />
"#;
        for cap in md_format.captures_iter(text) {
            println!("{}", &cap[1]);
        }
        for cap in html_format.captures_iter(text) {
            println!("{}", &cap[1]);
        }
    }
}
