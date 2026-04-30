
use crate::data_types::FxDashMap;
use crate::code_elements::*;

enum Symbol {
    OPCODE,
    REG_A,
    REG_B,
    REG_C,
    REG_D,
    REG_E,
    REG_F,
    REG_H,
    REG_L,
    REG_M,
    REGPAIR_SP,
    REGPAIR_PSW,
    REGPAIR_BC = Symbol::REG_B,
    REGPAIR_DE = Symbol::REG_D,
    REGPAIR_HL = Symbol::REG_H,
    NUMBER,
    COMMA,
    COLON,
    IDENT,
    NEWLINE,
    COMMENT,
    EOF,
};

#[derive(Debug)]
struct Lexer {
    file_content: String,
    index: u32,
    ch: Option<char>,

    line:   u32,
    column: u32,
};

#[derive(Debug)]
struct Parser {
    lexer: Lexer,
}

#[derive(Debug)]
pub struct FileContext {
    parser: Parser,

    pub macros:   FxDashMap<String, CodeMacro>,
    pub opcodes:  FxDashMap<String, CodeOpcode>,
    pub numbers:  FxDashMap<String, CodeNumber>,
    pub symbols:  FxDashMap<String, CodeSymbol>,
}

impl Lexer {
    pub fn new(file_content: &str) -> Self {
        Self {
            file_content: String::from(file_content),
            index:  0,
            line:   0,
            column: 0,
        }
    }

    fn read_ch(&mut self) -> Option<char> {
        if (index >= self.file_content.len())
        {
            self.ch = None;
            return None;
        }

        let ch = self.file_content[self.index++];
        if (ch == '\n') {
            self.line++;
            self.column = 0;
        }

        // return
        self.ch = Some(ch);
        return self.ch;
    }

    fn get_symbol(&mut self) -> Vec<Symbol> {
        while (self.read_ch().unwrap().is_ascii_whitespace()) {}

        match self.ch {
            None       => { self.read_ch(); return Symbol::EOF; }
            Some('\n') => { self.read_ch(); return Symbol::NEWLINE; }
            Some(',')  => { self.read_ch(); return Symbol::COMMA; }
            Some(':')  => { self.read_ch(); return Symbol::COLON; }
        }

        if (self.ch.is_ascii_number()) {
            // number
        }

        if (self.ch.is_alphanumeric()) {
            // ident, register, or opcode
        }




    }
}

impl Parser {
    pub fn new(file_content: &str) -> Self {
        Self {
            lexer: Lexer::new(file_content),
        }
    }
}

impl FileContext {
    pub fn new(file_content: &str) -> Self {
        Self {
            parser: Parser::new(file_content),

            macros:   FxDashMap::default(),
            opcodes:  FxDashMap::default(),
            numbers:  FxDashMap::default(),
            symbols:  FxDashMap::default(),
        }
    }

    fn add_debug_macros(&mut self) {
        // Hover Macro
        let debug_hover_key = String::from("debug_hover");
        let debug_hover_macro = CodeMacro {
            element: CodeElement::new(&debug_hover_key, CodeElementPosition::new(0, 0)),
            macro_text: String::from("hello this is a hover default"),
        };

        self.macros.insert(debug_hover_key, debug_hover_macro);
    }

    pub fn parse_file(&mut self) {
        // parse code


        // parse stuff

        self.add_debug_macros();
    }
}

// end of file
