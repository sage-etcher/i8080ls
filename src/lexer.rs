
use tower_lsp_server::ls_types::{Position, Range};

use crate::data_types::FxDashMap;
use crate::symbol::Symbol;


#[derive(Debug)]
pub struct Lexer {
    pub file_content: String,
    index: usize,
    pub ch: Option<char>,
    pub ch_lower: Option<char>,

    line: u32,
    line_len: u32,
    character: u32,
    pub position: Range,

    kw_syms: FxDashMap<String, Vec<Symbol>>,
    pub number: u32,
    pub ident: String,
}

impl Lexer {
    pub fn new(file_content: &str) -> Self {

        Self {
            file_content: String::from(file_content),
            index:0,
            ch: None,
            ch_lower: None,

            line:      0,
            line_len:  0,
            character: 0,
            position: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 0,
                },
            },

            kw_syms: Symbol::get_keywords(),
            number: 0,
            ident:  String::new(),
        }
    }

    pub fn from(src: &Lexer) -> Lexer {
        return Lexer {
            file_content: String::from(src.file_content.clone()),
            line_len:     src.line_len,
            index:        src.index,
            ch:           src.ch,
            ch_lower:     src.ch_lower,

            line:         src.line,
            character:    src.character,
            position:     src.position,
            kw_syms:      Symbol::get_keywords(),
            number:       src.number,
            ident:        String::from(src.ident.clone()),
        };
    }

    pub fn reset(&mut self) {
        self.index     = 0;
        self.line      = 0;
        self.character = 0;
    }

    pub fn read_ch(&mut self) -> Option<char> {
        if self.index >= self.file_content.len()
        {
            self.ch = None;
            self.ch_lower = None;
            return None;
        }

        let ch = self.file_content.chars().nth(self.index).unwrap();
        self.index += 1;

        if ch == '\n' {
            self.line_len = self.character;
            self.line += 1;
            self.character = 0;
        } else {
            self.character += 1;
        }

        // return
        self.ch = Some(ch);
        self.ch_lower = ch.to_lowercase().to_string().chars().nth(0);
        return self.ch;
    }

    fn set_pos_start(&mut self) {
        self.position.start.line = self.line;
        self.position.start.character = self.character;

        if self.position.start.character > 0 {
            self.position.start.character -= 1;
        }
    }

    fn set_pos_end(&mut self) {
        self.position.end.line = self.line;
        self.position.end.character = self.character;

        if self.position.end.character > 0 {
            self.position.end.character -= 1;
        } else {
            if self.position.end.line > 0 {
                self.position.end.line -= 1;
                self.position.end.character = self.line_len;
            }
        }
    }

    fn parse_number(&mut self) -> Vec<Symbol> {
        /* fill a list with characters */
        let mut xdigit_arr: Vec<char> = Vec::new();

        xdigit_arr.push(self.ch_lower.unwrap());
        self.set_pos_start();
        loop {
            match self.read_ch() {
                None      => break,
                Some('$') => continue,
                _         => {
                    if !self.ch_lower.unwrap().is_digit(16) {
                        break;
                    }

                    xdigit_arr.push(self.ch_lower.unwrap());
                }
            }
        }

        /* detect base */
        let base;

        match self.ch_lower { /* non xdigit suffix */
            None      => { base = 10; },
            Some('h') => { self.read_ch(); base = 16; },
            Some('o') => { self.read_ch(); base =  8; },
            Some('q') => { self.read_ch(); base =  8; },
            _ => {
                match xdigit_arr.last() { /* xdigit suffix */
                    Some('b') => { xdigit_arr.pop(); base =  2; },
                    Some('d') => { xdigit_arr.pop(); base = 10; },
                    _ => {
                        if !self.ch.unwrap().is_alphanumeric() {
                            base = 10; /* implicit decimal */
                        } else { /* error bad suffix */
                            return vec![Symbol::Unknown];
                        }
                    }
                }
            }
        }
        self.set_pos_end();

        /* convert iterable to number */
        let mut number: u32 = 0;
        for xdigit in xdigit_arr.into_iter() {
            let digit = xdigit.to_digit(base);
            if digit.is_none() {
                return vec![Symbol::Unknown];
            }

            number *= base;
            number += digit.unwrap();
        }


        self.number = number;
        if number < 256 {
            return vec![Symbol::NumberByte, Symbol::NumberWord];
        } else if number < 65536 {
            return vec![Symbol::NumberWord]
        } else {
            return vec![Symbol::NumberOverflow]
        }
    }

    fn parse_alpha(&mut self) -> Vec<Symbol> {

        /* collect ident */
        let mut ident_arr: Vec<char> = Vec::new();

        ident_arr.push(self.ch_lower.unwrap());
        self.set_pos_start();
        loop {
            match self.read_ch() {
                None      => break,
                Some('$') => continue,
                Some('_') => continue,
                _ => {
                    if !self.ch_lower.unwrap().is_alphanumeric() {
                        break;
                    }
                }
            }

            ident_arr.push(self.ch_lower.unwrap());
        }
        let ident: String = ident_arr.into_iter().collect();


        /* check to be equ or set */
        if ident == "equ" || ident == "set" {
            self.parse_skip_whitespace();
            self.parse_to_end_of_line();

            if ident == "equ" {
                return vec![Symbol::MacroEQU];
            } else {
                return vec![Symbol::MacroSET];
            }
        } else {
            self.set_pos_end();
        }

        /* check for keywords */
        let kw_sym = self.kw_syms.get(&ident);
        if !kw_sym.is_none() {
            return kw_sym.unwrap().value().to_vec();
        }

        /* assume to be ident */
        self.ident = ident;

        return vec![Symbol::Ident];
    }

    fn parse_to_end_of_line(&mut self) {
        let mut ident_arr: Vec<char> = Vec::new();

        while !self.ch.is_none() && self.ch.unwrap() != '\n' {
            ident_arr.push(self.ch.unwrap());
            self.read_ch();
        }
        self.set_pos_end();

        self.ident = ident_arr.into_iter().collect();
        return;
    }

    fn parse_skip_whitespace(&mut self) {
        while !self.ch.is_none() && 
            (
                self.ch.unwrap() == ' ' || 
                self.ch.unwrap() == '\t'
            ) 
        {
            self.read_ch();
        }
    }

    pub fn parse_comment(&mut self) -> Vec<Symbol> {
        self.set_pos_start();
        self.read_ch();
        self.parse_to_end_of_line();
        return vec![Symbol::Comment];
    }

    fn parse_string(&mut self) -> Vec<Symbol> {
        let mut ident_arr: Vec<char> = Vec::new();

        self.set_pos_start();
        while !self.read_ch().is_none()  &&
            self.ch.unwrap().is_ascii()  &&
            !self.ch.unwrap().is_ascii_control() &&
            self.ch.unwrap() != '!'  &&
            self.ch.unwrap() != '\'' 
        {
            ident_arr.push(self.ch.unwrap());
        }
        let ch: Option<char> = self.ch;
        self.read_ch();
        self.set_pos_end();
        if ch.is_none() || ch.unwrap() != '\'' {
            return vec![Symbol::Unknown];
        }

        let ident: String = ident_arr.into_iter().collect();
        let ident_bytes: &[u8] = ident.as_bytes();

        self.ident = ident.clone();
        match ident_bytes.len() {
            1 => {
                self.number = ident_bytes[0] as u32;
                return vec![Symbol::StringASCII, Symbol::NumberByte, Symbol::NumberWord];
            },
            2 => {
                self.number = ident_bytes[0] as u32;
                self.number *= 0x10;
                self.number += ident_bytes[1] as u32;
                return vec![Symbol::StringASCII, Symbol::NumberWord];
            },
            _ => {
                return vec![Symbol::StringASCII];
            }

        }


    }

    fn parse_symbol(&mut self, retval: Vec<Symbol>) -> Vec<Symbol> {
        self.set_pos_start();
        self.read_ch();
        self.set_pos_end();
        return retval;
    }

    pub fn get_symbol(&mut self) -> Vec<Symbol> {
        self.parse_skip_whitespace();

        // symbols
        match self.ch {
            None       => return self.parse_symbol(vec![Symbol::EOF]),
            Some('\n') => return self.parse_symbol(vec![Symbol::Newline]),
            Some('!')  => return self.parse_symbol(vec![Symbol::Newline]),
            Some(',')  => return self.parse_symbol(vec![Symbol::Comma]),
            Some(':')  => return self.parse_symbol(vec![Symbol::Colon]),
            Some('$')  => return self.parse_symbol(vec![Symbol::MacroPC]),
            Some('+')  => return self.parse_symbol(vec![Symbol::MacroAdd]),
            Some('-')  => return self.parse_symbol(vec![Symbol::MacroSub]),
            Some('*')  => return self.parse_symbol(vec![Symbol::MacroMult]),
            Some('/')  => return self.parse_symbol(vec![Symbol::MacroDiv]),
            Some('%')  => return self.parse_symbol(vec![Symbol::MacroMod]),
            Some(';')  => return self.parse_comment(),
            Some('\'') => return self.parse_string(),
            _ => {
                // number
                if self.ch.unwrap().is_digit(10) {
                    return self.parse_number();
                }

                // ident, register, or opcode
                if self.ch.unwrap().is_alphabetic() {
                    return self.parse_alpha();
                }

                self.read_ch();
                return self.parse_symbol(vec![Symbol::Unknown]);
            }
        }
    }
}

// end of file
