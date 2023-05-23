use iced::widget::{button, column, row, scrollable, text, text_input};
use iced::{Color, Element, Length, Sandbox};
use rfd::{FileDialog, MessageDialog};
use std::collections::HashSet;
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
                    if self.image_dir_path.is_none() {
                        let image_dir_path_string =
                            file_path.to_str().unwrap().to_string().replace(".md", "");
                        let image_dir_path = Path::new(&image_dir_path_string);
                        if image_dir_path.exists() && image_dir_path.is_dir() {
                            self.image_dir_path =
                                Some(image_dir_path.to_str().unwrap().to_string());
                            self.fresh_images()
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
                if MessageDialog::new()
                    .set_title("Remove Images")
                    .set_description(
                        &*("Are you sure to remove these images?\n\n".to_string()
                            + remove_images.join("\n").as_str()),
                    )
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
                            "{}\n\n{}",
                            "Remove images successfully!",
                            remove_images.join("\n")
                        ))
                        .set_level(rfd::MessageLevel::Info)
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show();
                    self.fresh_images();
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
                        Element::from(text(image.path.clone()).style(if image.used {
                            Color::from_rgb(0.0, 1.0, 0.0)
                        } else {
                            Color::from_rgb(1.0, 0.0, 0.0)
                        }))
                    })
                    .collect(),
            )
            .width(Length::Fill),
        )
        .height(200);
        let buttons = row![
            button(text("Fresh Images")).on_press(Message::FreshImages),
            button(text("Remove Images")).on_press(Message::RemoveImages),
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

struct Image {
    path: String,
    used: bool,
}

impl App {
    pub fn find_md_images(md_path: &str) -> Vec<String> {
        let file = std::fs::read_to_string(md_path).unwrap();
        let mut images = vec![];
        let re = regex::Regex::new(r"!\[.*?\]\((.*?)\)").unwrap();
        for cap in re.captures_iter(&file) {
            let absolute_path = Path::new(md_path)
                .parent()
                .unwrap()
                .join(cap[1].to_string().replace("/", "\\"));
            images.push(absolute_path.to_str().unwrap().to_string());
        }
        images
    }

    pub fn find_dir_images(dir_path: &str) -> Vec<String> {
        let mut images = vec![];
        let dir = std::fs::read_dir(dir_path).unwrap();
        for entry in dir {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                images.append(&mut Self::find_dir_images(path.to_str().unwrap()));
            } else {
                images.push(path.to_str().unwrap().to_string());
            }
        }
        images
    }

    pub fn fresh_images(&mut self) {
        let dir_images = if self.image_dir_path.is_some() {
            Self::find_dir_images(self.image_dir_path.as_ref().unwrap())
        } else {
            vec![]
        };
        let md_images = if self.md_file_path.is_some() {
            Self::find_md_images(self.md_file_path.as_ref().unwrap())
                .into_iter()
                .collect::<HashSet<_>>()
        } else {
            HashSet::new()
        };
        let mut images = vec![];
        for image in dir_images {
            images.push(Image {
                used: md_images.contains(&image),
                path: image,
            });
        }
        self.images = images;
    }

    pub fn find_remove_images(&mut self) -> Vec<String> {
        let mut images = vec![];
        for image in &self.images {
            if !image.used {
                images.push(image.path.clone());
            }
        }
        images
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_find_images() {
        let images =
            super::App::find_md_images("E:\\study_file\\课程相关\\移动开发技术\\移动开发技术.md");
        println!("{:?}", images);
        let images =
            super::App::find_dir_images("E:\\study_file\\课程相关\\移动开发技术\\移动开发技术");
        println!("{:?}", images);
    }
}
