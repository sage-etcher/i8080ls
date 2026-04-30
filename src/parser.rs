
use crate::data_types::FxDashMap;

#[derive(Debug)]
pub struct CodeElementPosition {
    start: u32,
    end: u32,
}

#[derive(Debug)]
pub struct CodeElement {
    pub text: String,                   // source text
    pub position: CodeElementPosition,  // position of text inside file_contents
    pub description: Option<String>,    // comment description
}

#[derive(Debug)]
pub struct CodeMacro {
    pub element: CodeElement,   // common CodeElement details
    pub macro_text: String,     // macro replace values
}

#[derive(Debug)]
pub struct CodeOpcode {
    pub element: CodeElement,   // common CodeElement details
}

pub type CodeSymbol = CodeOpcode;


#[derive(Debug)]
pub struct FileContext {
    file_content: String,
    pub macros:   FxDashMap<String, CodeMacro>,
    pub opcodes:  FxDashMap<String, CodeOpcode>,
}

impl CodeElementPosition {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

impl CodeElement {
    pub fn new(text: &str, position: CodeElementPosition) -> Self {
        Self {
            text: String::from(text),
            position,
            description: None,
        }
    }
}

impl FileContext {
    pub fn new(file_content: &str) -> Self {
        Self {
            file_content: String::from(file_content),
            macros:   FxDashMap::default(),
            opcodes:  FxDashMap::default(),
        }
    }

    fn add_debug_macros(&mut self) {
        // Hover Macro
        let debug_hover_key = "debug_hover";
        let debug_hover_macro = CodeMacro {
            element: CodeElement::new(debug_hover_key, CodeElementPosition::new(0, 0)),
            macro_text: String::from("hello this is a hover default"),
        };

        self.macros.insert(String::from(debug_hover_key), debug_hover_macro);
    }

    pub fn parse_file(&mut self) {
        // parse stuff

        self.add_debug_macros();
    }
}
