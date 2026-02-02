use crossterm::event::KeyCode;
use fluxion_core::Action;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeySequence {
    pub keys: Vec<KeyCode>,
}

impl KeySequence {
    pub fn new(keys: Vec<KeyCode>) -> Self {
        Self { keys }
    }
}

pub struct Keybindings {
    pub normal: HashMap<KeySequence, Action>,
    pub insert: HashMap<KeySequence, Action>,
    pub visual: HashMap<KeySequence, Action>,
    pub command: HashMap<KeySequence, Action>,
    pub save_dialog: HashMap<KeySequence, Action>,
    pub file_picker: HashMap<KeySequence, Action>,
}

impl Keybindings {
    pub fn default_vim() -> Self {
        let mut normal = HashMap::new();

        // Single key mappings
        normal.insert(
            KeySequence::new(vec![KeyCode::Char(':')]),
            Action::EnterCommandMode,
        );
        normal.insert(KeySequence::new(vec![KeyCode::Char('h')]), Action::MoveLeft);
        normal.insert(KeySequence::new(vec![KeyCode::Char('j')]), Action::MoveDown);
        normal.insert(KeySequence::new(vec![KeyCode::Char('k')]), Action::MoveUp);
        normal.insert(
            KeySequence::new(vec![KeyCode::Char('l')]),
            Action::MoveRight,
        );
        normal.insert(
            KeySequence::new(vec![KeyCode::Char('i')]),
            Action::EnterInsertMode,
        );
        normal.insert(
            KeySequence::new(vec![KeyCode::Char('v')]),
            Action::EnterVisualMode,
        );

        // Buffer prefix sequences
        normal.insert(
            KeySequence::new(vec![KeyCode::Char('b'), KeyCode::Char('n')]),
            Action::NextBuffer,
        );
        normal.insert(
            KeySequence::new(vec![KeyCode::Char('b'), KeyCode::Char('p')]),
            Action::PrevBuffer,
        );
        normal.insert(
            KeySequence::new(vec![KeyCode::Char('b'), KeyCode::Char('x')]),
            Action::CloseBuffer,
        );
        normal.insert(
            KeySequence::new(vec![KeyCode::Char('b'), KeyCode::Char('a')]),
            Action::CloseAllBuffersExcept,
        );

        // Single key buffer switches
        normal.insert(
            KeySequence::new(vec![KeyCode::Char('[')]),
            Action::PrevBuffer,
        );
        normal.insert(
            KeySequence::new(vec![KeyCode::Char(']')]),
            Action::NextBuffer,
        );

        for i in 0..=9 {
            let c = char::from_digit(i, 10).unwrap();
            normal.insert(
                KeySequence::new(vec![KeyCode::Char(c)]),
                Action::SwitchBuffer(i as usize),
            );
        }

        // Leader key sequences (Space = leader)
        normal.insert(
            KeySequence::new(vec![KeyCode::Char(' '), KeyCode::Char('f')]),
            Action::EnterFilePicker,
        );

        let mut insert = HashMap::new();
        insert.insert(
            KeySequence::new(vec![KeyCode::Esc]),
            Action::EnterNormalMode,
        );
        insert.insert(KeySequence::new(vec![KeyCode::Enter]), Action::Insert('\n'));
        insert.insert(KeySequence::new(vec![KeyCode::Backspace]), Action::Delete);

        let mut visual = HashMap::new();
        visual.insert(
            KeySequence::new(vec![KeyCode::Esc]),
            Action::EnterNormalMode,
        );
        visual.insert(KeySequence::new(vec![KeyCode::Char('h')]), Action::MoveLeft);
        visual.insert(KeySequence::new(vec![KeyCode::Char('j')]), Action::MoveDown);
        visual.insert(KeySequence::new(vec![KeyCode::Char('k')]), Action::MoveUp);
        visual.insert(
            KeySequence::new(vec![KeyCode::Char('l')]),
            Action::MoveRight,
        );

        let mut command = HashMap::new();
        command.insert(
            KeySequence::new(vec![KeyCode::Esc]),
            Action::EnterNormalMode,
        );
        command.insert(
            KeySequence::new(vec![KeyCode::Enter]),
            Action::ExecuteCommand,
        );
        command.insert(
            KeySequence::new(vec![KeyCode::Backspace]),
            Action::DeleteFromCommand,
        );

        let mut save_dialog = HashMap::new();
        save_dialog.insert(KeySequence::new(vec![KeyCode::Esc]), Action::CancelDialog);
        save_dialog.insert(
            KeySequence::new(vec![KeyCode::Enter]),
            Action::SaveBufferAs(None),
        );
        save_dialog.insert(
            KeySequence::new(vec![KeyCode::Backspace]),
            Action::DeleteFromCommand,
        );

        let mut file_picker = HashMap::new();
        file_picker.insert(KeySequence::new(vec![KeyCode::Esc]), Action::FilePickerEsc);
        file_picker.insert(
            KeySequence::new(vec![KeyCode::Enter]),
            Action::FilePickerEnter,
        );
        file_picker.insert(
            KeySequence::new(vec![KeyCode::Char('j')]),
            Action::FilePickerDown,
        );
        file_picker.insert(
            KeySequence::new(vec![KeyCode::Char('k')]),
            Action::FilePickerUp,
        );

        Self {
            normal,
            insert,
            visual,
            command,
            save_dialog,
            file_picker,
        }
    }
}
