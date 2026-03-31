use iced::Length::{FillPortion, Fill};
use iced::widget::{Column, button, checkbox, column, container, image, row, scrollable, text, toggler, text_input};
use iced::{keyboard, Subscription, Element, Alignment, Length};

use std::collections::HashMap;
use std::ffi::OsString;
use std::{fs};
use std::path::{PathBuf};

use std::os::unix::fs::symlink;

static WALLPAPER_PATH: &str = "images/wallpapers";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ManagerMode {
	SelectFile,
	SetTags,
}
impl Default for ManagerMode {
    fn default() -> Self {
        ManagerMode::SelectFile
    }
}

#[derive(Debug, Clone)]
enum Message {
	#[allow(unused)]
	Pass,

	ToggleSelector(bool),
	ToggleSelectorKey,

	PressFileSelector(String),
	PressNewFilenameSelector(String),
	InputFilename(String),
	ConfirmNameChange,

	ToggleTag(String, bool)
}

#[derive(Default)]
struct WallpapersManager {
	mode: ManagerMode,

	file_name: String,
	selected_file: String,

	all_wallpapers: HashMap<String, (PathBuf, HashMap<String, PathBuf>)>,
	all_categories: HashMap<String, PathBuf>,

	//consts
	wallpapers_dir: PathBuf,
}

impl WallpapersManager {
	pub fn new() -> Self {

		let homedir: PathBuf = match dirs::home_dir() {
			Some(path) => path,
			None => panic!("Could not determine home directory"),
		};
		let wallpapers_dir: PathBuf = homedir.join(WALLPAPER_PATH);

		
		let mut manager = Self {
			mode: ManagerMode::SelectFile,
			file_name: "".to_string(),
			selected_file: "".to_string(),
			all_wallpapers: HashMap::<String, (PathBuf, HashMap<String, PathBuf>)>::new(),
			all_categories: HashMap::<String, PathBuf>::new(),
			wallpapers_dir: wallpapers_dir
		};
		manager.fetch_wallpapers();

		//println!("{:?}", manager.all_wallpapers);

		return manager;
	}

	pub fn view(&self) -> Element<'_, Message> {


		let slider = 
			if self.mode == ManagerMode::SelectFile {
				let mut select_frame:Column<'_, Message>  = column![];

				let mut sorted_filenames: Vec<String> = self.all_wallpapers.keys().cloned().collect();
				sorted_filenames.sort();

				let mut all_tags: HashMap<String, Vec<String>> = HashMap::<String, Vec<String>>::new();
				let mut no_category_tag: Vec<String> = vec![];

				for (category, _) in self.all_categories.iter() {
					all_tags.insert(category.to_owned(), vec![]);
				}

				for filename in sorted_filenames {
					let (_, tags) = self.all_wallpapers.get(&filename).unwrap();
					
					if tags.is_empty() {
						no_category_tag.push(filename.to_owned());
					}

					for (tag_name, _) in tags {
						all_tags.get_mut(tag_name).unwrap().push(filename.to_owned());
					}
				}

				let mut sorted_tags: Vec<String> = all_tags.keys().cloned().collect();
				sorted_tags.sort();

				if !no_category_tag.is_empty() {
					all_tags.insert("no category".to_string(), no_category_tag);
					sorted_tags.insert(0, "no category".to_string());
				}

				for tag in sorted_tags {
					select_frame = select_frame.push(
						text(tag.to_owned()).width(Length::Fill)
					);
					for name in all_tags.get(&tag).unwrap() {
						select_frame = select_frame.push(
						button(text(name.to_owned()))
							.on_press(Message::PressFileSelector(name.to_owned()))
							.width(Length::Fill)
						);
					}
				}

				column![row![scrollable(select_frame)].height(FillPortion(1))]
			} else {
				let mut tags_frame:Column<'_, Message>  = column![];
				let mut names_frame:Column<'_, Message>  = column![];

				//fill tags_frame
				{
					let mut selected_image_tags: Vec<String> = vec![];
					for (tag, _) in self.all_wallpapers.get(&self.selected_file).unwrap().1.clone() {
						selected_image_tags.push(tag);
					}

					let mut all_tags: Vec<String> = self.all_categories.keys().cloned().collect();
					all_tags.sort();

					for tag in all_tags {
						tags_frame = tags_frame.push(
							checkbox(selected_image_tags.iter().any(|f| f == &tag)).label(tag.to_owned()).on_toggle(move |state| Message::ToggleTag(tag.to_owned(), state))
						)
					}
				}

				//fill names_frame
				{
					let extension_selected_file = self.all_wallpapers.get(&self.selected_file).unwrap().0.clone().extension().unwrap().to_string_lossy().into_owned();

					// name: (num, ext, fullname)
					let mut unique_names = HashMap::<String, (u32, String, String)>::new();

					for (filename, (path, _)) in self.all_wallpapers.iter() {

						if !filename.contains(&self.file_name) || filename.starts_with("_") {
							continue;
						}
						
						let stem = path.file_stem().unwrap().to_str().unwrap();
						let extension = path.extension().unwrap().to_string_lossy().into_owned();

						let (name, index_str) = stem.rsplit_once('_').unwrap();
						let index = index_str.parse::<u32>().ok().expect(&format!("Index {} is not integer", index_str));

						if let Some((ind, ex, fname)) = unique_names.get_mut(name) {
							if *ind < index {
								*ind = index;
								*ex = extension;
								*fname = format!("{}_{}.{}", name, index+1, extension_selected_file);
							}
						} else {
							unique_names.insert(name.to_string(), (index, extension, format!("{}_{}.{}", name, index+1, extension_selected_file)));
						}
					}

					let mut names: Vec<String> = unique_names.keys().cloned().collect();
					names.sort();

					for name in names {
						names_frame = names_frame.push(
							button(text(name.to_owned()))
								.on_press(Message::PressNewFilenameSelector(unique_names.get(&name).unwrap().2.clone()))
								.width(Length::Fill)
						)
					}

				}

				column![row![scrollable(tags_frame)].height(FillPortion(1)), row![scrollable(names_frame)].height(FillPortion(1))]
			}.height(Fill);

		let mut img = container(text("")).center_x(Fill).center_y(Fill);
		if self.selected_file != PathBuf::new() {
			img = container(image(self.all_wallpapers.get(&self.selected_file).unwrap().0.clone())).center_x(Fill).center_y(Fill)
		}

		container(
			row![
				column![
					img,
					container(
						row![
							column![
								toggler(self.mode == ManagerMode::SetTags).label("toggle selector mode").on_toggle(Message::ToggleSelector),
							].spacing(10).width(FillPortion(1)),
							column![
								if self.mode == ManagerMode::SetTags {text_input("", &self.file_name).on_input(Message::InputFilename)} else {text_input("", &self.file_name)},
								if self.mode == ManagerMode::SetTags {button(container("Save").align_x(Alignment::Center).align_y(Alignment::Center)).on_press(Message::ConfirmNameChange).width(FillPortion(1))} else {button(container("Save").align_x(Alignment::Center).align_y(Alignment::Center)).width(FillPortion(1))},
							].width(FillPortion(1))
						].spacing(10)
					).align_y(iced::alignment::Vertical::Bottom).height(Length::Shrink)
				].width(FillPortion(4)),
				slider.width(FillPortion(1))
			].spacing(10)
		).padding(10).into()
	}

	pub fn update(&mut self, message: Message) {
		match message {
			Message::Pass => {}
			Message::ToggleSelector(state) => {
				if self.selected_file != "" {
					if state {self.mode = ManagerMode::SetTags} else {self.mode = ManagerMode::SelectFile}
				}
			}
			Message::ToggleSelectorKey => {
				if self.selected_file != "" {
					if self.mode == ManagerMode::SetTags {
						self.mode = ManagerMode::SelectFile;
					} else {
						self.mode = ManagerMode::SetTags;
					}
				} 
			}

			Message::PressFileSelector(button) => {
				self.selected_file = button.to_owned();
				self.file_name = button.to_owned();
			}
			Message::PressNewFilenameSelector(file_name) => {
				self.file_name = file_name;
			}
			Message::InputFilename(file_name) => {
				self.file_name = file_name;
			}
			Message::ConfirmNameChange => {
				let old_path: PathBuf = self.all_wallpapers.get(&self.selected_file).unwrap().0.clone();
				let new_path: PathBuf = old_path.with_file_name(&self.file_name);
				let old_conf_path = old_path.with_extension("conf");
				let new_conf_path = new_path.with_extension("conf");

				let old_selected_file = self.selected_file.to_owned();
				self.selected_file = self.file_name.to_owned();
				fs::rename(old_path, new_path).expect("Cannot rename file");
				if old_conf_path.exists() {
					fs::rename(old_conf_path, new_conf_path).expect("Cannot rename file");
				}

				let categories: HashMap<String, PathBuf> = self.all_wallpapers.get(&old_selected_file).unwrap().1.clone();
				self.fetch_wallpapers();
				for (category, _) in categories {
					self.update(Message::ToggleTag(category, true));
				}
			}

			Message::ToggleTag(tag_name, state) => {
				if state {
					let selected_file_path = self.all_wallpapers.get(&self.selected_file).unwrap().0.clone();
					let tag_path = self.all_categories.get(&tag_name).unwrap();

					let _ = symlink(selected_file_path, tag_path.join(&self.selected_file)).expect("symlink cannot create");
				} else {
					let path: &PathBuf = self.all_wallpapers.get(&self.selected_file).unwrap().1.get(&tag_name).unwrap();
					let _ = fs::remove_file(path);
				}
			}
		}
		self.fetch_wallpapers();
	}

	fn subscription(&self) -> Subscription<Message> {
		keyboard::listen().filter_map(|event| {
			match event {
				keyboard::Event::KeyPressed { key, .. } => {
					match key {
						keyboard::Key::Character(ref c) if c == "i" || c == "I" => {
							Some(Message::ToggleSelectorKey)
						}
						_ => None,
					}
				}
				_ => None,
			}
		})
	}
}

impl WallpapersManager {
	fn fetch_wallpapers(&mut self) {
		let mut wallpapers: HashMap<String, (PathBuf, HashMap<String, PathBuf>)> = HashMap::<String, (PathBuf, HashMap<String, PathBuf>)>::new();
		let mut categories: HashMap<String, PathBuf> = HashMap::<String, PathBuf>::new();

		for entry in fs::read_dir(self.wallpapers_dir.join("all")).unwrap().flatten() {

			let file_name = entry.file_name();
			if file_name.to_string_lossy().starts_with('.') || file_name.to_string_lossy().ends_with(".conf") {
				continue;
			}
			let file_type = match entry.file_type() {
				Ok(t) => t,
				Err(_) => continue,
			};
			if !file_type.is_file() {
				continue;
			}
			let file_name: String = file_name.to_string_lossy().into_owned();
			let file_path: PathBuf = entry.path();

			wallpapers.insert(file_name, (file_path, HashMap::<String, PathBuf>::new()));
		}

		for tag in fs::read_dir(&self.wallpapers_dir).unwrap().flatten() {
			let tag_name: String = tag.file_name().to_string_lossy().into_owned();
			if tag_name.starts_with('.') || tag_name == "all" {
				continue;
			}
			let file_type: fs::FileType = match tag.file_type() {
				Ok(t) => t,
				Err(_) => continue,
			};
			if !file_type.is_dir() {
				continue;
			}
			categories.insert(tag_name.to_owned(), tag.path());

			for wallpaper in fs::read_dir(tag.path()).unwrap().flatten() {
				let file_name: OsString = wallpaper.file_name();
				if file_name.to_string_lossy().starts_with('.') {
					continue;
				}
				let file_type = match wallpaper.file_type() {
					Ok(t) => t,
					Err(_) => continue,
				};
				if !file_type.is_symlink() {
					continue;
				}
				
				//symlinc moderation
				let target = match fs::read_link(&wallpaper.path()) {
					Ok(p) => p,
					Err(_) => {
						let _ = fs::remove_file(&wallpaper.path());
						continue;
					}
				};
				let target_abs = if target.is_absolute() {
					target
				} else {
					wallpaper.path()
						.parent()
						.unwrap()
						.join(target)
				};
				if !target_abs.exists() {
					let _ = fs::remove_file(&wallpaper.path());
					continue;
				}
				if target_abs.file_name() != Some(&file_name) {
					let _ = fs::remove_file(&wallpaper.path());
					continue;
				}

				match wallpapers.get_mut(&file_name.to_string_lossy().into_owned()) {
					Some(wallpaper_data) => {
						wallpaper_data.1.insert(tag_name.to_owned(), wallpaper.path());
					},
					None => {}
				}
			}
		}
		self.all_categories = categories;
		self.all_wallpapers = wallpapers;
	}
}

fn main() -> iced::Result {
	iced::application(WallpapersManager::new, WallpapersManager::update, WallpapersManager::view).subscription(WallpapersManager::subscription).run()
}