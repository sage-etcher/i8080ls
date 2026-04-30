
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
pub struct CodeNumber{
    pub element: CodeElement,   // common CodeElement details
    pub value: u16,             // value of the number
}

#[derive(Debug)]
pub struct CodeOpcode {
    pub element: CodeElement,   // common CodeElement details
}

#[derive(Debug)]
pub struct CodeSymbol {
    pub element: CodeElement,   // common CodeElement details
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

// end of file
