

#[derive(Copy, Clone)]
pub enum ActionTypeId
{
	UnknownAction,
	AddEntries,
	RemoveEntries,
	ReplaceEntries,
	RenameEntry,
	MoveEntry,
	SetImgVersion,
	RecalculateOffsets
}

pub struct ActionHistory
{
	pub actions: Vec<ActionItem>,
	pub index: i64
}

impl Default for ActionHistory
{
	fn default() -> Self
	{
		Self
		{
			actions: Vec::new(),
			index: -1
		}
	}
}






pub struct ActionItem
{
	pub _type: ActionTypeId,
	
	pub add: ActionAdd,
	pub remove: ActionRemove,
	pub replace: ActionReplace,
	pub rename: ActionRename,
	pub _move: ActionMove,
	pub set_img_version: ActionSetImgVersion,
	pub recalculate_offsets: ActionRecalculateOffsets
}

impl Default for ActionItem
{
	fn default() -> Self
	{
		Self
		{
			_type: ActionTypeId::UnknownAction,
			add: ActionAdd { entries: Vec::new(), entry_offsets_were_recalculated: false, entry_offsets: Vec::new() },
			remove: ActionRemove { entries: Vec::new(), entry_offsets_were_recalculated: false, entry_offsets: Vec::new() },
			replace: ActionReplace { entries: Vec::new(), entry_offsets_were_recalculated: false, entry_offsets: Vec::new() },
			rename: ActionRename { entry_index: 0, old_name: String::from(""), new_name: String::from("") },
			_move: ActionMove { old_entry_index: 0, new_entry_index: 0 },
			set_img_version: ActionSetImgVersion { old_img_version: 0, old_img_encrypted: false, new_img_version: 0, new_img_encrypted: false },
			recalculate_offsets: ActionRecalculateOffsets { entry_offsets: Vec::new() }
		}
	}
}

impl ActionItem
{
	pub fn undo(&mut self)
	{
		match self._type
		{
			ActionTypeId::AddEntries =>
			{
				super::get_editor().undo_add_entries(&mut self.add);
			},
			ActionTypeId::RemoveEntries =>
			{
				super::get_editor().undo_remove_entries(&mut self.remove);
			},
			ActionTypeId::ReplaceEntries =>
			{
				super::get_editor().undo_replace_entries(&mut self.replace);
			},
			ActionTypeId::RenameEntry =>
			{
				super::get_editor().undo_rename_entry(&mut self.rename);
			},
			ActionTypeId::MoveEntry =>
			{
				super::get_editor().undo_move_entry(&mut self._move);
			},
			ActionTypeId::SetImgVersion =>
			{
				super::get_editor().undo_set_img_version(&mut self.set_img_version);
			},
			ActionTypeId::RecalculateOffsets =>
			{
				super::get_editor().undo_recalculate_offsets(&mut self.recalculate_offsets);
			},
			_ => {}
		}
		
		crate::editor::get_editor().update_all_stats();
	}
	
	pub fn redo(&mut self)
	{
		match self._type
		{
			ActionTypeId::AddEntries =>
			{
				super::get_editor().redo_add_entries(&mut self.add);
			},
			ActionTypeId::RemoveEntries =>
			{
				super::get_editor().redo_remove_entries(&mut self.remove);
			},
			ActionTypeId::ReplaceEntries =>
			{
				super::get_editor().redo_replace_entries(&mut self.replace);
			},
			ActionTypeId::RenameEntry =>
			{
				super::get_editor().redo_rename_entry(&mut self.rename);
			},
			ActionTypeId::MoveEntry =>
			{
				super::get_editor().redo_move_entry(&mut self._move);
			},
			ActionTypeId::SetImgVersion =>
			{
				super::get_editor().redo_set_img_version(&mut self.set_img_version);
			},
			ActionTypeId::RecalculateOffsets =>
			{
				super::get_editor().redo_recalculate_offsets(&mut self.recalculate_offsets);
			},
			_ => {}
		}
		
		crate::editor::get_editor().update_all_stats();
	}
	
	/*
	pub fn remove(&mut self)
	{
		match self._type
		{
			ActionTypeId::AddEntries =>
			{
				super::get_editor().remove_add_entries(&self.add);
			},
			ActionTypeId::RemoveEntries =>
			{
				super::get_editor().remove_remove_entries(&self.remove);
			},
			ActionTypeId::ReplaceEntries =>
			{
				println!("{}", "REMOVE REPLACE ENTRIES");
			},
			ActionTypeId::RenameEntry =>
			{
				println!("{}", "REMOVE RENAME ENTRY");
			},
			ActionTypeId::MoveEntry =>
			{
				println!("{}", "REMOVE MOVE ENTRIES");
			},
			_ => {}
		}
	}
	*/
}

// actions
#[derive(Default,Clone)]
pub struct ActionAdd
{
	pub entries: Vec<ActionAddEntry>,
	pub entry_offsets_were_recalculated: bool,
	pub entry_offsets: Vec<u64>
}

#[derive(Default,Clone)]
pub struct ActionRemove
{
	pub entries: Vec<ActionRemoveEntry>,
	pub entry_offsets_were_recalculated: bool,
	pub entry_offsets: Vec<u64>
}

#[derive(Default,Clone)]
pub struct ActionReplace
{
	pub entries: Vec<ActionReplaceEntry>,
	pub entry_offsets_were_recalculated: bool,
	pub entry_offsets: Vec<u64>
}

#[derive(Default,Clone)]
pub struct ActionRename
{
	pub entry_index: u64,
	pub old_name: String,
	pub new_name: String
}

#[derive(Default,Clone)]
pub struct ActionMove
{
	pub old_entry_index: u64,
	pub new_entry_index: u64
}

#[derive(Default,Clone)]
pub struct ActionSetImgVersion
{
	pub old_img_version: u8,
	pub old_img_encrypted: bool,
	pub new_img_version: u8,
	pub new_img_encrypted: bool
}

#[derive(Default,Clone)]
pub struct ActionRecalculateOffsets
{
	pub entry_offsets: Vec<u64>
}

// action entries
#[derive(Clone)]
pub struct ActionAddEntry
{
	pub entry_index: u64,
	pub entry_name: String,
	pub data_undo_path: String
}

#[derive(Clone)]
pub struct ActionRemoveEntry
{
	pub entry_index: u64,
	pub entry_name: String,
	pub data_undo_path: String
}

#[derive(Clone)]
pub struct ActionReplaceEntry
{
	pub entry_index: u64,
	pub entry_name: String,
	pub data_undo_path: String
}
