
use tower_lsp_server::ls_types::{Range, Position};

use crate::data_types::FxDashMap;


#[derive(Debug)]
pub struct TextPosition {
    pub text: String,
    _position: Range,

    _offset:  Option<u16>,
    _i_sbyte: Option<i8>,
    _i_byte:  Option<u8>,
    _i_word:  Option<u16>,
}

#[derive(Debug)]
pub struct FileContext {
    _file_content: String,
    pub _macros:   FxDashMap<String, TextPosition>,
    pub _opcodes:  FxDashMap<String, TextPosition>,
    pub _strings:  FxDashMap<String, TextPosition>,
    pub _numbers:  FxDashMap<String, TextPosition>,
    pub comments: FxDashMap<String, TextPosition>,
    pub _symbols:  FxDashMap<String, TextPosition>,
}

impl FileContext {
    pub fn new(file_content: String) -> Self {
        Self {
            _file_content: file_content,
            _macros:   FxDashMap::default(),
            _opcodes:  FxDashMap::default(),
            _strings:  FxDashMap::default(),
            _numbers:  FxDashMap::default(),
            comments: FxDashMap::default(),
            _symbols:  FxDashMap::default(),
        }
    }

    pub fn parse_file(&mut self) {
        // parse stuff

        // Debuging comment
        self.comments.insert("debug_hover".to_string(), TextPosition {
            text: "hello this is a hover default".to_string(),
            _position: Range {
                start: Position {
                    line:      0,
                    character: 0,
                },
                end: Position {
                    line:      0,
                    character: 0,
                },
            },
            _offset:  None,
            _i_sbyte: None,
            _i_byte:  None,
            _i_word:  None,
        });
    }
}
