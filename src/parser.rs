
use tower_lsp_server::ls_types::Range;

use crate::data_types::{FxDashMap, FxDashSet};
use crate::symbol::Symbol;
use crate::lexer::Lexer;
use crate::err::{InternalErrorCode, InternalError};

#[derive(Debug, Clone)]
pub struct MacroElement {
    key: String,
    value: Option<String>,
    pub declaration: Option<Range>,
    pub references: FxDashSet<Range>,
}

#[derive(Debug)]
pub struct Parser {
    lexer: Lexer,
    lexer_stack: Vec<Lexer>,
    symbol: Vec<Symbol>,

    origin: Option<u32>,
    offset: Option<u32>,

    pub error_list: Vec<InternalError>,
    pub macro_list: FxDashMap<String, MacroElement>,
}

#[derive(Debug)]
pub struct FileContext {
    pub parser: Parser,
}

impl MacroElement {
    pub fn new(key: String, value: Option<String>, declaration: Option<Range>) -> Self {
        Self {
            key, 
            value,
            declaration,
            references: FxDashSet::default(),
        }
    }
}

impl Parser {
    pub fn new(file_content: &str) -> Self {
        Self {
            lexer: Lexer::new(file_content),
            lexer_stack: Vec::default(),
            symbol: Vec::default(),

            origin: None,
            offset: None,

            error_list: Vec::default(),
            macro_list: FxDashMap::default(),
        }
    }

    fn add_error(&mut self, errcode: InternalErrorCode) {
        let pos: Range;

        if self.lexer_stack.len() == 0 {
            pos = self.lexer.position;
        } else {
            pos = self.lexer_stack[0].position;
        }

        self.error_list.push(InternalError::new(pos, errcode));
    }

    fn accept_raw(haystack: &Vec<Symbol>, needles: &Vec<Symbol>) -> Option<Vec<Symbol>> {
        // {{{
        let mut matches = Vec::new();

        for needle in needles {
            for hay in haystack {
                if *hay == *needle {
                    matches.push(*needle);
                }
            }
        }

        match matches.len() {
            0 => return None,
            _ => return Some(matches),
        }
        // }}}
    }

    fn accept(&mut self, needles: &Vec<Symbol>) -> Option<Vec<Symbol>> {
        return Parser::accept_raw(&self.symbol, needles);
    }

    fn expect(&mut self, needles: &Vec<Symbol>, errcode: InternalErrorCode) -> bool {
        // {{{
        let matches = self.accept(&needles);
        if matches.is_none() {
            self.add_error(errcode);
        }
        self.next_symbol();
        return !matches.is_none();
        // }}}
    }

    // MacroElement control
    fn add_reference(&mut self) {
        // {{{
        let key = self.lexer.ident.clone();

        let ref_pos = self.lexer.position;
        if !self.macro_list.contains_key(&key) {
            // make new element
            self.macro_list.insert(key.clone(), MacroElement::new(
                    key.clone(), None, None));
        }

        self.macro_list.get_mut(&key).unwrap().references.insert(ref_pos);
        // }}}
    }

    fn add_declaration(&mut self, key: String, value: String, pos: Range) {
        // {{{
        // add new macro to list
        if !self.macro_list.contains_key(&key) {
            self.macro_list.insert(key.clone(), MacroElement::new(
                    key, Some(value), Some(pos)));
            return;
        }

        // disallow duplicate declaration
        if !self.macro_list.get_mut(&key).unwrap().declaration.is_none() {
            self.add_error(InternalErrorCode::DuplicateIdent);
            return;
        }

        // add macro declaration + definition
        self.macro_list.get_mut(&key).unwrap().value       = Some(value);
        self.macro_list.get_mut(&key).unwrap().declaration = Some(pos);
        self.macro_list.get_mut(&key).unwrap().references.remove(&pos);
        // }}}
    }

    fn contains_macro(&mut self) -> bool {
        // {{{
        let key = &self.lexer.ident;
        let macro_get = self.macro_list.get(key);

        if macro_get.is_none() {
            return false;
        }

        let macro_unwrap = macro_get.unwrap();

        if macro_unwrap.declaration == Some(self.lexer.position) {
            return false;
        }

        return macro_unwrap.value != None;
        // }}}
    }

    fn macro_is_declaration(&mut self) -> bool {
        // {{{
        let key = &self.lexer.ident;
        let macro_get = self.macro_list.get(key);

        if macro_get.is_none() {
            return false;
        }

        let macro_unwrap = macro_get.unwrap();
        return macro_unwrap.declaration == Some(self.lexer.position);
        // }}}
    }

    fn eval_macro(&mut self) {
        // {{{
        if self.contains_macro() {
            let key = &self.lexer.ident;
            let file_content = self.macro_list.get_mut(key).unwrap().value.clone();

            self.lexer_push(file_content.unwrap());
        }
        // }}}
    }

    fn lexer_push(&mut self, file_content: String) {
        // {{{
        self.lexer_stack.push(Lexer::from(&self.lexer));
        self.lexer = Lexer::new(&file_content);
        self.lexer.read_ch();
        self.next_symbol();
        // }}}
    }

    fn lexer_pop(&mut self) {
        // {{{
        if self.lexer_stack.len() == 0 {
            return;
        }

        self.lexer = Lexer::from(&self.lexer_stack.pop().unwrap());
        self.next_symbol();
        // }}}
    }

    fn next_symbol(&mut self) {
        // {{{
        self.symbol = self.lexer.get_symbol();

        if !self.accept(&vec![Symbol::EOF]).is_none() {
            self.lexer_pop();
        }

        // return if not IDENT
        if self.accept(&vec![Symbol::Ident]).is_none() {
            dbg!(&self.symbol);
            return;
        }

        // evaluate macro, and add reference if applicable
        // returns true when the evaluation is a macro
        if !self.macro_is_declaration() {
            self.add_reference();
        }
        self.eval_macro();

        dbg!(&self.symbol);
        // }}}
    }

    fn inc_offset(&mut self, n: u32) {
        // {{{
        if self.origin.is_none() {
            self.add_error(InternalErrorCode::OffsetNotSet);
            return;
        }

        self.offset = Some(self.offset.unwrap() + n);
        // }}}
    }

    fn stmt_opcode(&mut self) {
        // {{{
        let opcode_list = vec![
            Symbol::OpcodeACI,  Symbol::OpcodeADC,  Symbol::OpcodeADD,  
            Symbol::OpcodeADI,  Symbol::OpcodeANA,  Symbol::OpcodeANI,  
            Symbol::OpcodeCALL, Symbol::OpcodeCC,   Symbol::OpcodeCM,   
            Symbol::OpcodeCMA,  Symbol::OpcodeCMC,  Symbol::OpcodeCMP,  
            Symbol::OpcodeCNC,  Symbol::OpcodeCNZ,  Symbol::OpcodeCP,   
            Symbol::OpcodeCPE,  Symbol::OpcodeCPI,  Symbol::OpcodeCPO,  
            Symbol::OpcodeCZ,   Symbol::OpcodeDAA,  Symbol::OpcodeDAD,  
            Symbol::OpcodeDCR,  Symbol::OpcodeDCX,  Symbol::OpcodeDI,   
            Symbol::OpcodeEI,   Symbol::OpcodeHLT,  Symbol::OpcodeIN,   
            Symbol::OpcodeINR,  Symbol::OpcodeINX,  Symbol::OpcodeJC,   
            Symbol::OpcodeJM,   Symbol::OpcodeJMP,  Symbol::OpcodeJNC,  
            Symbol::OpcodeJNZ,  Symbol::OpcodeJP,   Symbol::OpcodeJPE,  
            Symbol::OpcodeJPO,  Symbol::OpcodeJZ,   Symbol::OpcodeLDA,  
            Symbol::OpcodeLDAX, Symbol::OpcodeLHLD, Symbol::OpcodeLXI,  
            Symbol::OpcodeMOV,  Symbol::OpcodeMVI,  Symbol::OpcodeNOP,  
            Symbol::OpcodeORA,  Symbol::OpcodeORI,  Symbol::OpcodeOUT,  
            Symbol::OpcodePCHL, Symbol::OpcodePOP,  Symbol::OpcodePUSH, 
            Symbol::OpcodeRAL,  Symbol::OpcodeRAR,  Symbol::OpcodeRC,   
            Symbol::OpcodeRET,  Symbol::OpcodeRLC,  Symbol::OpcodeRM,   
            Symbol::OpcodeRNC,  Symbol::OpcodeRNZ,  Symbol::OpcodeRP,   
            Symbol::OpcodeRPE,  Symbol::OpcodeRPO,  Symbol::OpcodeRRC,  
            Symbol::OpcodeRST,  Symbol::OpcodeRZ,   Symbol::OpcodeSBB,  
            Symbol::OpcodeSBI,  Symbol::OpcodeSHLD, Symbol::OpcodeSPHL, 
            Symbol::OpcodeSTA,  Symbol::OpcodeSTAX, Symbol::OpcodeSTC,  
            Symbol::OpcodeSUB,  Symbol::OpcodeSUI,  Symbol::OpcodeXCHG, 
            Symbol::OpcodeXRA,  Symbol::OpcodeXRI,  Symbol::OpcodeXTHL, 
        ];

        let reg8 = vec![
            Symbol::RegB, Symbol::RegC, Symbol::RegD, Symbol::RegE,
            Symbol::RegH, Symbol::RegL, Symbol::RegM, Symbol::RegA,
        ];


        let reg16_alu = vec![
            Symbol::RegPairBC, Symbol::RegPairDE,
            Symbol::RegPairHL, Symbol::RegPairSP,
        ];


        let opcode_vec: Option<Vec<Symbol>> = self.accept(&opcode_list);
        if opcode_vec.is_none() {
            // not an opcode
            return;
        }
        let opcode = opcode_vec.unwrap()[0];
        self.next_symbol();

        match opcode {
            Symbol::OpcodeLXI => {
                if !(self.expect(&reg16_alu, 
                                 InternalErrorCode::SyntaxRegisterPair)
                     && self.expect(&vec![Symbol::Comma], 
                                    InternalErrorCode::SyntaxMissingComma)
                     && self.expect(&vec![Symbol::NumberWord], 
                                    InternalErrorCode::SyntaxIntermediateWord))
                {
                    return;
                }
            }
            Symbol::OpcodeINX | Symbol::OpcodeDCX | Symbol::OpcodeDAD => {
                if !self.expect(&reg16_alu, InternalErrorCode::SyntaxRegisterPair) {
                    return;
                }
            }
            Symbol::OpcodeSTAX | Symbol::OpcodeLDAX => {
                if !self.expect(&vec![Symbol::RegPairBC, Symbol::RegPairDE], 
                                InternalErrorCode::SyntaxRegisterPairBD) {
                    return;
                }
            }
            Symbol::OpcodePUSH | Symbol::OpcodePOP => {
                let reg16_push = vec![
                    Symbol::RegPairPSW, Symbol::RegPairBC, 
                    Symbol::RegPairDE,  Symbol::RegPairHL,
                ];

                if !self.expect(&reg16_push, 
                                InternalErrorCode::SyntaxRegisterPairPush) {
                    return;
                }
            }
            Symbol::OpcodeOUT  | Symbol::OpcodeIN   |
            Symbol::OpcodeADI  | Symbol::OpcodeACI  |
            Symbol::OpcodeSUI  | Symbol::OpcodeSBI  |
            Symbol::OpcodeANI  | Symbol::OpcodeXRI  |
            Symbol::OpcodeORI  | Symbol::OpcodeCPI => {
                if !self.expect(&vec![Symbol::NumberByte], 
                                InternalErrorCode::SyntaxIntermediateByte) {
                    return;
                }
            }
            Symbol::OpcodeRST => {
                if self.accept(&vec![Symbol::NumberByte]).is_none()
                   || self.lexer.number > 7
                {
                    self.next_symbol();
                    self.add_error(InternalErrorCode::SyntaxIntermediateRST);
                    return;
                }
                self.next_symbol();
            }
            Symbol::OpcodeMVI => {
                if !(self.expect(&reg8, InternalErrorCode::SyntaxRegister)
                     && self.expect(&vec![Symbol::Comma],
                                    InternalErrorCode::SyntaxMissingComma)
                     && self.expect(&vec![Symbol::NumberByte],
                                    InternalErrorCode::SyntaxIntermediateByte))
                {
                    return;
                }
            }
            Symbol::OpcodeMOV => {
                if !(self.expect(&reg8, InternalErrorCode::SyntaxRegister)
                     && self.expect(&vec![Symbol::Comma],
                                    InternalErrorCode::SyntaxMissingComma)
                     && self.expect(&reg8, InternalErrorCode::SyntaxRegister))
                {
                    return;
                }
            }
            Symbol::OpcodeINR  | Symbol::OpcodeDCR  |
            Symbol::OpcodeADD  | Symbol::OpcodeADC  |
            Symbol::OpcodeSUB  | Symbol::OpcodeSBB  |
            Symbol::OpcodeANA  | Symbol::OpcodeXRA  |
            Symbol::OpcodeORA  | Symbol::OpcodeCMP => {
                if !self.expect(&reg8, InternalErrorCode::SyntaxRegister) {
                    return;
                }
            }
            Symbol::OpcodeSHLD | Symbol::OpcodeLHLD |
            Symbol::OpcodeSTA  | Symbol::OpcodeLDA  |
            Symbol::OpcodeJMP  | Symbol::OpcodeJNZ  | Symbol::OpcodeJNC  |
            Symbol::OpcodeJPO  | Symbol::OpcodeJP   | Symbol::OpcodeJZ   |
            Symbol::OpcodeJC   | Symbol::OpcodeJPE  | Symbol::OpcodeJM   |
            Symbol::OpcodeCALL | Symbol::OpcodeCNZ  | Symbol::OpcodeCNC  |
            Symbol::OpcodeCPO  | Symbol::OpcodeCP   | Symbol::OpcodeCZ   |
            Symbol::OpcodeCC   | Symbol::OpcodeCPE  | Symbol::OpcodeCM => {
                if !self.expect(&vec![Symbol::NumberWord], 
                                InternalErrorCode::SyntaxIntermediateWord) {
                    return;
                }
            }
            Symbol::OpcodeNOP  | Symbol::OpcodeHLT  |
            Symbol::OpcodeRLC  | Symbol::OpcodeRAL  |
            Symbol::OpcodeDAA  | Symbol::OpcodeSTC  |
            Symbol::OpcodeRRC  | Symbol::OpcodeRAR  |
            Symbol::OpcodeCMA  | Symbol::OpcodeCMC  |
            Symbol::OpcodeRET  | Symbol::OpcodeRNZ  | Symbol::OpcodeRNC  |
            Symbol::OpcodeRPO  | Symbol::OpcodeRP   | Symbol::OpcodeRZ   |
            Symbol::OpcodeRC   | Symbol::OpcodeRPE  | Symbol::OpcodeRM   |
            Symbol::OpcodeXTHL | Symbol::OpcodeXCHG |
            Symbol::OpcodeDI   | Symbol::OpcodeEI   |
            Symbol::OpcodeSPHL | Symbol::OpcodePCHL => {
                return;
            }
            _ => {
                // unreachable
                self.add_error(InternalErrorCode::SyntaxUnknown);
                return;
            }
        }
        // }}}
    }

    // Parser function
    fn stmt_macro_dec(&mut self) {
        // {{{
        // get EQU definitions
        if self.accept(&vec![Symbol::Ident]).is_none() {
            return;
        }

        // get name and position
        let macro_name = String::from(self.lexer.ident.clone());
        let macro_position = self.lexer.position;

        // allow colon but dont do anything if it is there
        self.next_symbol();
        if !self.accept(&vec![Symbol::Colon]).is_none() {
            self.next_symbol();
        }

        // handle EQU
        if self.accept(&vec![Symbol::MacroEQU]).is_none() {
            return;
        }

        let macro_value = self.lexer.ident.clone();
        self.next_symbol();

        self.add_declaration(macro_name, macro_value, macro_position);
        // }}}
    }

    fn parse_get_macro_definitions(&mut self) {
        // get static preprocessor values, EQU, SET
        self.next_symbol();
        while self.accept(&vec![Symbol::EOF]).is_none() {
            self.stmt_macro_dec();

            // ignore remaining lines
            while self.accept(&vec![Symbol::Newline]).is_none() {
                self.next_symbol();
            }
            self.expect(&vec![Symbol::Newline], InternalErrorCode::SyntaxNewline);
        }
    }

    fn stmt_label_dec(&mut self) {
        // {{{
        // get label definitions
        if self.accept(&vec![Symbol::Ident]).is_none() {
            return;
        }

        // get name and position
        let macro_name = String::from(self.lexer.ident.clone());
        let macro_position = self.lexer.position;

        // allow colon but dont do anything if it is there
        self.next_symbol();
        if !self.accept(&vec![Symbol::Colon]).is_none() {
            self.next_symbol();
        }

        // quit if ident is a macro definition
        if !self.accept(&vec![Symbol::MacroEQU, Symbol::MacroSET]).is_none() {
            self.next_symbol();
            return;
        }

        // require that offset is set
        if self.offset.is_none() {
            self.lexer.position = macro_position;
            self.add_error(InternalErrorCode::OffsetNotSet);
            return;
        }

        // add new macro from offset
        let macro_value = format!("0{:x}h", self.offset.unwrap());
        //self.next_symbol();

        // add macro to list
        self.add_declaration(macro_name, macro_value, macro_position);
        // }}}
    }

    fn stmt_handle_offset(&mut self) {
        // {{{
        if self.symbol.len() > 1 {
            return;
        }

        match self.symbol[0] {
            Symbol::MacroORG => {
                self.next_symbol();

                if !self.accept(&vec![Symbol::NumberWord]).is_none() {
                    self.origin = Some(self.lexer.number);
                    self.offset = Some(self.lexer.number);
                } else {
                    self.add_error(InternalErrorCode::SyntaxIntermediateWord);
                }
                self.next_symbol();
            }
            Symbol::MacroEND => {
                self.next_symbol();

                if !self.accept(&vec![Symbol::NumberWord]).is_none() {
                    if self.origin != Some(self.lexer.number) {
                        self.add_error(InternalErrorCode::EndOffsetMismatch);
                    }
                    self.next_symbol();
                }
                self.origin = None;
                self.offset = None;
            }
            Symbol::MacroDB => {
                self.next_symbol();

                if self.accept(&vec![Symbol::NumberByte, Symbol::StringASCII]).is_none() {
                    self.add_error(InternalErrorCode::SyntaxDataByte);
                    self.next_symbol();
                    return;
                }
                if !self.accept(&vec![Symbol::StringASCII]).is_none() {
                    self.inc_offset(self.lexer.ident.len() as u32);
                } else {
                    self.inc_offset(1);
                }
                self.next_symbol();

                while !self.accept(&vec![Symbol::Comma]).is_none() {
                    self.next_symbol();

                    if self.accept(&vec![Symbol::NumberByte, Symbol::StringASCII]).is_none() {
                        self.add_error(InternalErrorCode::SyntaxDataByte);
                        self.next_symbol();
                        return;
                    }

                    if !self.accept(&vec![Symbol::StringASCII]).is_none() {
                        self.inc_offset(self.lexer.ident.len() as u32);
                    } else {
                        self.inc_offset(1);
                    }
                    self.next_symbol();
                }
            }
            Symbol::MacroDW => {
                self.next_symbol();

                if self.accept(&vec![Symbol::NumberWord]).is_none() {
                    self.add_error(InternalErrorCode::SyntaxDataWord);
                    self.next_symbol();
                    return;
                }
                self.inc_offset(2);
                self.next_symbol();

                while !self.accept(&vec![Symbol::Comma]).is_none() {
                    self.next_symbol();

                    if self.accept(&vec![Symbol::NumberWord]).is_none() {
                        self.add_error(InternalErrorCode::SyntaxDataWord);
                        self.next_symbol();
                        return;
                    }

                    self.inc_offset(2);
                    self.next_symbol();
                }
            }
            Symbol::MacroDS => {
                self.next_symbol();

                if !self.accept(&vec![Symbol::NumberWord]).is_none() {
                    self.inc_offset(self.lexer.number);
                } else {
                    self.add_error(InternalErrorCode::SyntaxIntermediateWord);
                }
                self.next_symbol();
            }
            Symbol::OpcodeMOV  |
            Symbol::OpcodeSTAX | Symbol::OpcodeLDAX |
            Symbol::OpcodePUSH | Symbol::OpcodePOP  |
            Symbol::OpcodeINR  | Symbol::OpcodeDCR  |
            Symbol::OpcodeINX  | Symbol::OpcodeDCX  | Symbol::OpcodeDAD  |
            Symbol::OpcodeADD  | Symbol::OpcodeADC  |
            Symbol::OpcodeSUB  | Symbol::OpcodeSBB  |
            Symbol::OpcodeANA  | Symbol::OpcodeXRA  |
            Symbol::OpcodeORA  | Symbol::OpcodeCMP  |
            Symbol::OpcodeNOP  | Symbol::OpcodeHLT  | Symbol::OpcodeRST  |
            Symbol::OpcodeRLC  | Symbol::OpcodeRAL  |
            Symbol::OpcodeDAA  | Symbol::OpcodeSTC  |
            Symbol::OpcodeRRC  | Symbol::OpcodeRAR  |
            Symbol::OpcodeCMA  | Symbol::OpcodeCMC  |
            Symbol::OpcodeRET  | Symbol::OpcodeRNZ  | Symbol::OpcodeRNC  |
            Symbol::OpcodeRPO  | Symbol::OpcodeRP   | Symbol::OpcodeRZ   |
            Symbol::OpcodeRC   | Symbol::OpcodeRPE  | Symbol::OpcodeRM   |
            Symbol::OpcodeXTHL | Symbol::OpcodeXCHG |
            Symbol::OpcodeDI   | Symbol::OpcodeEI   |
            Symbol::OpcodeSPHL | Symbol::OpcodePCHL => {
                self.inc_offset(1);
                self.next_symbol();
            }
            Symbol::OpcodeMVI  |
            Symbol::OpcodeOUT  | Symbol::OpcodeIN   |
            Symbol::OpcodeADI  | Symbol::OpcodeACI  |
            Symbol::OpcodeSUI  | Symbol::OpcodeSBI  |
            Symbol::OpcodeANI  | Symbol::OpcodeXRI  |
            Symbol::OpcodeORI  | Symbol::OpcodeCPI => {
                self.inc_offset(2);
                self.next_symbol();
            }
            Symbol::OpcodeLXI  |
            Symbol::OpcodeSHLD | Symbol::OpcodeLHLD |
            Symbol::OpcodeSTA  | Symbol::OpcodeLDA  |
            Symbol::OpcodeJMP  | Symbol::OpcodeJNZ  | Symbol::OpcodeJNC  |
            Symbol::OpcodeJPO  | Symbol::OpcodeJP   | Symbol::OpcodeJZ   |
            Symbol::OpcodeJC   | Symbol::OpcodeJPE  | Symbol::OpcodeJM   |
            Symbol::OpcodeCALL | Symbol::OpcodeCNZ  | Symbol::OpcodeCNC  |
            Symbol::OpcodeCPO  | Symbol::OpcodeCP   | Symbol::OpcodeCZ   |
            Symbol::OpcodeCC   | Symbol::OpcodeCPE  | Symbol::OpcodeCM => {
                self.inc_offset(3);
                self.next_symbol();
            }
            _ => {
                return;
            }
        }
        // }}}
    }

    fn parse_get_label_definitions(&mut self) {
        // get label declarations
        self.next_symbol();
        while self.accept(&vec![Symbol::EOF]).is_none() {
            self.stmt_label_dec();
            self.stmt_handle_offset();

            // ignore remaining line
            while self.accept(&vec![Symbol::Newline]).is_none() {
                self.next_symbol();
            }
            self.expect(&vec![Symbol::Newline], InternalErrorCode::SyntaxNewline);
        }
    }

    fn stmt_skip_declarations(&mut self) {
        // {{{
        if self.accept(&vec![Symbol::Ident]).is_none() {
            return;
        }

        self.next_symbol();

        if !self.accept(&vec![Symbol::Colon]).is_none() {
            self.next_symbol();
        }

        if !self.accept(&vec![Symbol::MacroEQU, Symbol::MacroSET]).is_none() {
            self.next_symbol();
        }
        // }}}
    }

    fn parse_get_references(&mut self) {
        // get label/macro references
        self.next_symbol();
        while self.accept(&vec![Symbol::EOF]).is_none() {
            self.stmt_skip_declarations();

            // ignore remaining line
            while self.accept(&vec![Symbol::Newline]).is_none() {
                if !self.accept(&vec![Symbol::Ident]).is_none() {
                    self.add_reference();
                }
                self.next_symbol();
            }
            self.expect(&vec![Symbol::Newline], InternalErrorCode::SyntaxNewline);
        }
    }

    fn stmt_skip_macros(&mut self) {
        // only act on macros
        if self.accept(&vec!(Symbol::MacroDB,  Symbol::MacroDW,  
                             Symbol::MacroDS,  Symbol::MacroORG, 
                             Symbol::MacroEND)).is_none() {
            return;
        }

        self.next_symbol();
        while self.accept(&vec!(Symbol::Newline)).is_none() {
            self.next_symbol();
        }

    }

    fn parse_validate_opcodes(&mut self) {
        // validate structure
        self.next_symbol();
        while self.accept(&vec![Symbol::EOF]).is_none() {
            self.stmt_skip_declarations();
            self.stmt_skip_macros();
            self.stmt_opcode();

            if !self.accept(&vec![Symbol::Comment]).is_none() {
                self.next_symbol();
            }

            self.expect(&vec![Symbol::Newline], InternalErrorCode::SyntaxNewline);
        }
    }

    pub fn parse(&mut self) {
        // collect fixed preproc values
        self.lexer.read_ch();
        self.parse_get_macro_definitions();

        // collect labels
        self.lexer.reset();
        self.lexer.read_ch();
        self.parse_get_label_definitions();

        // validate opcodes
        self.lexer.reset();
        self.lexer.read_ch();
        self.parse_validate_opcodes();

        dbg!(&self);
        return;

    }
}

impl FileContext {
    pub fn new(file_content: &str) -> Self {
        Self {
            parser: Parser::new(file_content),
        }
    }

    pub fn parse_file(&mut self) {
        // parse code
        self.parser.parse();
    }
}

// vim: fdm=marker
// end of file
