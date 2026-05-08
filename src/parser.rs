
use tower_lsp_server::ls_types::Range;

use crate::data_types::FxDashMap;
use crate::symbol::Symbol;
use crate::lexer::Lexer;
use crate::err::{InternalErrorCode, InternalError};

#[derive(Debug, Clone)]
pub struct MacroElement {
    key: String,
    value: String,
    pub declaration: Range,
    pub references: Vec<Range>,
}

#[derive(Debug)]
pub struct Parser {
    lexer: Lexer,
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
    pub fn new(key: String, value: String, declaration: Range) -> Self {
        Self {
            key, 
            value,
            declaration,
            references: Vec::default(),
        }
    }
}

impl Parser {
    pub fn new(file_content: &str) -> Self {
        Self {
            lexer:  Lexer::new(file_content),
            symbol: Vec::default(),

            origin: None,
            offset: None,

            error_list: Vec::default(),
            macro_list: FxDashMap::default(),
        }
    }

    fn add_error(&mut self, errcode: InternalErrorCode) {
        self.error_list.push(InternalError::new(self.lexer.position, errcode));
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

    fn eval_macro(&mut self, macro_name: String) -> Option<Vec<Symbol>> {
        let macro_lookup = self.macro_list.get_mut(&macro_name);
        if macro_lookup.is_none() {
            return None;
        }

        let mut macro_value = macro_lookup.unwrap();
        macro_value.references.push(self.lexer.position);

        let mut sub_lexer = Lexer::new(&macro_value.value().value);

        sub_lexer.read_ch();
        let macro_syms = sub_lexer.get_symbol();

        match macro_syms.len() {
            0 => return None,
            _ => {
                self.symbol       = macro_syms.clone();
                self.lexer.number = sub_lexer.number;
                self.lexer.ident  = sub_lexer.ident;
                return Some(macro_syms);
            },
        }
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

    fn next_symbol(&mut self) {
        self.symbol = self.lexer.get_symbol();
        //dbg!(&self.symbol);
    }


    fn inc_offset(&mut self, n: u32) {
        if self.origin.is_none() {
            self.add_error(InternalErrorCode::OffsetNotSet);
            return;
        }

        self.offset = Some(self.offset.unwrap() + n);
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
                    //dbg!(&self.lexer.number);
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

        // disallow duplicate definitions
        if !self.macro_list.get(&macro_name.clone()).is_none() {
            self.lexer.position = macro_position;
            self.add_error(InternalErrorCode::DuplicateIdent);
            return;
        }

        // add macro to list
        self.macro_list.insert(macro_name.clone(), MacroElement::new(
                macro_name, macro_value, macro_position));
        // }}}
    }

    fn parse_stage0(&mut self) {
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
        self.next_symbol();

        // disallow duplicate definitions
        if !self.macro_list.get(&macro_name.clone()).is_none() {
            self.lexer.position = macro_position;
            self.add_error(InternalErrorCode::DuplicateIdent);
            return;
        }

        // add macro to list
        self.macro_list.insert(macro_name.clone(), MacroElement::new(
                macro_name, macro_value, macro_position));
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
                self.next_symbol();
                self.inc_offset(1);
            }
            Symbol::OpcodeMVI  |
            Symbol::OpcodeOUT  | Symbol::OpcodeIN   |
            Symbol::OpcodeADI  | Symbol::OpcodeACI  |
            Symbol::OpcodeSUI  | Symbol::OpcodeSBI  |
            Symbol::OpcodeANI  | Symbol::OpcodeXRI  |
            Symbol::OpcodeORI  | Symbol::OpcodeCPI => {
                self.next_symbol();
                self.inc_offset(2);
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
                self.next_symbol();
                self.inc_offset(3);
            }
            _ => {
                return;
            }
        }
        // }}}
    }

    fn parse_stage1(&mut self) {
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
    }

    fn add_reference(&mut self) {
        let macro_name = self.lexer.ident.clone();
        let macro_get  = self.macro_list.get(&macro_name);

        if macro_get.is_none() {
            self.add_error(InternalErrorCode::UnknownMacro);
            return;
        }

        let macro_pair  = macro_get.unwrap();
        let macro_value = macro_pair.value();
        macro_value.references.push(self.lexer.position);
    }

    fn parse_stage2(&mut self) {
        // get label references
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

    fn parse_stage3(&mut self) {
        // validate structure
        self.next_symbol();
        while self.accept(&vec![Symbol::EOF]).is_none() {
            self.stmt_skip_declarations();
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
        self.parse_stage0();

        // expand preproc values

        // collect labels
        self.lexer = Lexer::new(&self.lexer.file_content);
        self.lexer.read_ch();
        self.parse_stage1();

        self.lexer = Lexer::new(&self.lexer.file_content);
        self.lexer.read_ch();
        self.parse_stage2();

        dbg!(&self);
        return;

        // expand all labels

        // validate opcodes
        self.lexer = Lexer::new(&self.lexer.file_content);
        self.lexer.read_ch();
        self.parse_stage3();
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
