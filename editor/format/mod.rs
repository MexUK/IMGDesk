extern crate dirs;

use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::str;
use std::path::Path;
use std::process;

pub mod format_detector;
pub mod entry;
pub mod img;
pub mod rpf;


use entry::Entry as Entry;






const LOCAL_DATA_FOLDER_NAME		: &str = "IMG Desk";

const TEMP_FOLDER_NAME				: &str = "Temp";

const TEMP_SAVING_FOLDER_NAME		: &str = "Saving";
const TEMP_ENTRYDATA_FOLDER_NAME	: &str = "EntryData";
const TEMP_UNDO_FOLDER_NAME			: &str = "Undo";
const TEMP_NEW_FOLDER_NAME			: &str = "New";




pub enum FormatType
{
	UNKNOWN,
	IMG,
	RPF
}





pub struct Format
{
	pub dir_path_in: String,
	pub img_path_in: String,
	pub entries: Vec<Entry>,
	pub img_version: u8,
	pub img_encrypted: bool
}

impl Default for Format
{
	fn default() -> Self
	{
		Self
		{
			dir_path_in: Default::default(),
			img_path_in: Default::default(),
			entries: Default::default(),
			img_version: 0,
			img_encrypted: false
		}
	}
}

impl Format
{
	pub fn new(&mut self, img_path_in: &str, dir_path_in: &str)
	{
		self.init_working_dir();
		
		self.img_path_in = img_path_in.to_string();
		self.dir_path_in = dir_path_in.to_string();
		
		self.img_version = 1;
		self.img_encrypted = false;
	}
	
	pub fn parse(&mut self, img_path_in: &str, dir_path_in: &str)
	{
		self.init_working_dir();
		
		let (format, version, img_encrypted) = format_detector::detect_version(&img_path_in.to_string());
		
		match format
		{
			FormatType::IMG => match version
			{
				1 => img::version1::parse_list(self, img_path_in, dir_path_in),
				2 => img::version2::parse_list(self, img_path_in),
				3 => match img_encrypted
				{
					false => img::version3_unencrypted::parse_list(self, img_path_in),
					true => img::version3_encrypted::parse_list(self, img_path_in),
					_ => {}
				},
				_ => {}
			},
			FormatType::RPF => match version
			{
				2 => rpf::version2::parse_list(self, img_path_in), // GTA IV
				//7 => rpf::version7::parse_list(self, img_path_in), // GTA V
				_ => {}
			},
			_ => {}
		}
		
		self.img_version = version;
		self.img_encrypted = img_encrypted;
	}
	
	pub fn save(&mut self, img_path_out: &str, dir_path_out: &str)
	{
		match self.img_version
		{
			1 => img::version1::save_list(self, img_path_out, dir_path_out),
			2 => img::version2::save_list(self, img_path_out),
			3 => match self.img_encrypted
			{
				false => img::version3_unencrypted::save_list(self, img_path_out),
				true => img::version3_encrypted::save_list(self, img_path_out),
				_ => {}
			},
			_ => {}
		}
		
		for mut entry in self.entries.iter_mut()
		{
			entry.offset_in = entry.offset_out;
		}
	}
	
	pub fn reset(&mut self)
	{
		self.dir_path_in = String::from("");
		self.img_path_in = String::from("");
		self.img_version = 0;
		self.img_encrypted = false;
		self.entries = Vec::new();
	}
	
	pub fn init_working_dir(&mut self)
	{
		let saving_dir : String = self.get_saving_dir();
		let entry_data_dir : String = self.get_entry_data_dir();
		let undo_dir : String = self.get_undo_dir();
		let new_dir : String = self.get_new_dir();
		
		fs::create_dir_all(saving_dir);
		fs::create_dir_all(entry_data_dir);
		fs::create_dir_all(undo_dir);
		fs::create_dir_all(new_dir);
	}
	
	fn get_working_dir(&mut self) -> String
	{
		let base1 = dirs::data_local_dir().unwrap();
		let base = base1.to_str().unwrap();
		format!("{}/{}/", base, LOCAL_DATA_FOLDER_NAME)
	}
	
	fn get_temp_dir(&mut self) -> String
	{
		let base = self.get_working_dir();
		format!("{}{}/{}/", base, TEMP_FOLDER_NAME, process::id().to_string().as_str())
	}
	
	fn get_saving_dir(&mut self) -> String
	{
		let base = self.get_temp_dir();
		format!("{}{}/", base, TEMP_SAVING_FOLDER_NAME)
	}
	
	fn get_entry_data_dir(&mut self) -> String
	{
		let base = self.get_temp_dir();
		format!("{}{}/", base, TEMP_ENTRYDATA_FOLDER_NAME)
	}
	
	pub fn get_undo_dir(&mut self) -> String
	{
		let base = self.get_temp_dir();
		format!("{}{}/", base, TEMP_UNDO_FOLDER_NAME)
	}
	
	pub fn get_new_dir(&mut self) -> String
	{
		let base = self.get_temp_dir();
		format!("{}{}/", base, TEMP_NEW_FOLDER_NAME)
	}
	
	pub fn remove_temp_dir(&mut self)
	{
		if !self.get_temp_dir().contains("IMG-DIR-Editor")
		{
			return;
		}
		
		if !self.get_temp_dir().contains("Temp")
		{
			return;
		}
		
		if !Path::is_dir(Path::new(&self.get_temp_dir()))
		{
			return;
		}
		
		fs::remove_dir_all(self.get_temp_dir());
	}
	
	fn get_next_lowest_offset(&mut self, new_data_size: u64) -> i64
	{
		if self.entries.len() == 0
		{
			crate::editor::utility::to_sector_bytes(self.get_img_header_size() + self.get_img_directory_entry_size()) as i64
		}
		else
		{
			// fetch offset and size for all entries
			let mut ranges = Vec::new();
			
			let j = self.entries.len();
			for i in 0..j
			{
				let entry = self.entries[i].clone();
				
				ranges.push((entry.offset_out as i64, entry.size));
			}
			
			// sort offset and size vector by offset
			ranges.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
			
			// check if new file size will fit before any existing entry
			let mut offset : i64 = self.get_entry_data_offset() as i64;
			for (offset2, size2) in &ranges
			{
				let gap_size = (*offset2) - offset;
				
				if (new_data_size as i64) <= gap_size
				{
					return offset;
				}
				
				offset = (*offset2 + ((*size2) as i64));
			}
			
			// add after last entry
			offset
		}
	}
	
	fn get_next_lowest_offset_excluding_entry(&mut self, new_data_size: u64, exclude_entry_index: u32) -> i64
	{
		if self.entries.len() == 0
		{
			0i64
		}
		else
		{
			// fetch offset and size for all entries
			let mut ranges = Vec::new();
			
			let j = self.entries.len();
			for i in 0..j
			{
				let entry = self.entries[i].clone();
				
				if entry.index == exclude_entry_index
				{
					continue;
				}
				
				ranges.push((entry.offset_out, entry.size));
			}
			
			// sort offset and size vector by offset
			ranges.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
			
			// check if new file size will fit before any existing entry
			let mut offset : i64 = self.get_entry_data_offset() as i64;
			for (offset2, size2) in &ranges
			{
				let gap_size = ((*offset2) as i64) - offset;
				
				if (new_data_size as i64) <= gap_size
				{
					return offset;
				}
				
				offset = (*offset2 + size2) as i64;
			}
			
			// add after last entry
			offset
		}
	}
	
	pub fn get_entry_offsets(&mut self) -> Vec<u64>
	{
		let mut entry_offsets : Vec<u64> = Vec::new();
		
		for entry in self.entries.iter()
		{
			entry_offsets.push(entry.offset_out as u64);
		}
		
		entry_offsets
	}
	
	pub fn recalculate_entry_offsets(&mut self)
	{
		let mut offset : u64 = self.get_entry_data_offset();
		
		for mut entry in self.entries.iter_mut()
		{
			entry.offset_out = offset as u32;
			
			offset += crate::editor::utility::to_sector_bytes(entry.size as u64);
		}
		
		crate::editor::get_editor().on_entry_offsets_change();
	}
	
	pub fn set_entry_offsets(&mut self, entry_offsets: &Vec<u64>)
	{
		let j = self.entries.len();
		for i in 0..j
		{
			self.entries[i].offset_out = entry_offsets[i] as u32;
		}
		
		crate::editor::get_editor().on_entry_offsets_change();
	}
	
	pub fn get_entry_data_offset(&mut self) -> u64
	{
		let header_size = self.get_img_header_size();
		let directory_size = self.get_img_directory_entry_size();
		let names_size = self.get_img_names_size();
		
		let first_entry_offset = header_size + directory_size + names_size;
		crate::editor::utility::to_sector_bytes(first_entry_offset)
	}
	
	pub fn get_img_header_size(&mut self) -> u64
	{
		match self.img_version
		{
			1 => 0,
			2 => 8,
			3 => 20,
			_ => 0
		}
	}
	
	pub fn get_img_directory_entry_size(&mut self) -> u64
	{
		match self.img_version
		{
			1 => 32,
			2 => 32,
			3 => 16,
			_ => 32
		}
	}
	
	pub fn get_img_directory_size(&mut self) -> u64
	{
		self.get_img_directory_entry_size() * (self.entries.len() as u64)
	}
	
	pub fn get_img_names_size(&mut self) -> u64
	{
		match self.img_version
		{
			1 => 0,
			2 => 0,
			3 => self.get_names_len_for_v3(),
			_ => 0
		}
	}
	
	pub fn add_file(&mut self, path: String) -> Entry
	{
		self.add_file_at(path, -1, String::from(""))
	}
	
	pub fn add_file_at(&mut self, path: String, entry_index: i32, entry_name: String) -> Entry
	{
		let entry_name2 = if entry_name.is_empty()
		{
			super::utility::get_file_name(path.as_str()).unwrap().to_string()
		}
		else
		{
			entry_name
		};
		self.add_data_at(entry_name2, super::utility::get_file_data(path), entry_index)
	}
	
	pub fn add_data(&mut self, name: String, data: Vec<u8>) -> Entry
	{
		self.add_data_at(name, data, -1)
	}
	
	pub fn add_data_at(&mut self, name: String, data: Vec<u8>, entry_index: i32) -> Entry
	{
		let data_temp_path : String =
		{
			super::utility::get_next_file_path2(self.get_entry_data_dir(), name.clone())
		};
		
		let mut name2 = name.clone();
		for _i in name2.len()..24
		{
			name2.push(0 as char);
		}
		
		let entry_offset = self.get_next_lowest_offset(data.len() as u64);
		
		let offset = crate::editor::utility::to_sector_bytes(entry_offset as u64) as u32;
		
		let entry = Entry
		{
			index: self.entries.len() as u32,
			offset_in: offset,
			offset_out: offset,
			size: crate::editor::utility::to_sector_bytes(data.len() as u64) as u32,
			name: super::vendor::clone_into_array(&name2.as_bytes()[0..24]),
			data_temp_path: data_temp_path.clone(),
			resource_type: 0, // todo
			flags: 0 // todo
		};
		
		if entry_index == -1
		{
			self.entries.push(entry);
		}
		else
		{
			self.entries.insert(entry_index as usize, entry);
		}
		
		crate::editor::utility::set_file_data(data_temp_path, &data);
		
		if entry_index == -1
		{
			let len = self.entries.len();
			self.entries[len - 1].clone()
		}
		else
		{
			self.entries[entry_index as usize].clone()
		}
	}
	
	pub fn replace_file(&mut self, file_path: String) -> Entry
	{
		self.replace_file_at(file_path, -1, String::from(""))
	}
	
	pub fn replace_file_at(&mut self, file_path: String, entry_index: i32, entry_name: String) -> Entry
	{
		let file_name : String = if entry_name.is_empty()
		{
			crate::editor::utility::get_file_name(file_path.as_str()).unwrap().to_string()
		}
		else
		{
			entry_name
		};
		
		
		
		/*
		let data_temp_path : String =
		{
			crate::editor::utility::get_next_file_path2(self.get_entry_data_dir(), file_name.clone())
		};
		
		let entry_index2 = if entry_index == -1
		{
			self.get_entry_by_name(file_name.clone()).unwrap().index
		}
		else
		{
			entry_index as u32
		};
		let entry_offset = self.get_next_lowest_offset_excluding_entry(new_file_data.len() as u64, entry_index2);
		*/
		
		let mut entry : &mut Entry = if entry_index == -1
		{
			self.get_entry_by_name(file_name.clone()).unwrap()
		}
		else
		{
			self.get_entry_by_index(entry_index as u64).unwrap()
		};
		
		let new_file_data : Vec<u8> = crate::editor::utility::get_file_data(file_path.clone());
		entry.set_data(new_file_data);
		
		/*
		//self.check_to_remove_entry_data(entry);
		if !entry.data_temp_path.is_empty()
		{
			fs::remove_file(entry.data_temp_path.as_str());
			entry.data_temp_path = String::from("");
		}
		
		{
			entry.data_temp_path = data_temp_path.clone();
			entry.size = crate::editor::utility::to_sector_bytes(new_file_data.len() as u64) as u32;
			entry.offset_out = crate::editor::utility::to_sector_bytes(entry_offset as u64) as u32;
		}
		
		crate::editor::utility::set_file_data(data_temp_path, &new_file_data);
		*/
		
		entry.clone()
	}
	
	pub fn remove(&mut self, entry: &Entry)
	{
		self.check_to_remove_entry_data(&mut entry.clone());
		
		let index : usize = self.get_index_by_entry(entry).unwrap() as usize;
		self.entries.remove(index);
	}
	
	fn check_to_remove_entry_data(&mut self, entry: &mut Entry)
	{
		if !entry.data_temp_path.is_empty()
		{
			fs::remove_file(entry.data_temp_path.as_str());
			entry.data_temp_path = String::from("");
		}
	}
	
	pub fn export_entry(&mut self, folder_path: &str, entry: &mut Entry)
	{
		let mut file_path : String = folder_path.to_string();
		let c : char = file_path.chars().last().unwrap();
		if c != '/' && c != '\\'
		{
			file_path.push_str("/");
		}
		file_path.push_str(unsafe { super::vendor::str_from_u8_nul_utf8_unchecked(&entry.name) });
		
		super::utility::set_file_data_no_overwrite(file_path, &self.get_entry_data(entry));
	}
	
	pub fn get_entry_by_name(&mut self, name: String) -> Option<&mut Entry>
	{
		for mut entry in self.entries.iter_mut()
		{
			if name == unsafe { super::vendor::str_from_u8_nul_utf8_unchecked(&entry.name) }
			{
				return Some(entry);
			}
		}
		None
	}
	
	pub fn get_entry_by_index(&mut self, index: u64) -> Option<&mut Entry>
	{
		let mut counter : u64 = 0;
		for mut entry in self.entries.iter_mut()
		{
			if index == counter
			{
				return Some(entry);
			}
			counter += 1;
		}
		None
	}
	
	pub fn get_index_by_entry(&mut self, entry2: &Entry) -> Option<u64>
	{
		let mut counter : u64 = 0;
		for entry in self.entries.iter()
		{
			if entry == entry2
			{
				return Some(counter);
			}
			counter += 1;
		}
		None
	}
	
	pub fn get_entry_data(&mut self, entry: &Entry) -> Vec<u8>
	{
		if entry.data_temp_path.is_empty()
		{
			super::utility::get_file_data_range(self.img_path_in.clone(), entry.offset_in as u64, entry.size as u64)
		}
		else
		{
			super::utility::get_file_data(entry.data_temp_path.clone())
		}
	}
	
	pub fn get_entry_data_by_index(&mut self, entry_index: u64) -> Vec<u8>
	{
		let entry = &self.entries[entry_index as usize];
		if entry.data_temp_path.is_empty()
		{
			super::utility::get_file_data_range(self.img_path_in.clone(), entry.offset_in as u64, entry.size as u64)
		}
		else
		{
			super::utility::get_file_data(entry.data_temp_path.clone())
		}
	}
	
	pub fn get_entry_data_by_index_with_reader(&mut self, entry_index: u64, mut reader: &mut BufReader<File>) -> Vec<u8>
	{
		let mut entry = self.entries[entry_index as usize].clone();
		entry.get_data_with_reader(&mut reader)
	}
	
	pub fn reassign_entry_indices(&mut self)
	{
		let mut index = 0;
		for mut entry in self.entries.iter_mut()
		{
			entry.index = index;
			index += 1;
		}
	}
	
	pub fn get_entry_count_by_name(&mut self, name: String) -> i32
	{
		let name4 = name.to_uppercase();
		
		let mut count = 0i32;
		for entry in self.entries.iter_mut()
		{
			let name2 = unsafe { super::vendor::str_from_u8_nul_utf8_unchecked(&entry.name) };
			let name3 = name2.to_uppercase();
			
			if name4 == name3
			{
				count += 1;
			}
		}
		
		count
	}
	
	pub fn get_entries_sorted_by_offset_out(&mut self) -> Vec<Entry>
	{
		let mut entries : Vec<Entry> = self.entries.clone();
		
		entries.sort_by(|a,b|a.offset_out.partial_cmp(&b.offset_out).unwrap());
		
		entries
	}
	
	pub fn get_names_len_for_v3(&mut self) -> u64
	{
		let mut len = self.entries.len();
		
		for entry in self.entries.iter()
		{
			let name = unsafe { super::vendor::str_from_u8_nul_utf8_unchecked(&entry.name) };
			
			len += name.len();
		}
		
		len as u64
	}
	
	pub fn is_new(&mut self) -> bool
	{
		self.img_path_in.is_empty()
	}
	
	pub fn set_version(&mut self, img_version: u8, img_encrypted: bool)
	{
		self.img_version = img_version;
		self.img_encrypted = img_encrypted;
		
		crate::editor::get_editor().on_img_version_change();
	}
}

