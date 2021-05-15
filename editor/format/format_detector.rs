use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

use crypto::aes;
use crypto::buffer::WriteBuffer;

// detect
pub fn detect_version(img_path_in: &String) -> (super::FormatType, u8, bool)
{
	let mut file1 = File::open(&img_path_in).expect("no IMG file found");
	
	let mut buffer1 = [0; 4];
	file1.read(&mut buffer1);
	
	// version 2
	let string1 = unsafe { super::super::vendor::str_from_u8_nul_utf8_unchecked(&buffer1) };
	if string1 == "VER2"
	{
		return (super::FormatType::IMG, 2, false);
	}
	
	// version 3 unencrypted
	if buffer1 == u32::to_le_bytes(0xA94E2A52)
	{
		return (super::FormatType::IMG, 3, false);
	}
	
	// RPF
	match unsafe { std::mem::transmute::<[u8; 4], u32>(buffer1) }.to_be()
	{
		0x52504630 => { return (super::FormatType::RPF, 0, false); }
		0x52504632 => { return (super::FormatType::RPF, 2, false); }
		0x52504633 => { return (super::FormatType::RPF, 3, false); }
		0x52504634 => { return (super::FormatType::RPF, 4, false); }
		0x52504636 => { return (super::FormatType::RPF, 6, false); }
		0x52504637 => { return (super::FormatType::RPF, 7, false); }
		0x52504638 => { return (super::FormatType::RPF, 8, false); }
		_ => {}
	}
	
	// version 1
	let metadata = fs::metadata(super::super::utility::replace_file_extension(img_path_in.as_str(), "dir").unwrap());
	match metadata
	{
		Err(e) => {},
		_ =>
		{
			if metadata.unwrap().is_file()
			{
				return (super::FormatType::IMG, 1, false);
			}
		}
	}
	
	// version 3 encrypted
	let mut buffer2 = Vec::with_capacity(20);
	buffer2.resize(20, 0);
	file1.seek(SeekFrom::Start(0));
	file1.read(&mut buffer2);
	
	let mut buffer_20B_decrypted = Vec::new();
	super::super::utility::decrypt_gta_4(&mut buffer2, &mut buffer_20B_decrypted);
	if buffer_20B_decrypted.len() >= 4 && &buffer_20B_decrypted[0..4] == u32::to_le_bytes(0xA94E2A52)
	{
		return (super::FormatType::IMG, 3, true);
	}
	
	// unknown version
	(super::FormatType::UNKNOWN, 0, false)
}

