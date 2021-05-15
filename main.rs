//#![windows_subsystem = "windows"]

extern crate native_windows_gui as nwg;

mod editor;

fn main()
{
	nwg::init().expect("Failed to init Native Windows GUI");
	
	editor::load();
	
    nwg::dispatch_thread_events();
}

