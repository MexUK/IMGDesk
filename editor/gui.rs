extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;
extern crate winapi;

use nwd::NwgUi;
use nwg::NativeUi;

use nwg::*;
//use nwd::*;

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::thread;

use num_format::{Locale, ToFormattedString};

use super::utility as utility;
use super::format::entry::Entry as Entry;







pub struct Gui
{
	pub app: Rc<basic_app_ui::BasicAppUi>,
	pub close_child_window: bool,
	pub text_input_window_data_in: (String, String, String),
	pub text_input_window_data_out: String
}

impl Default for Gui
{
	fn default() -> Self
	{
		Self
		{
			app: BasicApp::build_ui(Default::default()).expect("Failed to build UI"),
			close_child_window: false,
			text_input_window_data_in: (String::from(""), String::from(""), String::from("")),
			text_input_window_data_out: String::from("")
		}
	}
}

impl Gui
{
	// credits window
	pub fn show_credits_window(&mut self)
	{
		self.close_child_window = false;
		
		thread::spawn(move || {
			let mut app = CreditsWindow::build_ui(
			{
				Default::default()
			}).expect("Failed to build UI");
			
			use nwg::Event as E;
			let app_cloned = app.clone();
			
			let handle_events = move |evt:nwg::Event, _evt_data:nwg::EventData, handle:nwg::ControlHandle|
			{
				match evt
				{
					E::OnButtonClick =>
						if &handle == &app_cloned.ok_button
						{
							app_cloned.window.close();
							crate::editor::get_editor().gui.close_child_window = true;
						}
					E::OnInit =>
						{
							if &handle == &app_cloned.window
							{
								unsafe
								{
									nwg::win32::window_helper::set_window_text(handle.hwnd().unwrap(), &format!("Credits"));
									app_cloned.text.set_text("
IMG Desk by Mex


Thanks to:

App Icon by icons8.com
https://icons8.com/icon/PIlh7vxjC0J7/table

GTA IV IMG Format Encryption Key

Rust Crates:
dirs 2.0
native-windows-gui (modified version of 1.0-prerelease)
num-format 0.4.0
rust-crypto 0.2.36
winapi 0.3.8
winres 0.1.11

");
								}
							}
						}
					E::OnWindowClose =>
						{
							if &handle == &app_cloned.window
							{
								crate::editor::get_editor().gui.close_child_window = true;
							}
						}
					_ => {}
				}
			};
			
			// bind events
			nwg::full_bind_event_handler(&app.window.handle, handle_events);
			
            nwg::dispatch_thread_events();
        });
		
		loop
		{
			if self.close_child_window
			{
				break;
			}
			
			std::thread::sleep(std::time::Duration::from_millis(100));
		};
	}
	
	// text input window
	pub fn show_text_input_window(&mut self, title: &'static str, text: &str, text_box_text: &str) -> String
	{
		self.close_child_window = false;
		self.text_input_window_data_in = (title.to_string(), text.to_string(), text_box_text.to_string());
		self.text_input_window_data_out = String::from("");
		
		thread::spawn(move || {
			let mut app = TextInputWindow::build_ui(
			{
				Default::default()
			}).expect("Failed to build UI");
			
			use nwg::Event as E;
			let app_cloned = app.clone();
			
			let handle_events = move |evt:nwg::Event, _evt_data:nwg::EventData, handle:nwg::ControlHandle|
			{
				match evt
				{
					E::OnButtonClick =>
						if &handle == &app_cloned.ok_button
						{
							crate::editor::get_editor().gui.text_input_window_data_out = app_cloned.text_box.text().clone();
							app_cloned.window.close();
							crate::editor::get_editor().gui.close_child_window = true;
						}
						else if &handle == &app_cloned.cancel_button
						{
							app_cloned.window.close();
							crate::editor::get_editor().gui.close_child_window = true;
						}
					E::OnInit =>
						{
							if &handle == &app_cloned.window
							{
								unsafe
								{
									nwg::win32::window_helper::set_window_text(handle.hwnd().unwrap(), title);
								}
								app_cloned.text.set_text(&crate::editor::get_editor().gui.text_input_window_data_in.1);
								app_cloned.text_box.set_text(&crate::editor::get_editor().gui.text_input_window_data_in.2);
							}
						}
					E::OnWindowClose =>
						{
							if &handle == &app_cloned.window
							{
								crate::editor::get_editor().gui.close_child_window = true;
							}
						}
					_ => {}
				}
			};
			
			// bind events
			nwg::full_bind_event_handler(&app.window.handle, handle_events);
			
            nwg::dispatch_thread_events();
        });
		
		loop
		{
			if self.close_child_window
			{
				break;
			}
			
			std::thread::sleep(std::time::Duration::from_millis(100));
		};
		
		self.text_input_window_data_out.clone()
	}
	
	pub fn message(&mut self, params: &nwg::MessageParams) -> MessageChoice
	{
		::nwg::message(&self.app.window, &params)
	}
	
	// load
	pub fn load(&mut self)
	{
		self.app.main_entries.set_full_select_row();
	
		self.app.main_entries.insert_column(InsertListViewColumn
		{
			index: Some(0),
			fmt: None,
			width: Some(100),
			text: "Index".to_string()
		});
		self.app.main_entries.insert_column(InsertListViewColumn
		{
			index: Some(1),
			fmt: None,
			width: Some(160),
			text: "Name".to_string()
		});
		self.app.main_entries.insert_column(InsertListViewColumn
		{
			index: Some(2),
			fmt: None,
			width: Some(100),
			text: "Offset".to_string()
		});
		self.app.main_entries.insert_column(InsertListViewColumn
		{
			index: Some(3),
			fmt: None,
			width: Some(100),
			text: "Size".to_string()
		});
	}
	
	// img versions for combo box
	fn load_img_versions_for_combo(&mut self)
	{
		self.app.img_version_combo.push(String::from("Version 1"));
		self.app.img_version_combo.push(String::from("Version 2"));
		self.app.img_version_combo.push(String::from("Version 3 Encrypted"));
		self.app.img_version_combo.push(String::from("Version 3 Unencrypted"));
	}
	
	fn reset_img_versions_for_combo(&mut self)
	{
		let j = self.app.img_version_combo.collection().len();
		for i in 0..j
		{
			self.app.img_version_combo.remove(0);
		}
	}
	
	// active img version for combo
	pub fn set_active_img_version_for_combo(&mut self)
	{
		let option_index = match crate::editor::get_editor().format.img_version
		{
			1 => 0,
			2 => 1,
			3 =>
			{
				if crate::editor::get_editor().format.img_encrypted { 2 }
				else { 3 }
			},
			_ => { 0 }
		};
		self.app.img_version_combo.set_selection(Some(option_index));
	}
	
	fn reset_active_img_version_for_combo(&mut self)
	{
	}
	
	// file open/closed events
	pub fn on_file_open(&mut self)
	{
		self.app.save.set_enabled(true);
		self.app.close.set_enabled(true);
		self.app.add.set_enabled(true);
		self.app.remove.set_enabled(true);
		//self.app.find.set_enabled(true);
		self.app.replace.set_enabled(true);
		self.app.select_all.set_enabled(true);
		self.app.select_inverse.set_enabled(true);
		self.app.recalculate_offsets.set_enabled(true);
		self.app.img_version_combo.set_enabled(true);
		self.app.undo.set_enabled(false);
		self.app.redo.set_enabled(false);
		
		self.update_path();
		self.update_img_version();
		self.load_img_versions_for_combo();
		self.set_active_img_version_for_combo();
	}
	
	pub fn on_no_file_open(&mut self)
	{
		self.app.save.set_enabled(false);
		self.app.close.set_enabled(false);
		self.app.add.set_enabled(false);
		self.app.remove.set_enabled(false);
		//self.app.find.set_enabled(false);
		self.app.replace.set_enabled(false);
		self.app.select_all.set_enabled(false);
		self.app.select_inverse.set_enabled(false);
		self.app.recalculate_offsets.set_enabled(false);
		self.app.img_version_combo.set_enabled(false);
		self.app.undo.set_enabled(false);
		self.app.redo.set_enabled(false);
		
		self.reset_path();
		self.reset_img_version();
		self.reset_img_versions_for_combo();
		self.reset_active_img_version_for_combo();
	}
	
	// entry events
	pub fn on_entry_change(&mut self, entry: &Entry)
	{
		self.update_entry(entry);
	}
	
	// path
	pub fn update_path(&mut self)
	{
		if !super::get_editor().is_open()
		{
			self.reset_path();
		}
		else
		{
			let mut text : String = super::get_editor().format.img_path_in.replace("\\", "/").clone();
			if super::get_editor().edited
			{
				text.push_str(" *");
			}
			self.app.path.set_text(&text);
		}
	}
	
	fn reset_path(&mut self)
	{
		self.app.path.set_text("(path)");
	}
	
	// buttons
	pub fn update_undo_redo_buttons(&mut self)
	{
		let undo_enabled : bool = super::get_editor().action_history.index >= 0;
		let redo_enabled : bool = super::get_editor().action_history.index < ((super::get_editor().action_history.actions.len() - 1) as i64);
		
		self.app.undo.set_enabled(undo_enabled);
		self.app.redo.set_enabled(redo_enabled);
	}
	
	// add entries
	pub fn readd_entries_to_list(&mut self)
	{
		if !super::get_editor().is_open()
		{
			return;
		}
		
		let include_text : String = self.app.include_search_box.text().to_string().trim().to_uppercase();
		let exclude_text : String = self.app.exclude_search_box.text().to_string().trim().to_uppercase();
		
		let has_include_text = !include_text.is_empty();
		let has_exclude_text = !exclude_text.is_empty();
		
		self.clear_list();
		
		let j = super::get_editor().format.entries.len();
		for i in 0..j
		{
			let entry = super::get_editor().format.entries[i].clone();
			
			if self.can_entry_be_shown(
				&entry,
				has_include_text,
				has_exclude_text,
				&include_text,
				&exclude_text
			)
			{
				self.add_entry(&entry);
			}
		}
		
		super::get_editor().recalculate_selected_entry_count();
		
		self.update_selected_entry_count();
		self.update_shown_entry_count();
		
		super::get_editor().update_entry_extension_counts();
	}
	
	pub fn update_entry_indices(&mut self)
	{
		if !super::get_editor().is_open()
		{
			return;
		}
		
		let include_text : String = self.app.include_search_box.text().to_string().trim().to_uppercase();
		let exclude_text : String = self.app.exclude_search_box.text().to_string().trim().to_uppercase();
		
		let has_include_text = !include_text.is_empty();
		let has_exclude_text = !exclude_text.is_empty();
		
		let j = super::get_editor().format.entries.len();
		for i in 0..j
		{
			let entry = super::get_editor().format.entries[i].clone();
			
			if self.can_entry_be_shown(
				&entry,
				has_include_text,
				has_exclude_text,
				&include_text,
				&exclude_text
			)
			{
				let index = (entry.index + 1).to_formatted_string(&Locale::en);
				self.app.main_entries.insert_sub_item(InsertListViewItem{index:Some(entry.index as i32), sub_item:Some(0), text:index.to_string()});
			}
		}
	}
	
	pub fn update_entry_offsets(&mut self)
	{
		if !super::get_editor().is_open()
		{
			return;
		}
		
		let include_text : String = self.app.include_search_box.text().to_string().trim().to_uppercase();
		let exclude_text : String = self.app.exclude_search_box.text().to_string().trim().to_uppercase();
		
		let has_include_text = !include_text.is_empty();
		let has_exclude_text = !exclude_text.is_empty();
		
		let j = super::get_editor().format.entries.len();
		for i in 0..j
		{
			let entry = super::get_editor().format.entries[i].clone();
			
			if self.can_entry_be_shown(
				&entry,
				has_include_text,
				has_exclude_text,
				&include_text,
				&exclude_text
			)
			{
				let offset = entry.offset_out.to_formatted_string(&Locale::en);
				self.app.main_entries.insert_sub_item(InsertListViewItem{index:Some(entry.index as i32), sub_item:Some(2), text:offset.to_string()});
			}
		}
	}
	
	pub fn add_entry(&mut self, entry: &Entry)
	{
		self.add_entry_at(entry, -1)
	}
	
	pub fn add_entry_at(&mut self, entry: &Entry, entry_index: i32)
	{
		let row_index = if entry_index == -1
		{
			self.app.main_entries.len() as u32
		}
		else
		{
			entry_index as u32
		};
		
		let index = (row_index + 1).to_formatted_string(&Locale::en);
		let name = unsafe { super::vendor::str_from_u8_nul_utf8_unchecked(&entry.name) };
		let offset = entry.offset_out.to_formatted_string(&Locale::en);
		let size = entry.size.to_formatted_string(&Locale::en);
		
		self.app.main_entries.insert_item(InsertListViewItem{index:Some(row_index as i32), sub_item:Some(0), text:index.to_string()});
		self.app.main_entries.insert_sub_item(InsertListViewItem{index:Some(row_index as i32), sub_item:Some(1), text:name.to_string()});
		self.app.main_entries.insert_sub_item(InsertListViewItem{index:Some(row_index as i32), sub_item:Some(2), text:offset.to_string()});
		self.app.main_entries.insert_sub_item(InsertListViewItem{index:Some(row_index as i32), sub_item:Some(3), text:size.to_string()});
	}
	
	pub fn can_entry_be_shown(&mut self, entry: &Entry, has_include_text: bool, has_exclude_text: bool, include_text: &String, exclude_text: &String) -> bool
	{
		let name2 = unsafe { super::vendor::str_from_u8_nul_utf8_unchecked(&entry.name) };
		let name3 = name2.to_uppercase();
		
		if (!has_include_text || name3.contains(include_text)) && (!has_exclude_text || !name3.contains(exclude_text))
		{
			return true;
		}
		else
		{
			return false;
		}
	}
	
	pub fn check_to_add_entry(&mut self, entry: &Entry)
	{
		self.check_to_add_entry_at(entry, -1)
	}
	
	pub fn check_to_add_entry_at(&mut self, entry: &Entry, entry_index: i32)
	{
		let include_text : String = self.app.include_search_box.text().to_string().trim().to_uppercase();
		let exclude_text : String = self.app.exclude_search_box.text().to_string().trim().to_uppercase();
		
		let has_include_text = !include_text.is_empty();
		let has_exclude_text = !exclude_text.is_empty();
		
		if self.can_entry_be_shown(
			&entry,
			has_include_text,
			has_exclude_text,
			&include_text,
			&exclude_text
		)
		{
			if entry_index == -1
			{
				self.add_entry(&entry);
			}
			else
			{
				self.add_entry_at(&entry, entry_index);
			}
		}
	}
	
	// remove entries
	pub fn clear_list(&mut self)
	{
		self.app.main_entries.clear();
	}
	
	// update entries
	pub fn update_entry(&mut self, entry: &Entry)
	{
		let row = entry.index;
		
		let index = (entry.index + 1).to_formatted_string(&Locale::en);
		let name = unsafe { super::vendor::str_from_u8_nul_utf8_unchecked(&entry.name) };
		let offset = entry.offset_out.to_formatted_string(&Locale::en);
		let size = entry.size.to_formatted_string(&Locale::en);
		
		self.app.main_entries.insert_sub_item(InsertListViewItem{index:Some(row as i32), sub_item:Some(0), text:index.to_string()});
		self.app.main_entries.insert_sub_item(InsertListViewItem{index:Some(row as i32), sub_item:Some(1), text:name.to_string()});
		self.app.main_entries.insert_sub_item(InsertListViewItem{index:Some(row as i32), sub_item:Some(2), text:offset.to_string()});
		self.app.main_entries.insert_sub_item(InsertListViewItem{index:Some(row as i32), sub_item:Some(3), text:size.to_string()});
	}
	
	pub fn update_entries_past_entry(&mut self, entry: &Entry)
	{
		let start_index : u64 = (entry.index + 1) as u64;
		let end_index_ex : u64 = super::get_editor().format.entries.len() as u64;
		
		for i in start_index..end_index_ex
		{
			self.update_entry(&super::get_editor().format.entries[i as usize].clone());
		}
	}
	
	// entry count
	pub fn update_entry_count(&mut self)
	{
		self.app.entry_count.set_text((super::get_editor().format.entries.len().to_formatted_string(&Locale::en)+" entries").as_str());
	}
	
	pub fn reset_entry_count(&mut self)
	{
		self.app.entry_count.set_text("0 entries");
	}
	
	// selected entry count
	pub fn update_selected_entry_count(&mut self)
	{
		self.app.selected_entry_count.set_text(&format!("{} selected entries ({}%)", super::get_editor().selected_entry_count.to_formatted_string(&Locale::en), utility::get_percent(super::get_editor().selected_entry_count as i32, super::get_editor().format.entries.len() as i32)));
	}
	
	pub fn reset_selected_entry_count(&mut self)
	{
		super::get_editor().selected_entry_count = 0;
		self.update_entry_count();
	}
	
	// shown entry count
	pub fn update_shown_entry_count(&mut self)
	{
		self.app.showing_entry_count.set_text(&format!("Showing {} entries ({}%)", self.app.main_entries.len().to_formatted_string(&Locale::en), utility::get_percent(self.app.main_entries.len() as i32, super::get_editor().format.entries.len() as i32)));
	}
	
	// img version
	pub fn reset_img_version(&mut self)
	{
		self.app.img_version.set_text(&format!(""));
	}
	
	pub fn update_img_version(&mut self)
	{
		if !super::get_editor().is_open()
		{
			self.reset_img_version();
			return;
		}
		
		let img_version = super::get_editor().format.img_version;
		let img_encrypted = super::get_editor().format.img_encrypted;
		
		let img_version_text : String = self.get_img_version_text(img_version, img_encrypted);
		self.app.img_version.set_text(&format!("IMG Version {}", img_version_text.to_owned()));
	}
	
	pub fn get_img_version_text(&mut self, img_version: u8, img_encrypted: bool) -> String
	{
		let img_encrypted_text = if img_version == 3
		{
			if img_encrypted
			{
				" (Encrypted)"
			}
			else
			{
				" (Unencrypted)"
			}
		}
		else
		{
			""
		};
		return format!("{}{}", img_version, img_encrypted_text)
	}
}

#[derive(Default, NwgUi)]
pub struct BasicApp
{
	#[nwg_resource(family: "Verdana", size: 12)]
	pub main_font: nwg::Font,
	
    #[nwg_control(size: (700, 700), position: (100, 100), title: super::super::WINDOW_TITLE_BASE, flags: "WINDOW|VISIBLE|MINIMIZE_BOX")]
    pub window: nwg::Window,
	
	#[nwg_control(text: "New", size: (60, 25), position: (10, 10), font: Some(&data.main_font))]
    pub new: nwg::Button,
	
	#[nwg_control(text: "Open", size: (60, 25), position: (75, 10), font: Some(&data.main_font))]
    pub open: nwg::Button,
	
	#[nwg_control(text: "Save", size: (60, 25), position: (140, 10), font: Some(&data.main_font))]
    pub save: nwg::Button,
	
	#[nwg_control(text: "Close", size: (60, 25), position: (205, 10), font: Some(&data.main_font))]
    pub close: nwg::Button,
	
	
	
	
	#[nwg_control(text: "Add", size: (60, 25), position: (285, 10), font: Some(&data.main_font))]
    pub add: nwg::Button,
	
	#[nwg_control(text: "Remove", size: (60, 25), position: (350, 10), font: Some(&data.main_font))]
    pub remove: nwg::Button,
	
	#[nwg_control(text: "Replace", size: (60, 25), position: (415, 10), font: Some(&data.main_font))]
    pub replace: nwg::Button,
	
	#[nwg_control(text: "Export", size: (60, 25), position: (480, 10), font: Some(&data.main_font))]
    pub export: nwg::Button,
	
	
	
	
	#[nwg_control(text: "Undo", size: (60, 25), position: (560, 10), font: Some(&data.main_font))]
    pub undo: nwg::Button,
	
	#[nwg_control(text: "Redo", size: (60, 25), position: (625, 10), font: Some(&data.main_font))]
    pub redo: nwg::Button,
	
	
	
	
	
	
	
	#[nwg_control(size: (150, 25), position: (520, 85), font: Some(&data.main_font))]
    pub img_version_combo: nwg::ComboBox<String>,
	
	#[nwg_control(text: "Select All", size: (70, 25), position: (520, 120), font: Some(&data.main_font))]
    pub select_all: nwg::Button,
	
	#[nwg_control(text: "Select Inv.", size: (70, 25), position: (600, 120), font: Some(&data.main_font))]
    pub select_inverse: nwg::Button,
	
	#[nwg_control(text: "Rename", size: (70, 25), position: (520, 155), font: Some(&data.main_font))]
    pub rename: nwg::Button,
	
	#[nwg_control(text: "Move", size: (70, 25), position: (600, 155), font: Some(&data.main_font))]
    pub _move: nwg::Button,
	
	#[nwg_control(text: "Recalc Offsets", size: (90, 25), position: (520, 190), font: Some(&data.main_font))]
    pub recalculate_offsets: nwg::Button,
	
	#[nwg_control(text: "Credits", size: (50, 25), position: (620, 190), font: Some(&data.main_font))]
    pub credits: nwg::Button,
	
	
	
	
	
	
	
	
	
	
	
    #[nwg_control(text: "(path)", size: (600, 18), position: (10, 40), font: Some(&data.main_font))]
    pub path: nwg::Label,
	
	
	
	
	#[nwg_control(text: "Inc.", size: (25, 18), position: (10, 60+3), font: Some(&data.main_font))]
    pub include_search_box_label: nwg::Label,
	
	#[nwg_control(parent: window, text: "", size: (150, 20), position: (35, 60), font: Some(&data.main_font))]
    pub include_search_box: nwg::TextInput,
	
	
	
	
	#[nwg_control(text: "Exc.", size: (25, 18), position: (195, 60+3), font: Some(&data.main_font))]
    pub exclude_search_box_label: nwg::Label,
	
	#[nwg_control(parent: window, text: "", size: (150, 20), position: (220, 60), font: Some(&data.main_font))]
    pub exclude_search_box: nwg::TextInput,
	
	
	
	
	#[nwg_control(text: "Showing 0 entries (100%)", size: (250, 18), position: (380, 60+3), font: Some(&data.main_font))]
    pub showing_entry_count: nwg::Label,
	
	
	
	
	
	
	//#[nwg_layout(parent: window)]
	//layout: nwg::GridLayout,
	
	#[nwg_control(size: (500, 590), position: (10, 85), list_style: nwg::ListViewStyle::Detailed)]
	//#[nwg_layout_item(layout: layout, col: 0, col_span: 4, row: 0, row_span: 6)]
	pub main_entries: nwg::ListView,
	
	
	
	
	#[nwg_control(text: "0 entries", size: (200, 18), position: (10, 680), font: Some(&data.main_font))]
    pub entry_count: nwg::Label,
	
	#[nwg_control(text: "0 selected entries (0%)", size: (200, 18), position: (220, 680), font: Some(&data.main_font))]
    pub selected_entry_count: nwg::Label,
	
	#[nwg_control(text: "", size: (200, 18), position: (520, 680), font: Some(&data.main_font))]
    pub img_version: nwg::Label,
	
	
	
	
	
	#[nwg_control(text: "", size: (200, 15*10), position: (520, 230), font: Some(&data.main_font))]
    pub entry_extension_counts: nwg::Label,
	
	
	
	
	
	
	
	#[nwg_control(text: "0 overlapping entries", size: (200, 18), position: (520, 400), font: Some(&data.main_font))]
    pub overlapping_entries: nwg::Label,
	
	#[nwg_control(text: "0 entry gaps", size: (200, 18), position: (520, 420), font: Some(&data.main_font))]
    pub entry_gaps: nwg::Label,
	
	#[nwg_control(text: "0 blank entries", size: (200, 18), position: (520, 440), font: Some(&data.main_font))]
    pub blank_entries: nwg::Label,
	
	#[nwg_control(text: "0 entries missing data", size: (200, 18), position: (520, 460), font: Some(&data.main_font))]
    pub missing_entries: nwg::Label,
	
	#[nwg_control(text: "0 duplicate entry names", size: (200, 18), position: (520, 480), font: Some(&data.main_font))]
    pub duplicate_entry_names: nwg::Label,
	
	/*
	#[nwg_control(text: "0 duplicate entry data", size: (200, 18), position: (520, 500), font: Some(&data.main_font))]
    pub duplicate_entry_data: nwg::Label,
	
	#[nwg_control(text: "0 custom entries", size: (200, 18), position: (520, 520), font: Some(&data.main_font))]
    pub custom_entries: nwg::Label,
	*/
	
	
	
	
	
	
	#[nwg_control(text: "", size: (170, 125), position: (520, 550), font: Some(&data.main_font), readonly: true)]
    pub log: nwg::TextBox,
	
	
	
	
	
	
	
	#[nwg_resource(title: "Open IMG", action: nwg::FileDialogAction::Open, filters: "IMG(*.img)|RPF(*.rpf)")]
    pub open_dialog: nwg::FileDialog,
	
	#[nwg_resource(title: "Save IMG", action: nwg::FileDialogAction::Save, filters: "IMG(*.img)|RPF(*.rpf)")]
    pub save_dialog: nwg::FileDialog,
	
	#[nwg_resource(title: "Add File to IMG", action: nwg::FileDialogAction::Open, filters: "Any (*.*)", multiselect: true)]
    pub add_dialog: nwg::FileDialog,
	
	#[nwg_resource(title: "Replace Files in IMG (By Filename)", action: nwg::FileDialogAction::Open, filters: "Any (*.*)", multiselect: true)]
    pub replace_dialog: nwg::FileDialog,
	
	#[nwg_resource(title: "Export Files from IMG (To a Folder)", action: nwg::FileDialogAction::OpenDirectory)]
    pub export_dialog: nwg::FileDialog
	
	//#[nwg_resource(source_file: Some("./test_rc/cog.ico"))]
    //icon: nwg::Icon,
}

impl BasicApp
{
}







#[derive(Default, NwgUi)]
pub struct TextInputWindow
{
	#[nwg_resource(family: "Verdana", size: 13)]
	pub main_font: nwg::Font,
	
    #[nwg_control(size: (320, 100), position: (100, 100), title: "", flags: "WINDOW|VISIBLE")]
    pub window: nwg::Window,
	
	#[nwg_control(text: "", size: (300, 18), position: (10, 10), font: Some(&data.main_font))]
    pub text: nwg::Label,
	
	#[nwg_control(text: "", size: (300, 20), position: (10, 30), font: Some(&data.main_font), readonly: false)]
    pub text_box: nwg::TextBox,
	
	#[nwg_control(text: "Ok", size: (70, 25), position: (10, 60), font: Some(&data.main_font))]
    pub ok_button: nwg::Button,
	
	#[nwg_control(text: "Cancel", size: (70, 25), position: (80, 60), font: Some(&data.main_font))]
    pub cancel_button: nwg::Button
}

#[derive(Default, NwgUi)]
pub struct CreditsWindow
{
	#[nwg_resource(family: "Verdana", size: 13)]
	pub main_font: nwg::Font,
	
    #[nwg_control(size: (500, 400), position: (100, 100), title: "", flags: "WINDOW|VISIBLE")]
    pub window: nwg::Window,
	
	#[nwg_control(text: "", size: (480, 330), position: (10, 10), font: Some(&data.main_font))]
    pub text: nwg::Label,
	
	#[nwg_control(text: "Ok", size: (70, 25), position: (10, 360), font: Some(&data.main_font))]
    pub ok_button: nwg::Button
}