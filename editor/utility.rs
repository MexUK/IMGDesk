use std::fs;
use std::io::BufReader;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::io::SeekFrom;
use std::io::Write;
use std::ffi::OsStr;
use std::path::Path;
use std::str;
use std::time::SystemTime;

use num_format::{Locale, ToFormattedString};

// file data
pub fn set_file_data(path: String, data: &Vec<u8>) -> bool
{
	let mut f = File::create(&path).expect("no file found");
	f.write_all(data.as_slice());
	return true;
}

pub fn set_file_data_no_overwrite(path: String, data: &Vec<u8>) -> bool
{
	let mut f = File::create(&get_next_file_path(path)).expect("no file found");
	f.write_all(data.as_slice());
	return true;
}

pub fn set_file_data_overlap(path: String, data: &Vec<u8>, uses_temp_out_path: bool, temp_out_path: String) -> bool
{
	if uses_temp_out_path
	{
		let mut f = File::create(&temp_out_path).expect("no temp_out_path file created");
		f.write_all(data.as_slice());
		fs::remove_file(&path);
		fs::rename(temp_out_path, &path);
	}
	else
	{
		let mut f = File::create(&path).expect("no path file created");
		f.write_all(data.as_slice());
	}
	true
}

pub fn get_file_data(path: String) -> Vec<u8>
{
	let mut f = File::open(&path).expect("no file found");
	let metadata = fs::metadata(&path).expect("unable to read file metadata");
	let mut buffer = vec![0; metadata.len() as usize];
	f.read(&mut buffer).expect("buffer overflow reading file");
	return buffer;
}

pub fn get_file_data_with_meta(path: String) -> (Vec<u8>, fs::Metadata)
{
	let mut f = File::open(&path).expect("no file found");
	let metadata = fs::metadata(&path).expect("unable to read file metadata");
	let mut buffer = vec![0; metadata.len() as usize];
	f.read(&mut buffer).expect("buffer overflow reading file");
	(buffer, metadata)
}

pub fn get_file_data_range(path: String, offset: u64, size: u64) -> Vec<u8>
{
	let mut f = File::open(&path).expect("no ranged file found");
	let mut buffer = vec![0; size as usize];
	f.seek(SeekFrom::Start(offset));
	f.read(&mut buffer).expect("buffer overflow reading ranged file");
	return buffer;
}

pub fn get_file_data_range_with_reader(reader: &mut BufReader<File>, offset: u64, size: u64) -> Vec<u8>
{
	let mut buffer = vec![0; size as usize];
	reader.seek(SeekFrom::Start(offset));
	reader.read_exact(&mut buffer).expect("buffer overflow reading ranged file");
	return buffer;
}

// file attributes
pub fn get_file_size(path: String) -> u64
{
	let metadata = fs::metadata(&path);
	match metadata
	{
		Err(e1) =>
		{
			return 0;
		},
		Ok(r1) =>
		{
			return r1.len();
		}
		_ =>
		{
			return 0;
		}
	}
}

pub fn get_file_last_modified(path: String, default: std::time::SystemTime) -> std::time::SystemTime
{
	let metadata = fs::metadata(&path);
	match metadata
	{
		Err(e1) =>
		{
			return default;
		},
		_ =>
		{
		}
	};
	
	let modified = metadata.unwrap().modified();
	match modified
	{
		Err(e2) =>
		{
			return default;
		},
		_ =>
		{
			return modified.unwrap();
		}
	};
}

// file path
pub fn get_next_file_path(path_in: String) -> String
{
	let file_name : String = Path::new(&path_in).file_name().and_then(OsStr::to_str).unwrap().to_string();
	let dir : String = (&path_in[0..(path_in.len()-file_name.len())]).to_string();
	
	let mut counter : u64 = 1;
	
	let ext : String = super::vendor::get_extension_from_filename(&file_name).unwrap().to_string();
	let file_name_no_ext : String = (&file_name[0..file_name.len()-(ext.len()+1)]).to_string();
	
	loop
	{
		let path : String = if counter == 1
		{
			format!("{}{}", dir, file_name)
		}
		else
		{
			format!("{}{} ({}).{}", dir, file_name_no_ext, counter.to_string(), ext)
		};
		
		if !Path::new(&path).exists()
		{
			return path;
		}
		
		counter = counter + 1;
	}
}

pub fn get_next_file_path2(dir: String, file_name: String) -> String
{
	let mut file_path : String = dir.to_string();
	let c : char = file_path.chars().last().unwrap();
	if c != '/' && c != '\\'
	{
		file_path.push_str("/");
	}
	file_path.push_str(file_name.as_str());
	
	get_next_file_path(file_path)
}

pub fn get_file_name(file_name: &str) -> Option<&str>
{
	Path::new(file_name)
		.file_name()
		.and_then(OsStr::to_str)
}

pub fn replace_file_extension(path: &str, ext: &str) -> Option<String>
{
	Some(Path::new(path).with_extension(ext).to_str().unwrap().to_string())
}

// string
pub fn get_percent(partial_item_count: i32, total_item_count: i32) -> String
{
	if total_item_count == 0
	{
		return String::from("0");
	}
	else
	{
		let percent : i32 = ((partial_item_count as f64 / total_item_count as f64) * 100f64).floor() as i32;
		if percent == 0 && partial_item_count > 0
		{
			return String::from("1");
		}
		else
		{
			return percent.to_formatted_string(&Locale::en);
		}
	}
}

pub fn get_null_string(buffer: &mut Vec<u8>, mut seek: u64) -> String
{
	let start = seek as usize;
	for i in seek..(buffer.len() as u64)
	{
		let value = buffer[seek as usize];
		if value == 0
		{
			return str::from_utf8(&buffer[start..(seek as usize)]).unwrap().to_string();
		}
		seek += 1;
	}
	str::from_utf8(&buffer[start..buffer.len()]).unwrap().to_string()
}

// gta
pub fn encrypt_gta_4(mut buffer_in: &mut Vec<u8>, mut buffer_out: &mut Vec<u8>)
{
	let key = [ 0x1a, 0xb5, 0x6f, 0xed, 0x7e, 0xc3, 0xff, 0x1, 0x22, 0x7b, 0x69, 0x15, 0x33, 0x97, 0x5d, 0xce, 0x47, 0xd7, 0x69, 0x65, 0x3f, 0xf7, 0x75, 0x42, 0x6a, 0x96, 0xcd, 0x6d, 0x53, 0x7, 0x56, 0x5d ];
	encrypt_aes_128_all(&key.to_vec(), buffer_in, buffer_out, 16);
}

pub fn decrypt_gta_4(mut buffer_in: &mut Vec<u8>, mut buffer_out: &mut Vec<u8>)
{
	let key = [ 0x1a, 0xb5, 0x6f, 0xed, 0x7e, 0xc3, 0xff, 0x1, 0x22, 0x7b, 0x69, 0x15, 0x33, 0x97, 0x5d, 0xce, 0x47, 0xd7, 0x69, 0x65, 0x3f, 0xf7, 0x75, 0x42, 0x6a, 0x96, 0xcd, 0x6d, 0x53, 0x7, 0x56, 0x5d ];
	decrypt_aes_128_all(&key.to_vec(), buffer_in, buffer_out, 16);
}

// aes - encrypt
fn encrypt_aes_128_all(key: &Vec<u8>, mut buffer_in2: &mut Vec<u8>, mut buffer_out2: &mut Vec<u8>, round_count: u64)
{
	let block_size = 4096;
	
	let mut buffer_out3 = Vec::new();
	buffer_out3.resize(buffer_in2.len(), 0);
	
	let to = ((buffer_in2.len() as f64) / (block_size as f64)).ceil() as usize;
	for i in 0..to
	{
		let start = i*block_size;
		let end = if i == (to-1) { start + (buffer_in2.len() % block_size) } else { (i+1)*block_size };
		let mut buffer_in3 = (&buffer_in2[start..end].to_vec()).clone();
		
		encrypt_aes_128(&key, &mut buffer_in3, &mut buffer_out3, round_count);
		buffer_out2.extend(buffer_out3.clone());
	}
}

fn encrypt_aes_128(key: &Vec<u8>, mut buffer_in2: &mut Vec<u8>, mut buffer_out2: &mut Vec<u8>, round_count: u64)
{
	let mut buffer_in3 : Vec<u8> = buffer_in2.clone();
	let mut buffer_out3 = Vec::new();
	buffer_out3.resize(buffer_in2.len(), 0);
	
	for i in 0..(round_count as usize)
	{
		encrypt_aes_128_once(&key, &mut buffer_in3, &mut buffer_out3);
		buffer_in3 = buffer_out3.clone();
	}
	buffer_out2.clear();
	buffer_out2.extend(buffer_out3.clone());
}

fn encrypt_aes_128_once(key: &Vec<u8>, mut buffer_in2: &mut Vec<u8>, mut buffer_out2: &mut Vec<u8>)
{
	let mut encryptor = crypto::aes::ecb_encryptor(crypto::aes::KeySize::KeySize256, &key, crypto::blockmodes::NoPadding);
	
	let mut buffer_in = crypto::buffer::RefReadBuffer::new(&mut buffer_in2);
	let mut buffer_out = crypto::buffer::RefWriteBuffer::new(&mut buffer_out2);
	encryptor.encrypt(&mut buffer_in, &mut buffer_out, true);
}

// aes - decrypt
fn decrypt_aes_128_all(key: &Vec<u8>, mut buffer_in2: &mut Vec<u8>, mut buffer_out2: &mut Vec<u8>, round_count: u64)
{
	let block_size = 4096;
	
	let mut buffer_out3 = Vec::new();
	buffer_out3.resize(buffer_in2.len(), 0);
	
	let to = ((buffer_in2.len() as f64) / (block_size as f64)).ceil() as usize;
	for i in 0..to
	{
		let start = i*block_size;
		let end = if i == (to-1) { start + (buffer_in2.len() % block_size) } else { (i+1)*block_size };
		let mut buffer_in3 = (&buffer_in2[start..end].to_vec()).clone();
		
		decrypt_aes_128(&key, &mut buffer_in3, &mut buffer_out3, round_count);
		buffer_out2.extend(buffer_out3.clone());
	}
}

fn decrypt_aes_128(key: &Vec<u8>, mut buffer_in2: &mut Vec<u8>, mut buffer_out2: &mut Vec<u8>, round_count: u64)
{
	let mut buffer_in3 : Vec<u8> = buffer_in2.clone();
	let mut buffer_out3 = Vec::new();
	buffer_out3.resize(buffer_in2.len(), 0);
	
	for i in 0..(round_count as usize)
	{
		decrypt_aes_128_once(&key, &mut buffer_in3, &mut buffer_out3);
		buffer_in3 = buffer_out3.clone();
	}
	buffer_out2.clear();
	buffer_out2.extend(buffer_out3.clone());
}

fn decrypt_aes_128_once(key: &Vec<u8>, mut buffer_in2: &mut Vec<u8>, mut buffer_out2: &mut Vec<u8>)
{
	let mut decryptor = crypto::aes::ecb_decryptor(crypto::aes::KeySize::KeySize256, &key, crypto::blockmodes::NoPadding);
	
	let mut buffer_in = crypto::buffer::RefReadBuffer::new(&mut buffer_in2);
	let mut buffer_out = crypto::buffer::RefWriteBuffer::new(&mut buffer_out2);
	decryptor.decrypt(&mut buffer_in, &mut buffer_out, true);
}

// buffer
pub fn write_zeros(buffer: &mut Vec<u8>, zero_count: u64)
{
	for _i2 in 0..(zero_count as usize)
	{
		buffer.push(0);
	}
}

// sectors
pub fn to_sectors(size: u64) -> u64
{
	return (size as f64 / 2048f64).ceil() as u64;
}

pub fn sectors_to_bytes(bytes: u64) -> u64
{
	return bytes * 2048;
}

pub fn to_sector_bytes(bytes: u64) -> u64
{
	return sectors_to_bytes(to_sectors(bytes));
}

