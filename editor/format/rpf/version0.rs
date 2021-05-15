use std::convert::TryInto;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::io::Seek;
use std::io::SeekFrom;

// parse
pub fn parse_list(format: &mut super::super::Format, img_path_in: &str)
{
	let mut file = File::open(&img_path_in).expect("no file found");
	let mut reader = BufReader::new(file);
	
	let mut buffer = [0; 12];
	reader.read_exact(&mut buffer);
	
	let file_stamp = u32::from_le_bytes(buffer[0..4].try_into().unwrap());
	let table_data_size = u32::from_le_bytes(buffer[4..8].try_into().unwrap());
	let entry_count = u32::from_le_bytes(buffer[8..12].try_into().unwrap());
	
	format.img_path_in = img_path_in.clone().to_owned();
	format.entries = Vec::with_capacity(entry_count as usize);
	
	reader.seek(SeekFrom::Start(2048));
	for i in 0..entry_count
	{
		//format.entries.push(parse_entry(&mut reader, i as u64));
	}
}

/*
pub fn parse_entry(reader: &mut BufReader<File>, i: u64) -> super::super::entry::Entry
{
	let mut buffer = [0; 16];
	reader.read_exact(&mut buffer);
	
	let rpf : Option<RpfEntry> = None;
	
	if buffer[3] == 0
	{
		// file
		let offset = crate::editor::utility::sectors_to_bytes(u32::from_le_bytes(buffer[4..8].try_into().unwrap()) as u64) as u32;
		let size = crate::editor::utility::sectors_to_bytes(u32::from_le_bytes(buffer[8..12].try_into().unwrap()) as u64) as u32;
		let uncompressed_size = crate::editor::utility::sectors_to_bytes(u32::from_le_bytes(buffer[12..16].try_into().unwrap()) as u64) as u32;
		
		rpf = Some(super::entry::RpfEntry
		{
			uncompressed_size: uncompressed_size,
			is_directory: false
		});
	}
	else
	{
		// directory
		rpf = Some(super::entry::RpfEntry
		{
			uncompressed_size: uncompressed_size,
			is_directory: true
		});
	}
	
	super::super::entry::Entry
	{
		index: i as u32,
		offset_in: offset,
		offset_out: offset,
		size: size,
		name: crate::editor::vendor::clone_into_array(&buffer[8..32]),
		data_temp_path: String::from(""),
		//rpf: rpf
	}
}
*/

// save
pub fn save_list(format: &mut super::super::Format, img_path_out: &str)
{
	// IMG file
	{
		let img_uses_temp_path_out : bool = format.img_path_in != img_path_out;
		
		let count = format.entries.len();
		
		let mut seek : usize = 0;
		
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
		
		let mut buffer = Vec::new();
		
		// header
		let count2 = count.to_le_bytes();
		buffer.push('V' as u8);
		buffer.push('E' as u8);
		buffer.push('R' as u8);
		buffer.push('2' as u8);
		buffer.push(count2[0]);
		buffer.push(count2[1]);
		buffer.push(count2[2]);
		buffer.push(count2[3]);
		seek += 8;
		buffer_out.write_all(buffer.as_slice());
		
		// directory
		buffer.resize(32, 0);
		for i in 0..count
		{
			let entry = format.get_entry_by_index(i as u64).unwrap();
			
			let bytes1 : [u8; 4] = (entry.get_offset_out_sectors() as u32).to_le_bytes();
			let bytes2 : [u8; 2] = (entry.get_size_sectors() as u16).to_le_bytes();
			
			buffer[0] = bytes1[0] as u8;
			buffer[1] = bytes1[1] as u8;
			buffer[2] = bytes1[2] as u8;
			buffer[3] = bytes1[3] as u8;
			
			buffer[4] = bytes2[0] as u8;
			buffer[5] = bytes2[1] as u8;
			
			buffer[6] = 0 as u8;
			buffer[7] = 0 as u8;
			
			for i2 in 0..24
			{
				buffer[8 + i2] = entry.name[i2] as u8;
			}
			
			seek += 32;
			buffer_out.write_all(buffer.as_slice());
		}
		
		// pad directory
		buffer.clear();
		let remainder2 = 8 + (count * 32);
		if remainder2 % 2048 != 0
		{
			let remainder = 2048 - (remainder2 % 2048);
			crate::editor::utility::write_zeros(&mut buffer, remainder as u64);
			seek += remainder;
			buffer_out.write_all(buffer.as_slice());
		}
		
		// entry data
		let img_is_new = format.is_new();
		
		let mut file : Option<File> = None;
		let mut reader : Option<BufReader<File>> = None;
		
		if img_is_new
		{
			file = Some(File::open(&format.img_path_in).unwrap());
			reader = Some(BufReader::new(file.unwrap()));
		}
		
		let entries : Vec<super::super::entry::Entry> = crate::editor::get_editor().format.get_entries_sorted_by_offset_out();
		for mut entry in entries
		{
			buffer.clear();
			
			// pad entry gaps
			let entry_offset = entry.offset_out as u64;
			if (seek as u64) < entry_offset
			{
				let diff = (entry_offset as u64) - (seek as u64);
				crate::editor::utility::write_zeros(&mut buffer, diff);
				seek += diff as usize;
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
			seek += data.len() as usize;
			buffer.extend(data);
			
			// pad entry data
			if buffer.len() % 2048 != 0
			{
				let remainder = 2048 - (buffer.len() % 2048);
				crate::editor::utility::write_zeros(&mut buffer, remainder as u64);
				seek += remainder;
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
}