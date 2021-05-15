pub mod format;
pub mod gui;
pub mod action;
pub mod utility;
pub mod vendor;

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;
extern crate winapi;

//use nwd::NwgUi;
//use nwg::NativeUi;

use nwg::*;
//use nwd::*;

use std::path::Path;
use std::mem;
use std::collections::HashSet;
use std::collections::HashMap;
use std::cmp;
use std::fs;
use std::time::SystemTime;

use num_format::{Locale, ToFormattedString};

use winapi::um::winuser::WM_NOTIFY;
use winapi::um::winuser::WM_SETFOCUS;
use winapi::um::commctrl::LVN_ITEMCHANGING;
use winapi::um::commctrl::LVIF_STATE;
use winapi::um::commctrl::LVIS_SELECTED;
use winapi::um::commctrl::NMLISTVIEW;

use action::*;
use format::img as img;
use format::entry::Entry as Entry;






pub const WINDOW_TITLE_BASE			: &str = "IMG Desk 1.0 (Beta)";






static mut editor: Option<Editor> = None;

pub fn get_editor() -> &'static mut Editor
{
	unsafe
	{
		editor.as_mut().unwrap()
	}
}






pub fn load()
{
	unsafe
	{
		editor = Some(Editor
		{
			gui: gui::Gui::default(),
			format: format::Format::default(),
			edited: false,
			selected_entry_count: 0,
			action_history: ActionHistory::default(),
			img_file_last_modified: None,
			img_file_on_disk_edited: false,
			entry_offsets_were_recalculated: false,
			skip_prompt_for_recalculating_offsets: false
		});
		
		get_editor().load();
	}
}




pub struct Editor
{
	pub format: format::Format,
	pub gui: gui::Gui,
	pub edited: bool,
	pub selected_entry_count: u64,
	pub action_history: ActionHistory,
	pub img_file_last_modified: Option<SystemTime>,
	pub img_file_on_disk_edited: bool,
	pub entry_offsets_were_recalculated: bool,
	pub skip_prompt_for_recalculating_offsets: bool
}

impl Editor
{
	// load
	pub fn load(&mut self)
	{
		self.gui.load();
		self.on_no_file_open();
		self.bind_events();
	}
	
	// events
	pub fn new(&mut self)
	{
		if self.is_open()
		{
			if !self.close()
			{
				return;
			}
		}
		
		let file_name = String::from("new.img");
		let img_path_in : String = utility::get_next_file_path2(self.format.get_new_dir(), file_name.clone());
		let dir_path_in = utility::replace_file_extension(&img_path_in, "dir").unwrap();
		
		self.format.new(&img_path_in, &dir_path_in);
		
		self.on_file_open();
		self.set_edited(true);
		
		self.log(&format!("Created IMG {}", utility::get_file_name(&img_path_in).unwrap()));
	}
	
	pub fn open(&mut self)
	{
		if !self.gui.app.open_dialog.run(Some(&self.gui.app.window))
		{
			return;
		}
		
		let img_path_in = self.gui.app.open_dialog.get_selected_item().unwrap();
		self.open_directly(img_path_in);
	}
	
	fn open_directly(&mut self, img_path_in: String)
	{
		if self.is_open()
		{
			if !self.close()
			{
				return;
			}
		}
		
		let dir_path_in = utility::replace_file_extension(&img_path_in, "dir").unwrap();
		
		self.format.parse(&img_path_in, &dir_path_in);
		
		self.on_file_open();
		
		self.log(&format!("Opened IMG {}", utility::get_file_name(&img_path_in).unwrap()));
	}
	
	pub fn close(&mut self) -> bool
	{
		if !self.check_to_save(true)
		{
			return false;
		}
		
		self.log(&format!("Closed IMG {}", utility::get_file_name(&self.format.img_path_in).unwrap()));
		
		self.on_no_file_open();
		
		return true;
	}
	
	pub fn reopen(&mut self)
	{
		if !self.is_open()
		{
			return;
		}
		
		let img_path_in : String = self.format.img_path_in.clone();
		
		self.close();
		self.open_directly(img_path_in);
	}
	
	pub fn save(&mut self)
	{
		if !self.check_if_can_save()
		{
			return;
		}
		
		if !self.gui.app.save_dialog.run(Some(&self.gui.app.window))
		{
			return;
		}
		
		let img_path_out = self.gui.app.save_dialog.get_selected_item().unwrap();
		let dir_path_out = utility::replace_file_extension(&img_path_out, "dir").unwrap();
		
		self.format.save(&img_path_out, &dir_path_out);
		
		self.log(&format!("Saved IMG {}", utility::get_file_name(&img_path_out).unwrap()));
		
		self.set_edited(false);
	}
	
	pub fn add(&mut self)
	{
		if !self.gui.app.add_dialog.run(Some(&self.gui.app.window))
		{
			return;
		}
		
		if let Ok(file_paths) = self.gui.app.add_dialog.get_selected_items()
		{
			let mut entry_offsets = self.format.get_entry_offsets();
			
			let (added_entry_count, new_entries) = self.add_entries(file_paths);
			
			/*
			for new_entry in new_entries.iter()
			{
				entry_offsets.push(new_entry.offset_out as u64);
			}
			*/
			
			self.log(&format!("Added {} {}", added_entry_count, if added_entry_count == 1 { "entry" } else { "entries" }));
			
			self.add_action(ActionTypeId::AddEntries, new_entries);
			
			//let mut entry_offsets = self.format.get_entry_offsets();
			
			let mut action = self.action_history.actions.last_mut().unwrap();
			action.add.entry_offsets_were_recalculated = self.entry_offsets_were_recalculated;
			action.add.entry_offsets = entry_offsets;
		}
	}
	
	pub fn remove(&mut self)
	{
		let selected_entries = self.get_selected_entries();
		//let mut entry_offsets = self.format.get_entry_offsets();
		
		let params = nwg::MessageParams
		{
			title: "Remove Entries",
			content: &format!("Remove {} entries?", selected_entries.len()),
			buttons: nwg::MessageButtons::YesNoCancel,
			icons: nwg::MessageIcons::Warning
		};
		let result = self.gui.message(&params);
		match result
		{
			MessageChoice::Yes => {},
			MessageChoice::No =>
			{
				return;
			},
			MessageChoice::Cancel =>
			{
				return;
			},
			_ => {}
		}
		
		let selected_entry_count = selected_entries.len();
		self.log(&format!("Removed {} {}", selected_entry_count, if selected_entry_count == 1 { "entry" } else { "entries" }));
		
		self.add_action(ActionTypeId::RemoveEntries, selected_entries.clone());
		
		let mut entry_offsets = self.format.get_entry_offsets();
		
		self.remove_entries(selected_entries);
		
		let mut action = self.action_history.actions.last_mut().unwrap();
		action.remove.entry_offsets = entry_offsets;
		action.remove.entry_offsets_were_recalculated = self.entry_offsets_were_recalculated;
		
		/*
		for _i in 0..selected_entry_count
		{
			entry_offsets.pop();
		}
		*/
		
	}
	
	pub fn export(&mut self)
	{
		if !self.gui.app.export_dialog.run(Some(&self.gui.app.window))
		{
			return;
		}
		
		if let Ok(folder_path) = self.gui.app.export_dialog.get_selected_item()
		{
			let mut exported_entries = 0;
			
			let j = self.format.entries.len();
			for i in 0..j
			{
				if self.gui.app.main_entries.is_item_selected(i as i32)
				{
					let mut entry : Entry = self.format.entries[i].clone();
					self.format.export_entry(folder_path.as_str(), &mut entry);
					
					exported_entries += 1;
				}
			}
			
			self.log(&format!("Exported {} {}", exported_entries, if exported_entries == 1 { "entry" } else { "entries" }));
		}
		else
		{
			return;
		}
	}
	
	pub fn replace(&mut self)
	{
		if !self.gui.app.replace_dialog.run(Some(&self.gui.app.window))
		{
			return;
		}
		
		if let Ok(file_paths) = self.gui.app.replace_dialog.get_selected_items()
		{
			let old_entries = self.get_old_entries_for_replace(file_paths.clone());
			//let mut entry_offsets = self.format.get_entry_offsets();
			
			self.add_action(ActionTypeId::ReplaceEntries, old_entries);
			
			let (replaced_entry_count, new_entries) = self.replace_entries(file_paths);
			
			/*
			for new_entry in new_entries.iter()
			{
				entry_offsets.push(new_entry.offset_out as u64);
			}
			*/
			let mut entry_offsets = self.format.get_entry_offsets();
			
			let mut action = self.action_history.actions.last_mut().unwrap();
			action.replace.entry_offsets_were_recalculated = self.entry_offsets_were_recalculated;
			action.replace.entry_offsets = entry_offsets;
			
			self.log(&format!("Replaced {} {}", replaced_entry_count, if replaced_entry_count == 1 { "entry" } else { "entries" }));
		}
	}
	
	pub fn undo(&mut self)
	{
		if self.action_history.actions.len() == 0
		{
			return;
		}
		
		{
			self.action_history.actions[self.action_history.index as usize].undo();
		}
		
		self.action_history.index -= 1;
		
		if self.action_history.index == -1
		{
			self.set_edited(false);
		}
		
		self.gui.update_undo_redo_buttons();
		self.gui.readd_entries_to_list();
	}
	
	pub fn redo(&mut self)
	{
		if self.action_history.actions.len() == 0
		{
			return;
		}
		
		{
			self.action_history.index += 1;
			self.action_history.actions[self.action_history.index as usize].redo();
		}
		
		if self.action_history.index != (self.action_history.actions.len() as i64 - 1)
		{
			self.set_edited(true);
		}
		
		self.gui.update_undo_redo_buttons();
		self.gui.readd_entries_to_list();
	}
	
	pub fn select_all(&mut self)
	{
		if self.format.entries.len() == 0
		{
			return;
		}
		
		let selected : bool = !self.gui.app.main_entries.is_item_selected(0);
		
		self.gui.app.main_entries.set_focus();
		
		let j = self.format.entries.len();
		for i in 0..j
		{
			self.gui.app.main_entries.set_item_selected(i as i32, selected);
		}
	}
	
	pub fn select_inverse(&mut self)
	{
		if self.format.entries.len() == 0
		{
			return;
		}
		
		self.gui.app.main_entries.set_focus();
		
		let j = self.format.entries.len();
		for i in 0..j
		{
			let selected : bool = !self.gui.app.main_entries.is_item_selected(i as i32);
			
			self.gui.app.main_entries.set_item_selected(i as i32, selected);
		}
	}
	
	pub fn recalculate_offsets(&mut self)
	{
		if self.format.entries.len() == 0
		{
			return;
		}
		
		self.add_action_recalculate_offsets();
		
		self.format.recalculate_entry_offsets();
		
		self.log(&format!("Recalculated all entry offsets."));
		
		self.set_edited(true);
	}
	
	pub fn credits(&mut self)
	{
		self.gui.show_credits_window();
	}
	
	pub fn rename(&mut self)
	{
		let selected_entries = self.get_selected_entries();
		let sel_count = selected_entries.len();
		
		match sel_count
		{
			0 =>
			{
				let params = nwg::MessageParams
				{
					title: "Rename Requirements Failed",
					content: "An entry must be selected to perform a rename.",
					buttons: nwg::MessageButtons::Ok,
					icons: nwg::MessageIcons::Error
				};
				self.gui.message(&params);
				return;
			}
			1 => {},
			_ =>
			{
				let params = nwg::MessageParams
				{
					title: "Rename Requirements Failed",
					content: "Only one entry can be selected to perform a rename.",
					buttons: nwg::MessageButtons::Ok,
					icons: nwg::MessageIcons::Error
				};
				self.gui.message(&params);
				return;
			}
		}
		
		let selected_entry_copy = selected_entries[0].clone();
		
		let current_entry_name = unsafe
		{
			vendor::str_from_u8_nul_utf8_unchecked(&selected_entry_copy.name)
		};
		
		let mut new_entry_name : String = self.gui.show_text_input_window("Rename Entry", "Choose a new name for the entry.", current_entry_name);
		if new_entry_name.len() == 0
		{
			return;
		}
		
		new_entry_name = new_entry_name.trim().to_string();
		if new_entry_name.len() > 24
		{
			let params = nwg::MessageParams
			{
				title: "Rename Not Applied",
				content: "New name for entry must be 1-24 characters in length.",
				buttons: nwg::MessageButtons::Ok,
				icons: nwg::MessageIcons::Error
			};
			self.gui.message(&params);
			return;
		}
		
		//let mut selected_entry = &mut self.format.entries[selected_entry_copy.index as usize];
		{
			//let mut selected_entry_copy = &mut self.format.entries[selected_entry_copy.index as usize];
			self.add_action_rename(&selected_entry_copy, &(current_entry_name.to_string()), &(new_entry_name.to_string()));
		}
		{
			let mut selected_entry = &mut self.format.entries[selected_entry_copy.index as usize];
			selected_entry.set_name(&(new_entry_name.to_string()));
		}
		
		self.log(&format!("Renamed entry to {}", new_entry_name));
		
		self.set_edited(true);
	}
	
	pub fn _move(&mut self)
	{
		let selected_entries = self.get_selected_entries();
		let sel_count = selected_entries.len();
		
		match sel_count
		{
			0 =>
			{
				let params = nwg::MessageParams
				{
					title: "Move Requirements Failed",
					content: "An entry must be selected to perform a move.",
					buttons: nwg::MessageButtons::Ok,
					icons: nwg::MessageIcons::Error
				};
				self.gui.message(&params);
				return;
			}
			1 => {},
			_ =>
			{
				let params = nwg::MessageParams
				{
					title: "Move Requirements Failed",
					content: "Only one entry can be selected to perform a move.",
					buttons: nwg::MessageButtons::Ok,
					icons: nwg::MessageIcons::Error
				};
				self.gui.message(&params);
				return;
			}
		}
		
		let entry_count = self.format.entries.len();
		let selected_entry_copy = selected_entries[0].clone();
		let selected_entry_index = selected_entry_copy.index;
		
		let current_entry_index = {
			let selected_entry = &self.format.entries[selected_entry_copy.index as usize];
			selected_entry.index
		};
		let shown_text = &format!("Choose a new index for the entry.\n\nBetween 1 and {}.", entry_count.to_formatted_string(&Locale::en));
		let mut new_entry_index_str : String = self.gui.show_text_input_window("Move Entry", shown_text, &(current_entry_index + 1).to_formatted_string(&Locale::en));
		if new_entry_index_str.len() == 0
		{
			return;
		}
		
		new_entry_index_str = new_entry_index_str.trim().to_string().replace(",", "");
		let new_entry_index_result = new_entry_index_str.parse::<i64>();
		
		match new_entry_index_result
		{
			Err(e) =>
			{
				let params = nwg::MessageParams
				{
					title: "Move Not Applied",
					content: "New index for entry must be an integer.",
					buttons: nwg::MessageButtons::Ok,
					icons: nwg::MessageIcons::Error
				};
				self.gui.message(&params);
				return;
			},
			_ => {}
		};
		let new_entry_index = new_entry_index_result.unwrap();
		
		if new_entry_index < 1 || new_entry_index > (entry_count as i64)
		{
			let params = nwg::MessageParams
			{
				title: "Move Not Applied",
				content: &format!("New index for entry must be between 1 and {}.", entry_count.to_formatted_string(&Locale::en)),
				buttons: nwg::MessageButtons::Ok,
				icons: nwg::MessageIcons::Error
			};
			self.gui.message(&params);
			return;
		}
		
		{
			let mut selected_entry = &mut self.format.entries[selected_entry_index as usize].clone();
			self.add_action_move(&selected_entry, current_entry_index as u64, (new_entry_index - 1) as u64);
		}
		
		{
			let mut selected_entry = &mut self.format.entries[selected_entry_index as usize];
			selected_entry.set_index((new_entry_index - 1) as u64);
		}
		
		
		let entry_name = unsafe
		{
			let selected_entry = &self.format.entries[selected_entry_copy.index as usize];
			vendor::str_from_u8_nul_utf8_unchecked(&selected_entry.name)
		};
		self.log(&format!("Moved {} to #{}", entry_name, new_entry_index));
		
		self.set_edited(true);
	}
	
	pub fn set_img_version(&mut self)
	{
		let current_img_version = self.format.img_version;
		let current_img_encrypted = self.format.img_encrypted;
		
		let (img_version, img_encrypted) = match self.gui.app.img_version_combo.selection().unwrap()
		{
			0 => (1, false),
			1 => (2, false),
			2 => (3, true),
			3 => (3, false),
			_ => (1, false)
		};
		
		self.format.set_version(img_version, img_encrypted);
		
		self.add_action_set_img_version(current_img_version, current_img_encrypted, img_version, img_encrypted);
		
		let img_version_text : String = self.gui.get_img_version_text(img_version, img_encrypted);
		self.log(&format!("IMG version set to {}", &img_version_text.to_owned()));
		
		self.set_edited(true);
	}
	
	// log
	fn log(&mut self, text: &str)
	{
		let text_box_text = self.gui.app.log.text();
		
		if text_box_text.len() > 0
		{
			self.gui.app.log.set_text(&format!("{}\r\n{}", text_box_text, text));
		}
		else
		{
			self.gui.app.log.set_text(text);
		}
		
		let log_line_count = 2000000000;
		self.gui.app.log.set_scroll_pos(log_line_count);
	}
	
	// edited
	fn set_edited(&mut self, edited: bool)
	{
		if self.img_file_on_disk_edited
		{
			self.gui.update_path();
			return;
		}
		
		self.edited = edited;
		self.gui.update_path();
	}
	
	// open
	pub fn is_open(&mut self) -> bool
	{
		self.format.img_path_in.len() != 0
	}
	
	// save
	pub fn check_to_save(&mut self, allow_cancel_option: bool) -> bool
	{
		if self.edited
		{
			let params = nwg::MessageParams
			{
				title: "Unsaved Changes",
				content: "Save this IMG file?",
				buttons: if allow_cancel_option { nwg::MessageButtons::YesNoCancel } else { nwg::MessageButtons::YesNo },
				icons: nwg::MessageIcons::Warning
			};
			let result = self.gui.message(&params);
			match result
			{
				nwg::MessageChoice::Yes =>
				{
					self.save();
				},
				nwg::MessageChoice::No => {},
				nwg::MessageChoice::Cancel =>
				{
					return false;
				},
				_ => {}
			}
		}
		
		true
	}
	
	// file path used
	fn is_file_path_used(&mut self, file_path_str: &String) -> bool
	{
		let file_path = Path::new(&file_path_str).canonicalize().unwrap().into_os_string();
		
		let file_path_img = Path::new(&self.format.img_path_in).canonicalize().unwrap().into_os_string();
		if file_path == file_path_img
		{
			return true;
		}
		
		if !self.format.dir_path_in.is_empty()
		{
			let file_path_dir = Path::new(&self.format.dir_path_in).canonicalize().unwrap().into_os_string();
			if file_path == file_path_dir
			{
				return true;
			}
		}
		
		false
	}
	
	// add entries
	pub fn add_entries(&mut self, file_paths: Vec<String>) -> (u64, Vec<Entry>)
	{
		self.add_entries_with(file_paths, Vec::new(), Vec::new())
	}
	
	pub fn add_entries_with(&mut self, file_paths: Vec<String>, entry_indices: Vec<i32>, entry_names: Vec<String>) -> (u64, Vec<Entry>)
	{
		let mut added_entry_count = 0u64;
		let mut ensure_visible_item_index = -1i64;
		let mut new_entries = Vec::new();
		
		let mut i = 0;
		for file_path in file_paths
		{
			if self.is_file_path_used(&file_path)
			{
				let params = nwg::MessageParams
				{
					title: "Cannot Add File",
					content: "Cannot add recursive file.",
					buttons: nwg::MessageButtons::Ok,
					icons: nwg::MessageIcons::Error
				};
				self.gui.message(&params);
				i += 1;
				continue;
			}
			
			let entry_index = if entry_indices.len() == 0
			{
				-1
			}
			else
			{
				entry_indices[i]
			};
			
			let entry_name = if entry_names.len() == 0
			{
				String::from("")
			}
			else
			{
				entry_names[i].clone()
			};
			
			let entry = self.format.add_file_at(file_path.to_string(), entry_index, entry_name);
			
			if ensure_visible_item_index == -1i64
			{
				ensure_visible_item_index = if entry_indices.len() == 0
				{
					entry.index as i64
				}
				else
				{
					entry_indices[i] as i64
				};
			}
			
			if entry_indices.len() == 0
			{
				self.gui.check_to_add_entry(&entry);
			}
			else
			{
				self.gui.check_to_add_entry_at(&entry, entry_indices[i]);
			}
			
			new_entries.push(entry.clone());
			
			added_entry_count += 1;
			i += 1;
		}
		
		if added_entry_count > 0
		{
			self.on_add_entries();
		}
		
		self.format.reassign_entry_indices();
		//self.format.recalculate_offsets();
		//self.gui.update_entry_indices();
		self.gui.readd_entries_to_list();
		
		self.gui.app.main_entries.ensure_visible(ensure_visible_item_index as i32);
		
		self.gui.update_entry_count();
		
		self.set_edited(true);
		
		(added_entry_count, new_entries)
	}
	
	// remove entries
	pub fn remove_entries(&mut self, entries: Vec<Entry>)
	{
		//self.format.entries.retain(|x|!entries.contains(&x));
		
		//entries.into_iter().map(|x|self.format.remove(&x));
		
		for entry in entries.iter()
		{
			self.format.remove(&entry);
		}
		
		self.on_remove_entries();
		
		self.format.reassign_entry_indices();
		
		self.gui.readd_entries_to_list();
		
		self.gui.update_entry_count();
		
		self.set_edited(true);
	}
	
	// entry checking
	fn can_file_path_be_added(&mut self, file_path: &String) -> bool
	{
		let file_name = utility::get_file_name(&file_path).unwrap().to_string();
		
		if self.is_file_path_used(&file_path)
		{
			return false;
		}
		
		let entry_name_count = self.format.get_entry_count_by_name(file_name);
		if entry_name_count != 1
		{
			return false;
		}
		
		return true;
	}
	
	fn does_an_entry_overlap_header(&mut self) -> bool
	{
		let min_offset = self.format.get_img_header_size() as u32;
		self.format.entries.iter().filter(|&e| e.offset_out < min_offset).count() > 0
	}
	
	fn does_an_entry_overlap_directory(&mut self) -> bool
	{
		let min_offset = (self.format.get_img_header_size() + self.format.get_img_directory_size()) as u32;
		self.format.entries.iter().filter(|&e| e.offset_out < min_offset).count() > 0
	}
	
	fn is_any_entry_offset_invalid(&mut self) -> bool
	{
		let min_offset = self.format.get_entry_data_offset() as u32;
		self.format.entries.iter().filter(|&e| e.offset_out < min_offset).count() > 0
	}
	
	fn do_any_entries_overlap(&mut self) -> bool
	{
		self.get_overlapping_entry_count() > 0
	}
	
	fn check_if_can_save(&mut self) -> bool
	{
		let error_type = if self.does_an_entry_overlap_header()
		{
			1
		}
		else if self.does_an_entry_overlap_directory()
		{
			2
		}
		else if self.is_any_entry_offset_invalid()
		{
			3
		}
		else if self.do_any_entries_overlap()
		{
			4
		}
		else
		{
			0
		};
		
		if error_type == 0
		{
			return true;
		}
		else
		{
			let content = match error_type
			{
				1 => format!("Can't save because at least one entry overlaps the file header.\n\nRecalculate all entry offsets and save?"),
				2 => format!("Can't save because at least one entry overlaps the file directory.\n\nRecalculate all entry offsets and save?"),
				3 => format!("Can't save because at least one entry has an invalid offset (which is likely too low).\n\nRecalculate all entry offsets and save?"),
				4 => format!("Can't save because at least two entries overlap each other.\n\nRecalculate all entry offsets and save?"),
				_ => format!("")
			};
			let params = nwg::MessageParams
			{
				title: "Can't Save due to Overlapping Data",
				content: &content.to_owned(),
				buttons: nwg::MessageButtons::YesNoCancel,
				icons: nwg::MessageIcons::Error
			};
			let result = self.gui.message(&params);
			match result
			{
				MessageChoice::Yes =>
				{},
				MessageChoice::No =>
				{
					return false;
				},
				MessageChoice::Cancel =>
				{
					return false;
				},
				_ => {}
			}
			
			self.format.recalculate_entry_offsets();
			
			return true;
		}
	}
	
	// replace entries
	fn get_old_entries_for_replace(&mut self, file_paths: Vec<String>) -> Vec<Entry>
	{
		self.get_old_entries_for_replace_with(file_paths, Vec::new(), Vec::new())
	}
	
	fn get_old_entries_for_replace_with(&mut self, file_paths: Vec<String>, entry_indices: Vec<i32>, entry_names: Vec<String>) -> Vec<Entry>
	{
		let mut old_entries = Vec::new();
		
		let mut i = 0u64;
		for file_path in file_paths
		{
			let file_name = utility::get_file_name(&file_path).unwrap().to_string();
			
			if self.is_file_path_used(&file_path)
			{
				i += 1;
				continue;
			}
			
			let entry_name = if entry_names.len() == 0
			{
				file_name.clone()
			}
			else
			{
				entry_names[i as usize].clone()
			};
			let entry_name_count = self.format.get_entry_count_by_name(entry_name.clone());
			
			if entry_name_count != 1
			{
				i += 1;
				continue;
			}
			
			let entry_index : i32 = if entry_indices.len() == 0
			{
				-1
			}
			else
			{
				entry_indices[i as usize] as i32
			};
			
			let mut entry : &mut Entry = if entry_index == -1
			{
				self.format.get_entry_by_name(file_name.clone()).unwrap()
			}
			else
			{
				self.format.get_entry_by_index(entry_index as u64).unwrap()
			};
			
			old_entries.push(entry.clone());
			i += 1;
		}
		
		old_entries
	}
	
	fn replace_entries(&mut self, file_paths: Vec<String>) -> (i64, Vec<Entry>)
	{
		self.replace_entries_with(file_paths, Vec::new(), Vec::new())
	}
	
	fn replace_entries_with(&mut self, file_paths: Vec<String>, entry_indices: Vec<i32>, entry_names: Vec<String>) -> (i64, Vec<Entry>)
	{
		let mut replaced_entry_count = 0i64;
		let mut ensure_visible_item_index = -1i64;
		let mut new_entries = Vec::new();
		
		let mut i : u64 = 0;
		for file_path in file_paths
		{
			let file_name = utility::get_file_name(&file_path).unwrap().to_string();
			
			if self.is_file_path_used(&file_path)
			{
				let params = nwg::MessageParams
				{
					title: "Cannot Replace File",
					content: &format!("Cannot replace recursive file \"{}\"", file_name),
					buttons: nwg::MessageButtons::Ok,
					icons: nwg::MessageIcons::Error
				};
				self.gui.message(&params);
				i += 1;
				continue;
			}
			
			let entry_name = if entry_names.len() == 0
			{
				file_name.clone()
			}
			else
			{
				entry_names[i as usize].clone()
			};
			let entry_name_count = self.format.get_entry_count_by_name(entry_name.clone());
			
			if entry_name_count != 1
			{
				let params = nwg::MessageParams
				{
					title: "Cannot Replace File",
					content: &format!("Cannot replace file, entry name \"{}\" already found", entry_name),
					buttons: nwg::MessageButtons::Ok,
					icons: nwg::MessageIcons::Error
				};
				self.gui.message(&params);
				i += 1;
				continue;
			}
			
			let entry_index = if entry_indices.len() == 0
			{
				-1
			}
			else
			{
				entry_indices[i as usize]
			};
			
			let entry = self.format.replace_file_at(file_path.to_string(), entry_index, entry_name);
			
			replaced_entry_count += 1;
			
			if ensure_visible_item_index == -1i64
			{
				ensure_visible_item_index = entry.index as i64;
			}
			
			self.gui.update_entry(&entry);
			
			new_entries.push(entry.clone());
			
			i += 1;
		}
		
		self.gui.readd_entries_to_list();
		
		self.gui.app.main_entries.ensure_visible(ensure_visible_item_index as i32);
		self.set_edited(true);
		
		(replaced_entry_count, new_entries)
	}
	
	// actions
	fn remove_actions_after_index(&mut self, index: i64)
	{
		for i in ((index + 1)..(self.action_history.actions.len() as i64)).rev()
		{
			self.remove_action_by_index(i);
		}
		
		unsafe
		{
			self.action_history.actions.set_len((index + 1) as usize);
		}
	}
	
	fn remove_action_by_index(&mut self, index: i64)
	{
		let mut action = &self.action_history.actions[index as usize];
		
		//action_entry.remove();
		
		match action._type
		{
			ActionTypeId::AddEntries =>
			{
				action.add.entries.clone().into_iter().for_each(|mut action_entry|{
					if !action_entry.data_undo_path.is_empty()
					{
						fs::remove_file(action_entry.data_undo_path.as_str());
						action_entry.data_undo_path = String::from("");
					}
				});
			},
			ActionTypeId::RemoveEntries =>
			{
				action.remove.entries.clone().into_iter().for_each(|mut action_entry|{
					if !action_entry.data_undo_path.is_empty()
					{
						fs::remove_file(action_entry.data_undo_path.as_str());
						action_entry.data_undo_path = String::from("");
					}
				});
			},
			ActionTypeId::ReplaceEntries =>
			{
				action.replace.entries.clone().into_iter().for_each(|mut action_entry|{
					if !action_entry.data_undo_path.is_empty()
					{
						fs::remove_file(action_entry.data_undo_path.as_str());
						action_entry.data_undo_path = String::from("");
					}
				});
			},
			ActionTypeId::RenameEntry =>
			{
			},
			ActionTypeId::MoveEntry =>
			{
			},
			ActionTypeId::SetImgVersion =>
			{
			},
			_ => {}
		}
		
		
	}
	
	/*
	fn remove_action_entry(&mut self, action_entry: &mut ActionItem)
	{
		if !action_entry.data_undo_path.is_empty()
		{
			fs::remove_file(action_entry.data_undo_path.as_str());
			action_entry.data_undo_path = String::from("");
		}
	}
	*/
	
	fn add_undo_file(&mut self, entry: &Entry, action_item: &mut ActionReplaceEntry)
	{
		let entry_name = unsafe
		{
			vendor::str_from_u8_nul_utf8_unchecked(&entry.name).to_string()
		};
		let data_undo_path : String = utility::get_next_file_path2(self.format.get_undo_dir(), entry_name.clone());
		utility::set_file_data(data_undo_path.clone(), &self.format.get_entry_data(&entry));
		action_item.data_undo_path = data_undo_path;
	}
	
	fn remove_undo_file(&mut self, entry: &Entry)
	{
		//fs::remove_file(entry.data_undo_path.as_str());
		//entry.data_undo_path = String::from("");
	}
	
	fn add_action_before(&mut self, _type: ActionTypeId)
	{
		{
			self.remove_actions_after_index(self.action_history.index);
		}
		
		{
			let mut action = ActionItem::default();
			action._type = _type;
			self.action_history.actions.push(action);
		}
		
		{
			self.action_history.index += 1;
		}
	}
	
	fn add_action_after(&mut self)
	{
		self.gui.update_undo_redo_buttons();
	}
	
	fn add_action(&mut self, _type: ActionTypeId, entries: Vec<Entry>)
	{
		self.add_action_before(_type);
		
		let mut action = self.action_history.actions.last_mut().unwrap();
		
		let e = action._type;
		match e
		{
			ActionTypeId::AddEntries =>
			{
				action.add = ActionAdd
				{
					entries: Vec::new(),
					entry_offsets_were_recalculated: false,
					entry_offsets: Vec::new()
				};
				for entry in entries
				{
					let entry_name = unsafe
					{
						vendor::str_from_u8_nul_utf8_unchecked(&entry.name).to_string()
					};
					
					let data_undo_path : String = utility::get_next_file_path2(self.format.get_undo_dir(), entry_name.clone());
					utility::set_file_data(data_undo_path.clone(), &self.format.get_entry_data(&entry));
					
					action.add.entries.push(ActionAddEntry
					{
						entry_index: entry.index as u64,
						entry_name: entry_name,
						data_undo_path: data_undo_path
					});
				}
			},
			ActionTypeId::RemoveEntries =>
			{
				action.remove = ActionRemove
				{
					entries: Vec::new(),
					entry_offsets_were_recalculated: false,
					entry_offsets: Vec::new()
				};
				for entry in entries
				{
					let entry_name = unsafe
					{
						vendor::str_from_u8_nul_utf8_unchecked(&entry.name).to_string()
					};
					
					let data_undo_path : String = utility::get_next_file_path2(self.format.get_undo_dir(), entry_name.clone());
					utility::set_file_data(data_undo_path.clone(), &self.format.get_entry_data(&entry));
					
					action.remove.entries.push(ActionRemoveEntry
					{
						entry_index: entry.index as u64,
						entry_name: entry_name,
						data_undo_path: data_undo_path
					});
				}
			},
			ActionTypeId::ReplaceEntries =>
			{
				action.replace = ActionReplace
				{
					entries: Vec::new(),
					entry_offsets_were_recalculated: false,
					entry_offsets: Vec::new()
				};
				for entry in entries
				{
					let entry_name = unsafe
					{
						vendor::str_from_u8_nul_utf8_unchecked(&entry.name).to_string()
					};
					
					let data_undo_path : String = utility::get_next_file_path2(self.format.get_undo_dir(), entry_name.clone());
					utility::set_file_data(data_undo_path.clone(), &self.format.get_entry_data(&entry));
					
					action.replace.entries.push(ActionReplaceEntry
					{
						entry_index: entry.index as u64,
						entry_name: entry_name,
						data_undo_path: data_undo_path
					});
				}
			},
			_ => {}
		}
		
		self.add_action_after();
	}
	
	fn add_action_rename(&mut self, entry: &Entry, old_name: &String, new_name: &String)
	{
		self.add_action_before(ActionTypeId::RenameEntry);
		
		let mut action = self.action_history.actions.last_mut().unwrap();
		
		action.rename = ActionRename
		{
			entry_index: entry.index as u64,
			old_name: old_name.clone(),
			new_name: new_name.clone()
		};
		
		self.add_action_after();
	}
	
	fn add_action_move(&mut self, entry: &Entry, old_entry_index: u64, new_entry_index: u64)
	{
		self.add_action_before(ActionTypeId::MoveEntry);
		
		let mut action = self.action_history.actions.last_mut().unwrap();
		
		action._move = ActionMove
		{
			old_entry_index: old_entry_index,
			new_entry_index: new_entry_index
		};
		
		self.add_action_after();
	}
	
	fn add_action_set_img_version(&mut self, old_img_version: u8, old_img_encrypted: bool, new_img_version: u8, new_img_encrypted: bool)
	{
		self.add_action_before(ActionTypeId::SetImgVersion);
		
		let mut action = self.action_history.actions.last_mut().unwrap();
		
		action.set_img_version = ActionSetImgVersion
		{
			old_img_version: old_img_version,
			old_img_encrypted: old_img_encrypted,
			new_img_version: new_img_version,
			new_img_encrypted: new_img_encrypted
		};
		
		self.add_action_after();
	}
	
	fn add_action_recalculate_offsets(&mut self)
	{
		self.add_action_before(ActionTypeId::RecalculateOffsets);
		
		let mut action = self.action_history.actions.last_mut().unwrap();
		
		action.recalculate_offsets = ActionRecalculateOffsets { entry_offsets: self.format.get_entry_offsets() };
		
		self.add_action_after();
	}
	
	// undo
	pub fn undo_add_entries(&mut self, action: &mut ActionAdd)
	{
		self.skip_prompt_for_recalculating_offsets = true;
		
		let entries : Vec<Entry> = action.entries.clone().into_iter().map(|v| -> Entry {
			self.format.get_entry_by_index(v.entry_index).unwrap().clone()
		}).collect();
		
		self.log(&format!("[Undo Add] Removed {} {}", entries.len(), if entries.len() == 1 { "entry" } else { "entries" }));
		
		self.remove_entries(entries);
		
		//if action.entry_offsets_were_recalculated
		{
			self.format.set_entry_offsets(&action.entry_offsets);
		}
		
		self.skip_prompt_for_recalculating_offsets = false;
	}
	
	pub fn undo_remove_entries(&mut self, action: &mut ActionRemove)
	{
		self.skip_prompt_for_recalculating_offsets = true;
		
		let entry_file_paths : Vec<String> = action.entries.clone().into_iter().map(|v| v.data_undo_path).collect();
		let entry_indices : Vec<i32> = action.entries.clone().into_iter().map(|v| v.entry_index as i32).collect();
		let entry_names : Vec<String> = action.entries.clone().into_iter().map(|v| v.entry_name).collect();
		
		self.log(&format!("[Undo Remove] Added {} {}", entry_file_paths.len(), if entry_file_paths.len() == 1 { "entry" } else { "entries" }));
		
		self.add_entries_with(entry_file_paths, entry_indices, entry_names);
		
		//if action.entry_offsets_were_recalculated
		{
			self.format.set_entry_offsets(&action.entry_offsets);
		}
		
		self.skip_prompt_for_recalculating_offsets = false;
	}
	
	pub fn undo_replace_entries(&mut self, action: &mut ActionReplace)
	{
		self.skip_prompt_for_recalculating_offsets = true;
		
		let entry_file_paths : Vec<String> = action.entries.clone().into_iter().map(|v| v.data_undo_path).collect();
		let entry_indices : Vec<i32> = action.entries.clone().into_iter().map(|v| v.entry_index as i32).collect();
		let entry_names : Vec<String> = action.entries.clone().into_iter().map(|v| v.entry_name).collect();
		
		for mut action_entry in action.entries.iter_mut()
		{
			//self.remove_undo_file(action.data_undo_path);
			if self.can_file_path_be_added(&action_entry.data_undo_path)
			{
				let entry = self.format.get_entry_by_index(action_entry.entry_index).unwrap().clone();
				self.add_undo_file(&entry, &mut action_entry);
			}
		}
		
		let (replaced_entry_count, new_entries) = self.replace_entries_with(entry_file_paths.clone(), entry_indices, entry_names);
		
		for entry_file_path in entry_file_paths.iter()
		{
			if self.can_file_path_be_added(&entry_file_path)
			{
				fs::remove_file(entry_file_path.as_str());
			}
		}
		
		//if action.entry_offsets_were_recalculated
		{
			self.format.set_entry_offsets(&action.entry_offsets);
		}
		
		self.log(&format!("[Undo Replace] Restored {} {}", replaced_entry_count, if replaced_entry_count == 1 { "entry" } else { "entries" }));
		
		self.skip_prompt_for_recalculating_offsets = false;
	}
	
	pub fn undo_rename_entry(&mut self, action: &ActionRename)
	{
		let mut entry = self.format.get_entry_by_index(action.entry_index).unwrap();
		
		entry.set_name(&action.old_name);
		
		self.log(&format!("[Undo Rename] Restored name {}", action.old_name));
	}
	
	pub fn undo_move_entry(&mut self, action: &ActionMove)
	{
		let mut entry = self.format.get_entry_by_index(action.new_entry_index).unwrap();
		
		entry.set_index(action.old_entry_index);
		
		self.log(&format!("[Undo Move] Restored index #{}", action.old_entry_index + 1));
	}
	
	pub fn undo_set_img_version(&mut self, action: &ActionSetImgVersion)
	{
		self.format.set_version(action.old_img_version, action.old_img_encrypted);
		
		let img_version_text : String = self.gui.get_img_version_text(action.old_img_version, action.old_img_encrypted);
		self.log(&format!("[Undo Version] Restored as {}", &img_version_text.to_owned()));
	}
	
	pub fn undo_recalculate_offsets(&mut self, action: &ActionRecalculateOffsets)
	{
		self.format.set_entry_offsets(&action.entry_offsets);
		
		self.log(&format!("[Undo Recalculate Offsets]"));
	}
	
	// redo
	pub fn redo_add_entries(&mut self, action: &mut ActionAdd)
	{
		self.skip_prompt_for_recalculating_offsets = true;
		
		let entry_file_paths : Vec<String> = action.entries.clone().into_iter().map(|v| v.data_undo_path).collect();
		let entry_indices : Vec<i32> = action.entries.clone().into_iter().map(|v| v.entry_index as i32).collect();
		let entry_names : Vec<String> = action.entries.clone().into_iter().map(|v| v.entry_name).collect();
		
		self.log(&format!("[Redo Add] Added {} {}", entry_file_paths.len(), if entry_file_paths.len() == 1 { "entry" } else { "entries" }));
		
		self.add_entries_with(entry_file_paths, entry_indices, entry_names);
		
		if self.entry_offsets_were_recalculated
		{
			self.format.recalculate_entry_offsets();
		}
		
		self.skip_prompt_for_recalculating_offsets = false;
	}
	
	pub fn redo_remove_entries(&mut self, action: &mut ActionRemove)
	{
		self.skip_prompt_for_recalculating_offsets = true;
		
		let entries : Vec<Entry> = action.entries.clone().into_iter().map(|v| -> Entry {
			self.format.get_entry_by_index(v.entry_index).unwrap().clone()
		}).collect();
		
		self.log(&format!("[Redo Remove] Removed {} {}", entries.len(), if entries.len() == 1 { "entry" } else { "entries" }));
		
		self.remove_entries(entries);
		
		if self.entry_offsets_were_recalculated
		{
			self.format.recalculate_entry_offsets();
		}
		
		self.skip_prompt_for_recalculating_offsets = false;
	}
	
	pub fn redo_replace_entries(&mut self, action: &mut ActionReplace)
	{
		self.skip_prompt_for_recalculating_offsets = true;
		
		let entry_file_paths : Vec<String> = action.entries.clone().into_iter().map(|v| v.data_undo_path).collect();
		let entry_indices : Vec<i32> = action.entries.clone().into_iter().map(|v| v.entry_index as i32).collect();
		let entry_names : Vec<String> = action.entries.clone().into_iter().map(|v| v.entry_name).collect();
		
		for mut action_entry in action.entries.iter_mut()
		{
			let entry = self.format.get_entry_by_index(action_entry.entry_index).unwrap().clone();
			
			if self.can_file_path_be_added(&action_entry.data_undo_path)
			{
				self.add_undo_file(&entry, &mut action_entry);
			}
		}
		
		let (replaced_entry_count, new_entries) = self.replace_entries_with(entry_file_paths.clone(), entry_indices, entry_names);
		
		for entry_file_path in entry_file_paths.iter()
		{
			if self.can_file_path_be_added(&entry_file_path)
			{
				fs::remove_file(entry_file_path.as_str());
			}
		}
		
		if self.entry_offsets_were_recalculated
		{
			self.format.recalculate_entry_offsets();
		}
		
		self.log(&format!("[Redo Replace] Restored {} {}", replaced_entry_count, if replaced_entry_count == 1 { "entry" } else { "entries" }));
		
		self.skip_prompt_for_recalculating_offsets = false;
	}
	
	pub fn redo_rename_entry(&mut self, action: &ActionRename)
	{
		let mut entry = self.format.get_entry_by_index(action.entry_index).unwrap();
		
		entry.set_name(&action.new_name);
		
		self.log(&format!("[Redo Rename] Renamed to {}", action.old_name));
	}
	
	pub fn redo_move_entry(&mut self, action: &ActionMove)
	{
		let mut entry = self.format.get_entry_by_index(action.old_entry_index).unwrap();
		
		entry.set_index(action.new_entry_index);
		
		self.log(&format!("[Redo Move] Restored index #{}", action.new_entry_index + 1));
	}
	
	pub fn redo_set_img_version(&mut self, action: &ActionSetImgVersion)
	{
		self.format.set_version(action.new_img_version, action.new_img_encrypted);
		
		let img_version_text : String = self.gui.get_img_version_text(action.new_img_version, action.new_img_encrypted);
		self.log(&format!("[Redo Version] Restored as {}", &img_version_text.to_owned()));
	}
	
	pub fn redo_recalculate_offsets(&mut self, action: &ActionRecalculateOffsets)
	{
		self.format.recalculate_entry_offsets();
		
		self.log(&format!("[Redo Recalculate Offsets]"));
	}
	
	// other events
	fn on_file_open(&mut self)
	{
		self.on_no_rows_selected();
		self.on_only_one_row_not_selected();
		
		self.gui.update_entry_count();
		self.gui.reset_selected_entry_count();
		self.gui.update_shown_entry_count();
		
		self.update_all_stats();
		
		self.gui.readd_entries_to_list();
		
		self.gui.on_file_open();
		
		self.img_file_on_disk_edited = false;
		self.img_file_last_modified = Some(self.get_img_file_last_modified());
		
		self.update_window_title();
		
		self.set_edited(false);
	}
	
	fn on_no_file_open(&mut self)
	{
		self.format.remove_temp_dir();
		
		self.gui.on_no_file_open();
		
		self.on_no_rows_selected();
		self.on_only_one_row_not_selected();
		
		self.reset_all_stats();
		
		self.gui.clear_list();
		
		self.format.reset();
		
		self.recalculate_selected_entry_count();
		
		self.update_entry_extension_counts();
		self.gui.update_entry_count();
		self.gui.update_selected_entry_count();
		self.gui.update_shown_entry_count();
		
		self.img_file_on_disk_edited = false;
		self.img_file_last_modified = None;
		
		self.update_window_title();
		
		self.action_history.actions = Vec::new();
		self.action_history.index = -1;
		
		self.set_edited(false);
	}
	
	fn on_no_rows_selected(&mut self)
	{
		self.gui.app.export.set_enabled(false);
		self.gui.app.remove.set_enabled(false);
		
		self.selected_entry_count = 0;
		self.gui.update_selected_entry_count();
	}
	
	fn on_at_least_one_row_selected(&mut self)
	{
		self.gui.app.export.set_enabled(true);
		self.gui.app.remove.set_enabled(true);
	}
	
	fn on_only_one_row_selected(&mut self)
	{
		self.gui.app.rename.set_enabled(true);
		self.gui.app._move.set_enabled(true);
	}
	
	fn on_only_one_row_not_selected(&mut self)
	{
		self.gui.app.rename.set_enabled(false);
		self.gui.app._move.set_enabled(false);
	}
	
	fn on_select_entry(&mut self)
	{
		self.selected_entry_count += 1;
		
		if self.selected_entry_count == 1
		{
			self.on_at_least_one_row_selected();
			self.on_only_one_row_selected();
		}
		else
		{
			self.on_only_one_row_not_selected();
		}
		
		self.gui.update_selected_entry_count();
	}
	
	fn on_unselect_entry(&mut self)
	{
		self.selected_entry_count -= 1;
		
		if self.selected_entry_count == 0
		{
			self.on_no_rows_selected();
			self.on_only_one_row_not_selected();
		}
		else if self.selected_entry_count == 1
		{
			self.on_only_one_row_selected();
		}
		
		self.gui.update_selected_entry_count();
	}
	
	fn on_add_entries(&mut self)
	{
		self.on_directory_size_change();
		self.update_all_stats();
	}
	
	fn on_remove_entries(&mut self)
	{
		self.on_directory_size_change();
		self.update_all_stats();
	}
	
	pub fn on_rename_entry(&mut self)
	{
		self.update_all_stats();
		self.set_edited(true);
	}
	
	fn on_directory_size_change(&mut self)
	{
		self.check_to_adjust_entry_offsets();
		self.set_edited(true);
	}
	
	fn on_change_entry_index(&mut self)
	{
		self.format.reassign_entry_indices();
		self.gui.readd_entries_to_list();
		self.set_edited(true);
	}
	
	pub fn on_img_version_change(&mut self)
	{
		self.gui.set_active_img_version_for_combo();
		self.gui.update_img_version();
		self.set_edited(true);
	}
	
	pub fn on_entry_offsets_change(&mut self)
	{
		self.gui.update_entry_offsets();
		self.set_edited(true);
	}
	
	// entry offsets
	pub fn check_to_adjust_entry_offsets(&mut self)
	{
		self.entry_offsets_were_recalculated = false;
		
		if self.format.img_version == 1
		{
			return;
		}
		
		if self.format.entries.len() == 0
		{
			return;
		}
		
		let entry_data_offset = self.format.get_entry_data_offset() as u32;
		let entries : Vec<crate::editor::format::entry::Entry> = self.format.get_entries_sorted_by_offset_out();
		let _type = if entries.first().unwrap().offset_out < entry_data_offset
		{
			1
		}
		else if entries.first().unwrap().offset_out > entry_data_offset
		{
			2
		}
		else
		{
			3
		};
		
		if _type == 3
		{
			return;
		}
		
		if !self.skip_prompt_for_recalculating_offsets
		{
			if _type == 1
			{
				let params = nwg::MessageParams
				{
					title: "Entry Offsets Require Increase",
					content: &format!("At least one entry requires it's offset increased.\n\nThis is because the directory size has reached a higher 2,048 boundary.\n\nRecalculate all entry offsets?"),
					buttons: nwg::MessageButtons::YesNoCancel,
					icons: nwg::MessageIcons::Warning
				};
				let result = self.gui.message(&params);
				match result
				{
					MessageChoice::Yes =>
					{},
					MessageChoice::No =>
					{
						return;
					},
					MessageChoice::Cancel =>
					{
						return;
					},
					_ => {}
				}
			}
			else if _type == 2
			{
				let params = nwg::MessageParams
				{
					title: "Entry Offsets Optional Decrease",
					content: &format!("At least one entry can optionally have it's offset decreased.\n\nThis is because the directory size has reached a lower 2,048 boundary.\n\nRecalculate all entry offsets?"),
					buttons: nwg::MessageButtons::YesNoCancel,
					icons: nwg::MessageIcons::Warning
				};
				let result = self.gui.message(&params);
				match result
				{
					MessageChoice::Yes =>
					{},
					MessageChoice::No =>
					{
						return;
					},
					MessageChoice::Cancel =>
					{
						return;
					},
					_ => {}
				}
			}
		}
		
		self.format.recalculate_entry_offsets();
		self.entry_offsets_were_recalculated = true;
		
		self.log(&format!("Recalculated Offsets"));
		
		self.set_edited(true);
	}
	
	// date modified
	pub fn check_for_modified_img_file(&mut self)
	{
		if !self.is_open()
		{
			return;
		}
		
		let file_date_modified = self.get_img_file_last_modified();
		if self.img_file_last_modified.unwrap() != file_date_modified
		{
			let params = nwg::MessageParams
			{
				title: "IMG File Has Changed",
				content: &format!("IMG file on disk has changed.\n\nReload from disk?"),
				buttons: nwg::MessageButtons::YesNoCancel,
				icons: nwg::MessageIcons::Error
			};
			let result = self.gui.message(&params);
			match result
			{
				MessageChoice::Yes =>
				{
					self.reopen();
					return;
				},
				MessageChoice::No =>
				{
				},
				MessageChoice::Cancel =>
				{
				},
				_ => {}
			}
			
			self.set_edited(true);
			
			self.img_file_on_disk_edited = true;
			self.img_file_last_modified = Some(file_date_modified);
		}
	}
	
	fn get_img_file_last_modified(&mut self) -> SystemTime
	{
		let time = if self.img_file_last_modified == None
		{
			std::time::SystemTime::now()
		}
		else
		{
			self.img_file_last_modified.unwrap()
		};
		utility::get_file_last_modified(self.format.img_path_in.clone(), time)
	}
	
	// window title
	fn update_window_title(&mut self)
	{
		let window_handle = self.gui.app.window.handle.hwnd().unwrap();
		
		if self.is_open()
		{
			unsafe
			{
				nwg::win32::window_helper::set_window_text(window_handle, &format!("{} - {}", WINDOW_TITLE_BASE, utility::get_file_name(&self.format.img_path_in).unwrap()));
			}
		}
		else
		{
			unsafe
			{
				nwg::win32::window_helper::set_window_text(window_handle, &format!("{}", WINDOW_TITLE_BASE));
			}
		}
	}
	
	// selected entries
	pub fn recalculate_selected_entry_count(&mut self)
	{
		let mut sel_count = 0u64;
		let j = self.format.entries.len();
		for i in 0..j
		{
			let entry = self.format.entries[i].clone();
			
			if self.gui.app.main_entries.is_item_selected(entry.index as i32)
			{
				sel_count += 1;
			}
		}
		self.selected_entry_count = sel_count;
		
		if sel_count == 0
		{
			self.on_no_rows_selected();
		}
		else
		{
			self.on_at_least_one_row_selected();
		}
	}
	
	fn get_selected_entries(&mut self) -> Vec<Entry>
	{
		let mut selected_entries = Vec::new();
		
		let j = self.format.entries.len();
		for i in 0..j
		{
			if self.gui.app.main_entries.is_item_selected(i as i32)
			{
				selected_entries.push(self.format.entries[i].clone());
			}
		}
		
		selected_entries
	}
	
	// entry extension counts
	pub fn update_entry_extension_counts(&mut self)
	{
		let mut ext_counts = HashMap::new();
		
		let j = self.format.entries.len();
		for i in 0..j
		{
			let mut entry = self.format.entries[i].clone();
			
			let ext = entry.get_extension().to_uppercase();
			
			if ext_counts.contains_key(&ext)
			{
				ext_counts.insert(ext.clone(), ext_counts.get(&ext).unwrap() + 1);
			}
			else
			{
				ext_counts.insert(ext.clone(), 1);
			}
		}
		
		let mut text : String = String::from("");
		let mut i = 0i32;
		for (key, value) in ext_counts
		{
			if i != 0
			{
				text.push_str("\r\n");
			}
			
			if key.len() == 0
			{
				text.push_str("(No-Ext)");
			}
			else
			{
				text.push_str(&key);
			}
			
			text.push_str(" ");
			text.push_str(&value.to_formatted_string(&Locale::en));
			
			i += 1;
		}
		
		self.gui.app.entry_extension_counts.set_text(&text);
	}
	
	// stats
	pub fn update_all_stats(&mut self)
	{
		self.update_overlapping_entries();
		self.update_entry_gaps();
		self.update_blank_entries();
		self.update_missing_entries();
		self.update_duplicate_entry_name_count();
		
		self.update_entry_extension_counts();
	}
	
	fn reset_all_stats(&mut self)
	{
		self.reset_overlapping_entries();
		self.reset_entry_gaps();
		self.reset_blank_entries();
		self.reset_missing_entries();
		self.reset_duplicate_entry_name_count();
	}
	
	// overlapping entries
	fn get_overlapping_entry_count(&mut self) -> u64
	{
		let test_overlap = |entry: &Entry| -> bool
		{
			let index1 = entry.index;
			let offset = entry.offset_out;
			let size = entry.size;
			let offset2 = offset + size;
			
			for entry2 in self.format.entries.iter()
			{
				if index1 != entry2.index && cmp::max(offset, entry2.offset_out) < cmp::min(offset2, entry2.offset_out + entry2.size)
				{
					return true;
				}
			}
			false
		};
		
		let mut overlap_count = 0u64;
		
		for i in 0..self.format.entries.len()
		{
			if test_overlap(&self.format.entries[i])
			{
				overlap_count += 1;
			}
		}
		
		overlap_count
	}
	
	fn update_overlapping_entries(&mut self)
	{
		let overlap_count = self.get_overlapping_entry_count();
		self.gui.app.overlapping_entries.set_text(&format!("{} overlapping entries", overlap_count.to_formatted_string(&Locale::en)));
	}
	
	fn reset_overlapping_entries(&mut self)
	{
		self.gui.app.overlapping_entries.set_text(&format!("0 overlapping entries"));
	}
	
	// entry gaps
	fn get_entry_gap_count(&mut self) -> u64
	{
		let test_gap = |entry: &Entry| -> bool
		{
			let index1 = entry.index;
			let offset = entry.offset_out;
			let size = entry.size;
			let offset2 = offset + size;
			
			for entry2 in self.format.entries.iter()
			{
				if index1 != entry2.index && cmp::max(offset, entry2.offset_out) < cmp::min(offset2, entry2.offset_out + entry2.size)
				{
					return true;
				}
			}
			false
		};
		
		let mut gap_count = 0u64;
		
		for i in 0..self.format.entries.len()
		{
			if test_gap(&self.format.entries[i])
			{
				gap_count += 1;
			}
		}
		
		gap_count
	}
	
	fn update_entry_gaps(&mut self)
	{
		let gap_count = self.get_entry_gap_count();
		self.gui.app.entry_gaps.set_text(&format!("{} entry gaps", gap_count.to_formatted_string(&Locale::en)));
	}
	
	fn reset_entry_gaps(&mut self)
	{
		self.gui.app.entry_gaps.set_text(&format!("0 entry gaps"));
	}
	
	// blank entries
	fn get_blank_entry_count(&mut self) -> u64
	{
		let mut blank_entries = 0u64;
		
		for i in 0..self.format.entries.len()
		{
			if self.format.entries[i].size == 0
			{
				blank_entries += 1;
			}
		}
		
		blank_entries
	}
	
	fn update_blank_entries(&mut self)
	{
		let blank_entries = self.get_blank_entry_count();
		self.gui.app.blank_entries.set_text(&format!("{} blank entries", blank_entries.to_formatted_string(&Locale::en)));
	}
	
	fn reset_blank_entries(&mut self)
	{
		self.gui.app.blank_entries.set_text(&format!("0 blank entries"));
	}
	
	// missing entries
	fn get_missing_entries(&mut self) -> u64
	{
		if self.format.is_new()
		{
			return 0;
		}
		
		let mut entry_count_with_missing_data = 0u64;
		let img_file_size = crate::editor::utility::get_file_size(self.format.img_path_in.clone());
		
		let entries : Vec<Entry> = self.format.get_entries_sorted_by_offset_out();
		let total_entry_count = entries.len() as u64;
		let mut i = 0u64;
		for mut entry in entries
		{
			if entry.offset_in as u64 >= img_file_size
			{
				return total_entry_count - i;
			}
			
			i += 1;
		}
		
		0
	}
	
	fn update_missing_entries(&mut self)
	{
		let entry_count_with_missing_data = self.get_missing_entries();
		self.gui.app.missing_entries.set_text(&format!("{} entries missing data", entry_count_with_missing_data.to_formatted_string(&Locale::en)));
	}
	
	fn reset_missing_entries(&mut self)
	{
		self.gui.app.missing_entries.set_text(&format!("0 entries missing data"));
	}
	
	// duplicate entry names
	fn get_duplicate_name_count(&mut self) -> u64
	{
		let mut set = HashSet::new();
		let mut dupe_count = 0u64;
		
		let j = self.format.entries.len();
		for i in 0..j
		{
			let entry = self.format.entries[i].clone();
			
			let name2 = unsafe { vendor::str_from_u8_nul_utf8_unchecked(&entry.name) };
			let name3 = name2.to_uppercase();
			
			if set.contains(&name3)
			{
				dupe_count += 1;
			}
			else
			{
				set.insert(name3);
			}
		}
		dupe_count
	}
	
	fn update_duplicate_entry_name_count(&mut self)
	{
		let dupe_name_count = self.get_duplicate_name_count();
		self.gui.app.duplicate_entry_names.set_text(&format!("{} duplicate entry names", dupe_name_count));
	}
	
	fn reset_duplicate_entry_name_count(&mut self)
	{
		self.gui.app.duplicate_entry_names.set_text(&format!("0 duplicate entry names"));
	}
	
	// events
	fn bind_events(&mut self)
	{
		use nwg::Event as E;
		
		let evt_ui = self.gui.app.clone();
		//let evt_ui_rename = self.gui.rename.clone();
		
		let handle_events = move |evt, _evt_data, handle| {
			match evt {
				E::OnButtonClick =>
					if &handle == &evt_ui.new
					{
						get_editor().new();
					}
					else if &handle == &evt_ui.open
					{
						get_editor().open();
					}
					else if &handle == &evt_ui.save
					{
						get_editor().save();
					}
					else if &handle == &evt_ui.close
					{
						get_editor().close();
					}
					
					else if &handle == &evt_ui.add
					{
						get_editor().add();
					}
					else if &handle == &evt_ui.remove
					{
						get_editor().remove();
					}
					else if &handle == &evt_ui.replace
					{
						get_editor().replace();
					}
					else if &handle == &evt_ui.export
					{
						get_editor().export();
					}
					else if &handle == &evt_ui.undo
					{
						get_editor().undo();
					}
					else if &handle == &evt_ui.redo
					{
						get_editor().redo();
					}
					
					else if &handle == &evt_ui.select_all
					{
						get_editor().select_all();
					}
					else if &handle == &evt_ui.select_inverse
					{
						get_editor().select_inverse();
					}
					else if &handle == &evt_ui.recalculate_offsets
					{
						get_editor().recalculate_offsets();
					}
					else if &handle == &evt_ui.credits
					{
						get_editor().credits();
					}
					
					else if &handle == &evt_ui.rename
					{
						get_editor().rename();
					}
					else if &handle == &evt_ui._move
					{
						get_editor()._move();
					}
					
					// rename window
					/*
					else if &handle == &evt_ui_rename.ok_button
					{
						
					}
					*/
				E::OnTextInput =>
					if &handle == &evt_ui.include_search_box
					{
						get_editor().gui.readd_entries_to_list();
					}
					else if &handle == &evt_ui.exclude_search_box
					{
						get_editor().gui.readd_entries_to_list();
					}
				E::OnComboxBoxSelection =>
					if &handle == &evt_ui.img_version_combo
					{
						get_editor().set_img_version();
					}
				E::OnWindowClose =>
					{
						if &handle == &evt_ui.window
						{
							get_editor().check_to_save(false);
							get_editor().format.remove_temp_dir();
							
							std::process::exit(0);
						}
					}
				_ => {}
			}
		};
		
		nwg::full_bind_event_handler(&self.gui.app.window.handle, handle_events);
		
		let handler_id = 0x10000;     // handler ids equal or smaller than 0xFFFF are reserved by NWG
		nwg::bind_raw_event_handler(&self.gui.app.window.handle, handler_id, move |_hwnd, msg, _w, _l| {
			if msg == WM_NOTIFY// && _hwnd == self.gui.app.main_entries.handle //LVN_ITEMCHANGING//WM_NOTIFY
			{
				unsafe
				{
					let data = {
						let notif_ptr: *mut NMLISTVIEW = mem::transmute(_l);
						&*notif_ptr
					};
					
					if data.hdr.code == LVN_ITEMCHANGING
					{
						if ((*data).uChanged & LVIF_STATE) == LVIF_STATE
						{
							if ((*data).uOldState & LVIS_SELECTED) == 0 && ((*data).uNewState & LVIS_SELECTED) == LVIS_SELECTED
							{
								get_editor().on_select_entry();
							}
							else if ((*data).uOldState & LVIS_SELECTED) == LVIS_SELECTED && ((*data).uNewState & LVIS_SELECTED) == 0
							{
								get_editor().on_unselect_entry();
							}
						}
					}
				}
			}
			else if msg == WM_SETFOCUS
			{
				get_editor().check_for_modified_img_file();
			}
			None
		});
	}
}

