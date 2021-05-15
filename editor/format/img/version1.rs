use std::convert::TryInto;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;

// parse
pub fn parse_list(format: &mut super::super::Format, img_path_in: &str, dir_path_in: &str)
{
	let (mut buffer, metadata) = crate::editor::utility::get_file_data_with_meta(dir_path_in.to_string());
	
	let entry_count = metadata.len() / 32;
	
	format.img_path_in = img_path_in.clone().to_owned();
	format.dir_path_in = dir_path_in.clone().to_owned();
	format.entries = Vec::with_capacity(entry_count as usize);
	
	for i in 0..entry_count
	{
		format.entries.push(parse_entry(&mut buffer, i));
	}
}

pub fn parse_entry(buffer: &mut Vec<u8>, i: u64) -> super::super::entry::Entry
{
	let seek = (i * 32) as usize;
	let offset = crate::editor::utility::sectors_to_bytes(u32::from_le_bytes(buffer[seek..seek+4].try_into().unwrap()) as u64) as u32;
	let entry = super::super::entry::Entry
	{
		index: i as u32,
		offset_in: offset,
		offset_out: offset,
		size: crate::editor::utility::sectors_to_bytes(u32::from_le_bytes(buffer[seek+4..seek+8].try_into().unwrap()) as u64) as u32,
		name: crate::editor::vendor::clone_into_array(&buffer[seek+8..seek+32]),
		data_temp_path: String::from(""),
		resource_type: 0,
		flags: 0
	};
	entry
}

// save
pub fn save_list(format: &mut super::super::Format, img_path_out: &str, dir_path_out: &str)
{
	// IMG file
	{
		let img_uses_temp_path_out : bool = format.img_path_in != img_path_out;
		
		let img_temp_path_out : String = crate::editor::utility::get_next_file_path2(format.get_saving_dir(), "temp.img".to_string());
		let img_path_out2 = if img_uses_temp_path_out
		{
			img_temp_path_out
		}
		else
		{
			img_path_out.to_string()
		};
		
		let mut file_out = File::create(&img_path_out2).expect("no IMG file created/opened");
		let mut buffer_out = BufWriter::new(file_out);
		
		let count = format.entries.len();
		
		let mut seek : u64 = 0;
		
		let img_is_new = format.is_new();
		
		let mut file : Option<File> = None;
		let mut reader : Option<BufReader<File>> = None;
		
		if !img_is_new
		{
			file = Some(File::open(&format.img_path_in).unwrap());
			reader = Some(BufReader::new(file.unwrap()));
		}
		
		let entries : Vec<super::super::entry::Entry> = format.get_entries_sorted_by_offset_out();
		for mut entry in entries
		{
			let mut buffer = Vec::new();
			
			// pad entry gaps
			let entry_offset = entry.offset_out as u64;
			if seek < entry_offset
			{
				let diff = entry_offset - seek;
				crate::editor::utility::write_zeros(&mut buffer, diff);
				seek += diff as u64;
			}
			
			// push entry data
			let data : Vec<u8> = if img_is_new
			{
				entry.get_data()
			}
			else
			{
				entry.get_data_with_reader(&mut reader.as_mut().unwrap())
			};
			seek += data.len() as u64;
			buffer.extend(data);
			
			// pad entry data
			if buffer.len() % 2048 != 0
			{
				let remainder = 2048 - (buffer.len() % 2048);
				crate::editor::utility::write_zeros(&mut buffer, remainder as u64);
				seek += remainder as u64;
			}
			
			buffer_out.write_all(buffer.as_slice());
		}
		
		buffer_out.flush();
		
		if img_uses_temp_path_out
		{
			fs::remove_file(&img_path_out);
			fs::rename(img_path_out2, &img_path_out);
		}
		
		//let img_temp_path_out : String = crate::editor::utility::get_next_file_path2(format.get_saving_dir(), "temp.dir".to_string());
		//crate::editor::utility::set_file_data_overlap(img_path_out.to_string(), &buffer, img_uses_temp_path_out, img_temp_path_out.to_string());
	}
	
	// DIR file
	{
		let dir_uses_temp_path_out : bool = format.dir_path_in != dir_path_out;
		
		let dir_temp_path_out : String = crate::editor::utility::get_next_file_path2(format.get_saving_dir(), "temp.dir".to_string());
		let dir_path_out2 = if dir_uses_temp_path_out
		{
			dir_temp_path_out
		}
		else
		{
			dir_path_out.to_string()
		};
		
		let mut file_out = File::create(&dir_path_out2).expect("no DIR file created/opened");
		let mut buffer_out = BufWriter::new(file_out);
		
		let mut seek = 0;
		
		let mut buffer = Vec::with_capacity(32);
		buffer.resize(32, 0);
		
		for entry in format.entries.iter()
		{
			let bytes1 : [u8; 4] = (entry.get_offset_out_sectors() as u32).to_le_bytes();
			let bytes2 : [u8; 4] = (entry.get_size_sectors() as u32).to_le_bytes();
			
			buffer[0] = bytes1[0] as u8;
			buffer[1] = bytes1[1] as u8;
			buffer[2] = bytes1[2] as u8;
			buffer[3] = bytes1[3] as u8;
			
			buffer[4] = bytes2[0] as u8;
			buffer[5] = bytes2[1] as u8;
			buffer[6] = bytes2[2] as u8;
			buffer[7] = bytes2[3] as u8;
			
			for i2 in 0..24
			{
				buffer[8+i2] = entry.name[i2] as u8;
			}
			
			buffer_out.write_all(buffer.as_slice());
			buffer_out.flush();
		}
		
		if dir_uses_temp_path_out
		{
			fs::remove_file(&dir_path_out);
			fs::rename(dir_path_out2, &dir_path_out);
		}
		
		//let dir_temp_path_out : String = crate::editor::utility::get_next_file_path2(format.get_saving_dir(), "temp.dir".to_string());
		//crate::editor::utility::set_file_data_overlap(dir_path_out.to_string(), &buffer, dir_uses_temp_path_out, dir_temp_path_out.to_string());
	}
}