
use tower_lsp_server::ls_types::{Position, Range};

use crate::data_types::FxDashMap;
use crate::symbol::Symbol;


#[derive(Debug)]
pub struct Lexer {
    file_content: String,
    index: usize,
    ch: Option<char>,
    ch_lower: Option<char>,

    pub position: Range,

    kw_syms: FxDashMap<String, Vec<Symbol>>,
    number: u32,
    ident: String,
}

impl Lexer {
    pub fn new(file_content: &str) -> Self {
        Self {
            file_content: String::from(file_content),
            index:0,
            ch: None,
            ch_lower: None,

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
            self.position.end.line += 1;
            self.position.end.character = 0;
        }

        // return
        self.ch = Some(ch);
        self.ch_lower = ch.to_lowercase().to_string().chars().nth(0);
        return self.ch;
    }

    fn parse_number(&mut self) -> Vec<Symbol> {
        /* fill a list with characters */
        let mut xdigit_arr: Vec<char> = Vec::new();

        xdigit_arr.push(self.ch_lower.unwrap());
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

        /* check for keywords */
        let kw_sym = self.kw_syms.get(&ident);
        if !kw_sym.is_none() {
            return kw_sym.unwrap().value().to_vec();
        }

        /* assume to be ident */
        self.ident = ident;

        return vec![Symbol::Ident];
    }

    pub fn parse_comment(&mut self) -> Vec<Symbol> {
        let mut ident_arr: Vec<char> = Vec::new();

        while !self.read_ch().is_none() && self.ch.unwrap() != '\n' {
            ident_arr.push(self.ch.unwrap());
        }

        self.ident = ident_arr.into_iter().collect();
        return vec![Symbol::Comment];
    }

    pub fn get_symbol(&mut self) -> Vec<Symbol> {
        while !self.ch.is_none() && 
                (self.ch.unwrap() == ' ' || self.ch.unwrap() == '\t') {
            self.read_ch();
        }

        self.position.start.line      = self.position.end.line;
        self.position.start.character = self.position.end.character;

        // symbols
        match self.ch {
            None       => { self.read_ch(); return vec![Symbol::EOF]; }
            Some('\n') => { self.read_ch(); return vec![Symbol::Newline]; }
            Some('!')  => { self.read_ch(); return vec![Symbol::Newline]; }
            Some(',')  => { self.read_ch(); return vec![Symbol::Comma]; }
            Some(':')  => { self.read_ch(); return vec![Symbol::Colon]; }
            Some('$')  => { self.read_ch(); return vec![Symbol::MacroPC]; }
            Some('+')  => { self.read_ch(); return vec![Symbol::MacroAdd]; }
            Some('-')  => { self.read_ch(); return vec![Symbol::MacroSub]; }
            Some('*')  => { self.read_ch(); return vec![Symbol::MacroMult]; }
            Some('/')  => { self.read_ch(); return vec![Symbol::MacroDiv]; }
            Some('%')  => { self.read_ch(); return vec![Symbol::MacroMod]; }
            Some(';')  => return self.parse_comment(),
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
                return vec![Symbol::Unknown];
            }
        }
    }
}

// end of file
