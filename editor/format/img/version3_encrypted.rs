use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::io::Seek;
use std::io::SeekFrom;

use crypto::aes;
use crypto::buffer::WriteBuffer;

// parse
pub fn parse_list(format: &mut super::super::Format, img_path_in: &str)
{
	let mut file = File::open(&img_path_in).expect("no file found");
	let mut reader = BufReader::new(file);
	
	// read header
	let mut buffer = Vec::with_capacity(32);
	buffer.resize(32, 0);
	reader.read_exact(&mut buffer);
	
	// decrypt header
	//let mut buffer_decrypted = Vec::with_capacity(32*5);
	let mut buffer_decrypted = Vec::new();
	//buffer_decrypted.resize(32 as usize, 0);
	
	crate::editor::utility::decrypt_gta_4(&mut buffer, &mut buffer_decrypted);
	
	// parse header
	let seek = 0 as usize;
	let buf1 = [buffer_decrypted[0], buffer_decrypted[1], buffer_decrypted[2], buffer_decrypted[3]];
	let buf2 = [buffer_decrypted[4], buffer_decrypted[5], buffer_decrypted[6], buffer_decrypted[7]];
	let buf3 = [buffer_decrypted[8], buffer_decrypted[9], buffer_decrypted[10], buffer_decrypted[11]];
	let buf4 = [buffer_decrypted[12], buffer_decrypted[13], buffer_decrypted[14], buffer_decrypted[15]];
	let buf5 = [buffer_decrypted[16], buffer_decrypted[17]];
	let buf6 = [buffer_decrypted[18], buffer_decrypted[19]];
	
	let file_stamp = u32::from_le_bytes(buf1);
	let file_version = u32::from_le_bytes(buf2);
	let entry_count = u32::from_le_bytes(buf3);
	let mut table_data_size = u32::from_le_bytes(buf4);
	let table_item_data_size = u16::from_le_bytes(buf5);
	let unknown1 = u16::from_le_bytes(buf6);
	
	// decrypt directory
	let remainder = table_data_size % 16;
	table_data_size -= remainder;
	
	reader.seek(SeekFrom::Start(20));
	buffer.resize(table_data_size as usize, 0);
	reader.read_exact(&mut buffer);
	
	let mut buffer2 = Vec::new();
	if remainder != 0
	{
		buffer2.resize(remainder as usize, 0);
		reader.read_exact(&mut buffer2);
	}
	
	//let mut buffer_decrypted2 = Vec::with_capacity((table_data_size*5) as usize);
	//buffer_decrypted2.resize((table_data_size*5) as usize, 0);
	let mut buffer_decrypted2 = Vec::new();
	crate::editor::utility::decrypt_gta_4(&mut buffer, &mut buffer_decrypted2);
	
	if remainder != 0
	{
		buffer_decrypted2.extend(buffer2);
	}
	
	// parse directory I - entry offset, entry size
	//let table_entry_count = table_data_size / (table_item_data_size as u32);
	let mut seek = 0;
	
	format.entries = Vec::with_capacity(entry_count as usize);
	
	//buffer.resize(16 as usize, 0);
	for i in 0..entry_count
	{
		//reader.read_exact(&mut buffer);
		
		let buf1 = [buffer_decrypted2[seek], buffer_decrypted2[seek+1], buffer_decrypted2[seek+2], buffer_decrypted2[seek+3]];
		let buf2 = [buffer_decrypted2[seek+4], buffer_decrypted2[seek+5], buffer_decrypted2[seek+6], buffer_decrypted2[seek+7]];
		let buf3 = [buffer_decrypted2[seek+8], buffer_decrypted2[seek+9], buffer_decrypted2[seek+10], buffer_decrypted2[seek+11]];
		let buf4 = [buffer_decrypted2[seek+12], buffer_decrypted2[seek+13]];
		let buf5 = [buffer_decrypted2[seek+14], buffer_decrypted2[seek+15]];
		
		let item_size = u32::from_le_bytes(buf1);
		let resource_type = u32::from_le_bytes(buf2);
		let offset = u32::from_le_bytes(buf3);
		let size = u16::from_le_bytes(buf4);
		let flags = u16::from_le_bytes(buf5);
		
		let offset2 = crate::editor::utility::sectors_to_bytes(offset as u64) as u32;
		
		let entry = super::super::entry::Entry
		{
			index: i as u32,
			offset_in: offset2,
			offset_out: offset2,
			size: crate::editor::utility::sectors_to_bytes(size as u64) as u32,
			name: [0; 24],
			data_temp_path: String::from(""),
			resource_type: resource_type,
			flags: flags
		};
		format.entries.push(entry);
		
		seek += 16 as usize;
	}
	
	// parse directory II - entry names
	//buffer.resize(4096, 0);
	//let mut a = 0;
	for i in 0..entry_count
	{
		//let read = reader.read_until(0, &mut buffer).unwrap();
		
		//format.entries[i as usize].name = crate::editor::vendor::clone_into_array(&buffer[0..read]);
		
		let mut entry_name = crate::editor::utility::get_null_string(&mut buffer_decrypted2, seek as u64);
		let entry_name_len = entry_name.len();
		
		for _i in entry_name.len()..24
		{
			entry_name.push(0 as char);
		}
		
		format.entries[i as usize].name = crate::editor::vendor::clone_into_array(entry_name.as_bytes());
		
		seek += entry_name_len + 1;
		
		
		
		//a += entry_name_len as usize;
	}
	//a += entry_count as usize;
	
	format.img_path_in = img_path_in.clone().to_owned();
}

// save
pub fn save_list(format: &mut super::super::Format, img_path_out: &str)
{
	let img_uses_temp_path_out : bool = format.img_path_in != img_path_out;
	
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
	
	let mut unencrypted_buffer : Vec<u8> = Vec::new();
	let mut buffer : Vec<u8> = Vec::new();
	let mut names_buffer : Vec<u8> = Vec::new();
	
	// header
	let names_len_v3 = format.get_names_len_for_v3();
	let entry_count = format.entries.len();
	//let table_data_size = crate::editor::utility::to_sector_bytes(((16 * entry_count) + (names_len_v3 as usize)) as u64);
	let table_data_size = (16 * entry_count) + (names_len_v3 as usize);
	let body_start = 20 + (16 * entry_count) + (names_len_v3 as usize);
	
	buffer.extend(&0xA94E2A52u32.to_le_bytes());
	buffer.extend(&3u32.to_le_bytes());
	buffer.extend(&(entry_count as u32).to_le_bytes());
	buffer.extend(&(table_data_size as u32).to_le_bytes());
	buffer.extend(&16u16.to_le_bytes());
	buffer.extend(&0u16.to_le_bytes());
	
	seek += 20;
	unencrypted_buffer.extend(&buffer);
	
	// directory - most entry info
	for i in 0..entry_count
	{
		buffer.clear();
		
		let entry = format.get_entry_by_index(i as u64).unwrap();
		
		buffer.extend(&0u32.to_le_bytes());
		buffer.extend(&(entry.resource_type as u32).to_le_bytes());
		buffer.extend(&(entry.get_offset_out_sectors() as u32).to_le_bytes());
		buffer.extend(&(entry.get_size_sectors() as u16).to_le_bytes());
		
		let remainder = entry.size % 2048;
		let value = entry.flags | ((if remainder == 0 { 0 } else { 2048 - remainder }) as u16);
		buffer.extend(&(value as u16).to_le_bytes());
		
		seek += 16;
		unencrypted_buffer.extend(&buffer);
		
		let entry_name = unsafe
		{
			crate::editor::vendor::str_from_u8_nul_utf8_unchecked(&entry.name)
		};
		
		names_buffer.extend(entry_name.as_bytes());
		names_buffer.push(0);
	}
	
	// directory - entry names
	unencrypted_buffer.extend(&names_buffer);
	seek += names_buffer.len();
	names_buffer.clear();
	
	// padding after directory
	buffer.clear();
	
	// encrypt header and directory
	/*
	{
		let mut encrypted_buffer1 = Vec::new();
		let mut encrypted_buffer2 = Vec::new();
		
		let mut unencrypted_header = (&unencrypted_buffer[0..32]).to_vec();
		crate::editor::utility::encrypt_gta_4(&mut unencrypted_header, &mut encrypted_buffer1);
		let encrypted_header = &encrypted_buffer1[0..20];
		
		let mut unencrypted_directory = (&unencrypted_buffer[20..(unencrypted_buffer.len())]).to_vec();
		let pad_len = unencrypted_directory.len();
		crate::editor::utility::write_zeros(&mut unencrypted_directory, pad_len as u64);
		crate::editor::utility::encrypt_gta_4(&mut unencrypted_directory, &mut encrypted_buffer2);
		let encrypted_directory = &encrypted_buffer2[0..(unencrypted_buffer.len() - 20)];
		
		buffer_out.write_all(encrypted_header);
		buffer_out.write_all(encrypted_directory);
	}
	*/
	
	{
		let mut encrypted_buffer = Vec::new();
		
		let mut unencrypted_header = (&unencrypted_buffer[0..32]).to_vec();
		crate::editor::utility::encrypt_gta_4(&mut unencrypted_header, &mut encrypted_buffer);
		let encrypted_header = &encrypted_buffer[0..20];
		
		buffer_out.write_all(encrypted_header);
	}
	
	{
		let unencrypted_data_len = unencrypted_buffer.len() - 20;
		let encrypted_data_len = unencrypted_data_len - (unencrypted_data_len % 16);
		let mut unencrypted_directory = (&unencrypted_buffer[20..(20 + encrypted_data_len)]).to_vec();
		
		/*
		let pad_len = 20;//unencrypted_data_len - unencrypted_directory.len();
		if pad_len > 0
		{
			crate::editor::utility::write_zeros(&mut unencrypted_directory, pad_len as u64);
		}
		*/
		
		let mut encrypted_buffer = Vec::new();
		crate::editor::utility::encrypt_gta_4(&mut unencrypted_directory, &mut encrypted_buffer);
		let encrypted_directory = &encrypted_buffer;
		buffer_out.write_all(encrypted_directory);
		
		let unencrypted_directory_remainder = &unencrypted_buffer[(20 + encrypted_data_len)..unencrypted_buffer.len()];
		buffer_out.write_all(unencrypted_directory_remainder);
		
		if ((body_start % 2048) != 0) && entry_count > 0
		{
			let pad_data_size = 2048 - (body_start % 2048);
			crate::editor::utility::write_zeros(&mut buffer, pad_data_size as u64);
			seek += pad_data_size;
			buffer_out.write_all(&buffer);
		}
	}
	
	// entry data
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
}