
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

    stage: Stage,

    pub error_list: Vec<InternalError>,
    pub macro_list: FxDashMap<String, MacroElement>,
    pub label_list: FxDashMap<String, MacroElement>,
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

            stage: Stage::Error,

            error_list: Vec::default(),
            macro_list: FxDashMap::default(),
            label_list: FxDashMap::default(),

        }
    }

    fn add_error(&mut self, errcode: InternalErrorCode) {
        self.error_list.push(InternalError::new(self.lexer.position, errcode));
    }

    fn accept(&mut self, needles: &Vec<Symbol>) -> Option<Vec<Symbol>> {
        // {{{
        let mut matches = Vec::new();

        for needle in needles {
            for hay in &self.symbol {
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


    fn stmt_label_definition(&mut self) {
        // {{{
        if self.accept(&vec![Symbol::Ident]).is_none() {
            // not a label
            return;
        }
        let macro_name = String::from(self.lexer.ident.clone());
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
                        macro_name, macro_value, self.lexer.position));
            }

            self.next_symbol();

        } else { // implicit file offset
            if self.stage == Stage::Label {
                //let macro_value = PC;
                //self.label_list.insert(macro_name.clone(), MacroElement::new(
                //        macro_name, macro_value, self.lexer.position));
                // add label declaration
            }
        }

        // }}}
    }

    fn stmt_opcode(&mut self) {
        // {{{
        let opcode_list = vec![
            Symbol::MacroORG,   Symbol::MacroEND,   
            Symbol::MacroDB,    Symbol::MacroDW,    Symbol::MacroDS,    
            Symbol::MacroPC,    
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
                if self.expect(&vec![Symbol::NumberWord], 
                                InternalErrorCode::SyntaxIntermediateWord) {
                    return;
                }
            }
            Symbol::MacroEND => {
                if !self.accept(&vec![Symbol::NumberWord]).is_none() {
                    self.next_symbol();
                    return
                }
            }
            Symbol::MacroDB => {
                if !self.expect(&vec![Symbol::NumberByte, Symbol::StringASCII],
                               InternalErrorCode::SyntaxDataByte) {
                    return;
                }

                while !self.accept(&vec![Symbol::Comma]).is_none() {
                    self.next_symbol();
                    if !self.expect(&vec![Symbol::NumberByte, Symbol::StringASCII],
                                   InternalErrorCode::SyntaxDataByte) {
                        return;
                    }
                }
            }
            Symbol::MacroDW => {
                if !self.expect(&vec![Symbol::NumberWord],
                                InternalErrorCode::SyntaxDataWord) {
                    return;
                }

                while !self.accept(&vec![Symbol::Comma]).is_none() {
                    self.next_symbol();
                    if !self.expect(&vec![Symbol::NumberWord],
                                   InternalErrorCode::SyntaxDataWord) {
                        return;
                    }
                }
            }
            Symbol::MacroDS => {
                if !self.expect(&vec![Symbol::NumberWord],
                                InternalErrorCode::SyntaxIntermediateWord) {
                    return;
                }
            }

            /* stage 3 */
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
                if !self.expect(&reg16_alu, 
                                InternalErrorCode::SyntaxRegisterPair) {
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
                    dbg!(&self.lexer.number);
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

    fn parse_step(&mut self, stage: Stage) {
        self.stage = stage;

        self.next_symbol();
        while self.accept(&vec![Symbol::EOF]).is_none() {

            self.stmt_label_definition();
            self.stmt_opcode();

            if !self.accept(&vec![Symbol::Comment]).is_none() {
                self.next_symbol();
            }

            self.expect(&vec![Symbol::Newline], InternalErrorCode::SyntaxNewline);
        }
    }

    pub fn parse(&mut self) {
        // get preprocessor set/equ macros
        self.parse_step(Stage::Preprocessor);

        // get position dependant labels
        self.parse_step(Stage::Label);

        // validate symbols
        //self.parse_step(Stage::Opcode);
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
        self.parser.lexer.read_ch();
        self.parser.parse();
    }
}

// vim: fdm=marker
// end of file
