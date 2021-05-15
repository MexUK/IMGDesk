use std::fs;
use std::fs::File;
use std::io::BufReader;






#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Entry
{
	pub index: u32,
	pub name: [u8; 24],
	pub offset_in: u32,
	pub offset_out: u32,
	pub size: u32,
	pub data_temp_path: String,
	
	pub resource_type: u32,
	pub flags: u16
}

impl Entry
{
	// entry data
	pub fn set_data(&mut self, data: Vec<u8>) -> bool
	{
		super::super::get_editor().format.check_to_remove_entry_data(self);
		
		self.data_temp_path = unsafe
		{
			super::super::utility::get_next_file_path2(super::super::get_editor().format.get_temp_dir().clone(), super::super::vendor::str_from_u8_nul_utf8_unchecked(&self.name).to_string())
		};
		
		if !super::super::utility::set_file_data(self.data_temp_path.clone(), &data)
		{
			return false;
		}
		
		self.offset_out = super::super::utility::to_sector_bytes(super::super::get_editor().format.get_next_lowest_offset_excluding_entry(data.len() as u64, self.index) as u64) as u32;
		self.size = super::super::utility::to_sector_bytes(data.len() as u64) as u32;
		
		return true;
	}
	
	pub fn get_data_with_reader(&mut self, mut reader: &mut BufReader<File>) -> Vec<u8>
	{
		if self.data_temp_path.is_empty()
		{
			super::super::utility::get_file_data_range_with_reader(&mut reader, self.offset_in as u64, self.size as u64)
		}
		else
		{
			super::super::utility::get_file_data(self.data_temp_path.clone())
		}
	}
	
	pub fn get_data(&mut self) -> Vec<u8>
	{
		super::super::utility::get_file_data_range(super::super::get_editor().format.img_path_in.clone(), self.offset_in as u64, self.size as u64)
	}
	
	// entry name
	pub fn set_name(&mut self, new_entry_name: &String)
	{
		let mut new_entry_name_padded : String = new_entry_name.clone();
		
		let start = new_entry_name.len();
		for i in start..24
		{
			new_entry_name_padded.push(0 as char);
		}
		
		self.name = crate::editor::vendor::clone_into_array(new_entry_name_padded.as_bytes());
		
		super::super::get_editor().gui.on_entry_change(self);
		super::super::get_editor().on_rename_entry();
	}
	
	// entry index
	pub fn set_index(&mut self, mut new_entry_index: u64)
	{
		let entry = crate::editor::get_editor().format.entries.remove(self.index as usize);
		crate::editor::get_editor().format.entries.insert(new_entry_index as usize, entry);
		self.index = new_entry_index as u32;
		
		crate::editor::get_editor().on_change_entry_index();
		
		crate::editor::get_editor().gui.app.main_entries.ensure_visible(new_entry_index as i32);
	}
	
	// entry extension
	pub fn get_extension(&mut self) -> String
	{
		let ext = super::super::vendor::get_extension_from_filename(unsafe { super::super::vendor::str_from_u8_nul_utf8_unchecked(&self.name) });
		match ext
		{
			None => String::from(""),
			_ => ext.unwrap().to_string()
		}
	}
	
	// entry offset
	pub fn get_offset_in_sectors(&self) -> u64
	{
		super::super::utility::to_sectors(self.offset_in as u64)
	}
	
	pub fn get_offset_out_sectors(&self) -> u64
	{
		super::super::utility::to_sectors(self.offset_out as u64)
	}
	
	// entry size
	pub fn get_size_sectors(&self) -> u64
	{
		super::super::utility::to_sectors(self.size as u64)
	}
}