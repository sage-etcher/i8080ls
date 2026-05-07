
use tower_lsp_server::ls_types::Range;

use crate::data_types::FxDashMap;
use crate::symbol::Symbol;
use crate::lexer::Lexer;
use crate::err::{InternalErrorCode, InternalError};

#[derive(Debug, PartialEq)]
enum Stage {
    Error,
    Preprocessor,
    Label,
    Opcode,
}

#[derive(Debug)]
pub struct MacroElement {
    key: String,
    value: String,
    declaration: Range,
    references: Vec<Range>,
}

#[derive(Debug)]
pub struct Parser {
    lexer: Lexer,
    symbol: Vec<Symbol>,

    origin: Option<u32>,
    offset: Option<u32>,
    stage: Stage,

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
            stage: Stage::Error,

            error_list: Vec::default(),
            macro_list: FxDashMap::default(),
        }
    }

    fn add_error(&mut self, stage: Stage, errcode: InternalErrorCode) {
        if stage == self.stage {
            //dbg!(&stage);
            //dbg!(&errcode);
            //dbg!(&self.lexer.position);
            //dbg!(&self.symbol);
            //println!("Custom backtrace: {}", Backtrace::force_capture());
            self.error_list.push(InternalError::new(self.lexer.position, errcode));
        }
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
        let base_matches = Parser::accept_raw(&self.symbol, needles);
        if !base_matches.is_none() {
            return base_matches;
        }

        // if symbol is a $ PC replace with number
        if !Parser::accept_raw(&vec![Symbol::MacroPC], &self.symbol).is_none() {

            self.symbol = vec![Symbol::NumberWord];
            if !self.offset.is_none() {
                self.lexer.number = self.offset.unwrap();
            }
            return Parser::accept_raw(&self.symbol, needles);
        }

        // if symbol can't be an ident, return None, else
        if Parser::accept_raw(&vec![Symbol::Ident], &self.symbol).is_none() {
            return None;
        }

        // macro expansion
        let macro_values = self.eval_macro(self.lexer.ident.clone());
        if macro_values.is_none() {
            self.add_error(Stage::Opcode, InternalErrorCode::UnknownMacro);
            return None;
        }
        // check if macro matches needle
        return Parser::accept_raw(&self.symbol, needles);
    }

    fn eval_macro(&mut self, macro_name: String) -> Option<Vec<Symbol>> {
        if self.stage != Stage::Opcode {
            return None;
        }

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

    fn expect(&mut self, needles: &Vec<Symbol>, stage: Stage, 
              errcode: InternalErrorCode) -> bool {
        // {{{
        let matches = self.accept(&needles);
        if matches.is_none() {
            self.add_error(stage, errcode);
        }
        self.next_symbol();
        return !matches.is_none();
        // }}}
    }

    fn next_symbol(&mut self) {
        self.symbol = self.lexer.get_symbol();
        //dbg!(&self.symbol);
    }


    fn stmt_label_definition(&mut self) {
        // {{{
        //dbg!(&self.lexer.position);
        //dbg!(&self.symbol);
        if self.accept(&vec![Symbol::Ident]).is_none() {
            // not a label
            return;
        }

        let macro_name  = String::from(self.lexer.ident.clone());
        let declaration = self.lexer.position;
        self.next_symbol();

        if !self.accept(&vec![Symbol::Colon]).is_none() {
            self.next_symbol();
        }

        // macro definition
        if !self.accept(&vec![Symbol::MacroEQU, Symbol::MacroSET]).is_none() {
            let macro_value = String::from(self.lexer.ident.clone());

            if self.stage == Stage::Preprocessor {
                // add preprocessor value to list
                self.macro_list.insert(macro_name.clone(), MacroElement::new(
                        macro_name, macro_value, declaration));
            }

            self.next_symbol();

        } else { // implicit file offset
            if self.stage == Stage::Label {
                if self.origin.is_none() {
                    self.add_error(Stage::Opcode, InternalErrorCode::OffsetNotSet);
                    return
                }
                let macro_value = format!("0{:x}h", self.offset.unwrap());
                self.macro_list.insert(macro_name.clone(), MacroElement::new(
                        macro_name, macro_value, declaration));
            }
        }

        // }}}
    }

    fn inc_offset(&mut self, n: u32) {
        if self.origin.is_none() {
            if self.stage == Stage::Label {
                self.add_error(Stage::Opcode, InternalErrorCode::OffsetNotSet);
            }
            return;
        }

        self.offset = Some(self.offset.unwrap() + n);
    }

    fn stmt_opcode(&mut self) {
        // {{{
        let opcode_list = vec![
            Symbol::MacroORG,   Symbol::MacroEND,   
            Symbol::MacroDB,    Symbol::MacroDW,    Symbol::MacroDS,    
            //Symbol::MacroAdd,   Symbol::MacroSub,   Symbol::MacroMult,  
            //Symbol::MacroDiv,   Symbol::MacroMod,   
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
            Symbol::MacroORG => {
                if !self.accept(&vec![Symbol::NumberWord]).is_none() {
                    self.offset = Some(self.lexer.number);
                    self.origin = Some(self.lexer.number);
                } else {
                    // missing word, throw error
                    self.add_error(Stage::Opcode, InternalErrorCode::SyntaxIntermediateWord);
                }

                self.next_symbol();
            }
            Symbol::MacroEND => {
                if !self.accept(&vec![Symbol::NumberWord]).is_none() {
                    if self.origin != Some(self.lexer.number) &&
                       self.stage  == Stage::Label
                    {
                        self.add_error(Stage::Opcode, InternalErrorCode::EndOffsetMismatch);
                    }
                    self.next_symbol();
                }
                self.origin = None;
                self.offset = None;
            }
            Symbol::MacroDB => {
                if self.accept(&vec![Symbol::NumberByte, Symbol::StringASCII]).is_none() {
                    self.add_error(Stage::Opcode, InternalErrorCode::SyntaxDataByte);
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
                        self.add_error(Stage::Opcode, InternalErrorCode::SyntaxDataByte);
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
                if self.accept(&vec![Symbol::NumberWord]).is_none() {
                    self.add_error(Stage::Opcode, InternalErrorCode::SyntaxDataWord);
                    self.next_symbol();
                    return;
                }
                self.inc_offset(2);
                self.next_symbol();

                while !self.accept(&vec![Symbol::Comma]).is_none() {
                    self.next_symbol();

                    if self.accept(&vec![Symbol::NumberWord]).is_none() {
                        self.add_error(Stage::Opcode, InternalErrorCode::SyntaxDataWord);
                        self.next_symbol();
                        return;
                    }

                    self.inc_offset(2);
                    self.next_symbol();

                }
            }
            Symbol::MacroDS => {
                if self.accept(&vec![Symbol::NumberWord]).is_none() {
                    self.add_error(Stage::Opcode, InternalErrorCode::SyntaxIntermediateWord);
                    self.next_symbol();
                    return;
                }
                self.inc_offset(self.lexer.number);
                self.next_symbol();
            }

            /* stage 3 */
            Symbol::OpcodeLXI => {
                self.inc_offset(3);
                if !(self.expect(&reg16_alu, 
                                 Stage::Opcode, InternalErrorCode::SyntaxRegisterPair)
                     && self.expect(&vec![Symbol::Comma], Stage::Opcode, 
                                    InternalErrorCode::SyntaxMissingComma)
                     && self.expect(&vec![Symbol::NumberWord], Stage::Opcode, 
                                    InternalErrorCode::SyntaxIntermediateWord))
                {
                    return;
                }
            }
            Symbol::OpcodeINX | Symbol::OpcodeDCX | Symbol::OpcodeDAD => {
                self.inc_offset(1);
                if !self.expect(&reg16_alu, Stage::Opcode, 
                                InternalErrorCode::SyntaxRegisterPair) {
                    return;
                }
            }
            Symbol::OpcodeSTAX | Symbol::OpcodeLDAX => {
                if !self.expect(&vec![Symbol::RegPairBC, Symbol::RegPairDE],
                                Stage::Opcode, InternalErrorCode::SyntaxRegisterPairBD) {
                    return;
                }
            }
            Symbol::OpcodePUSH | Symbol::OpcodePOP => {
                self.inc_offset(1);
                let reg16_push = vec![
                    Symbol::RegPairPSW, Symbol::RegPairBC, 
                    Symbol::RegPairDE,  Symbol::RegPairHL,
                ];

                if !self.expect(&reg16_push, Stage::Opcode, 
                                InternalErrorCode::SyntaxRegisterPairPush) {
                    return;
                }
            }
            Symbol::OpcodeOUT  | Symbol::OpcodeIN   |
            Symbol::OpcodeADI  | Symbol::OpcodeACI  |
            Symbol::OpcodeSUI  | Symbol::OpcodeSBI  |
            Symbol::OpcodeANI  | Symbol::OpcodeXRI  |
            Symbol::OpcodeORI  | Symbol::OpcodeCPI => {
                self.inc_offset(2);
                if !self.expect(&vec![Symbol::NumberByte], Stage::Opcode, 
                                InternalErrorCode::SyntaxIntermediateByte) {
                    return;
                }
            }
            Symbol::OpcodeRST => {
                self.inc_offset(1);
                if self.accept(&vec![Symbol::NumberByte]).is_none()
                   || self.lexer.number > 7
                {
                    //dbg!(&self.lexer.number);
                    self.next_symbol();
                    self.add_error(Stage::Opcode, 
                                   InternalErrorCode::SyntaxIntermediateRST);
                    return;
                }
                self.next_symbol();
            }
            Symbol::OpcodeMVI => {
                self.inc_offset(2);
                if !(self.expect(&reg8, Stage::Opcode, 
                                 InternalErrorCode::SyntaxRegister)
                     && self.expect(&vec![Symbol::Comma], Stage::Opcode, 
                                    InternalErrorCode::SyntaxMissingComma)
                     && self.expect(&vec![Symbol::NumberByte], Stage::Opcode, 
                                    InternalErrorCode::SyntaxIntermediateByte))
                {
                    return;
                }
            }
            Symbol::OpcodeMOV => {
                self.inc_offset(1);
                if !(self.expect(&reg8, Stage::Opcode, 
                                 InternalErrorCode::SyntaxRegister)
                     && self.expect(&vec![Symbol::Comma], Stage::Opcode, 
                                    InternalErrorCode::SyntaxMissingComma)
                     && self.expect(&reg8, Stage::Opcode, 
                                    InternalErrorCode::SyntaxRegister))
                {
                    return;
                }
            }
            Symbol::OpcodeINR  | Symbol::OpcodeDCR  |
            Symbol::OpcodeADD  | Symbol::OpcodeADC  |
            Symbol::OpcodeSUB  | Symbol::OpcodeSBB  |
            Symbol::OpcodeANA  | Symbol::OpcodeXRA  |
            Symbol::OpcodeORA  | Symbol::OpcodeCMP => {
                self.inc_offset(1);
                if !self.expect(&reg8, Stage::Opcode, 
                                InternalErrorCode::SyntaxRegister) {
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
                self.inc_offset(3);
                if !self.expect(&vec![Symbol::NumberWord], Stage::Opcode, 
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
                self.inc_offset(1);
                return;
            }
            _ => {
                // unreachable
                self.add_error(Stage::Opcode, InternalErrorCode::SyntaxUnknown);
                return;
            }
        }
        // }}}
    }

    fn parse_step(&mut self, stage: Stage) {
        self.stage = stage;

        self.next_symbol();
        while !self.lexer.ch.is_none() {

            self.stmt_label_definition();
            self.stmt_opcode();

            if !self.accept(&vec![Symbol::Comment]).is_none() {
                self.next_symbol();
            }

            self.expect(&vec![Symbol::Newline], 
                        Stage::Opcode, InternalErrorCode::SyntaxNewline);
        }
    }

    pub fn parse(&mut self) {
        let stages: Vec<Stage> = vec!(
            Stage::Preprocessor,    // get preprocessor set/equ macros
            Stage::Label,           // get position dependant labels
            Stage::Opcode,          // validate symbols
        );

        for stage in stages {
            self.origin = None;
            self.offset = None;
            self.lexer = Lexer::new(&self.lexer.file_content);
            self.lexer.read_ch();
            self.parse_step(stage);
        }
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
