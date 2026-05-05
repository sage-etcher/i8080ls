
use tower_lsp_server::ls_types::Range;

use crate::data_types::FxDashMap;
use crate::code_elements::*;
use crate::symbol::Symbol;
use crate::lexer::Lexer;

#[derive(Debug)]
pub struct Parser {
    lexer: Lexer,
    symbol: Vec<Symbol>,

    pub syntax_error: Vec<Range>,
}

#[derive(Debug)]
pub struct FileContext {
    pub parser: Parser,

    pub macros:  FxDashMap<String, CodeMacro>,
    pub opcodes: FxDashMap<String, CodeOpcode>,
    pub numbers: FxDashMap<String, CodeNumber>,
    pub symbols: FxDashMap<String, CodeSymbol>,
}


impl Parser {
    pub fn new(file_content: &str) -> Self {
        Self {
            lexer:  Lexer::new(file_content),
            symbol: Vec::default(),

            syntax_error: Vec::default(),
        }
    }

    fn add_error(&mut self) {
        self.syntax_error.push(self.lexer.position);
    }

    fn accept(&mut self, needles: &Vec<Symbol>) -> Option<Vec<Symbol>> {
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
    }

    fn expect(&mut self, needles: &Vec<Symbol>) -> bool {
        let matches = self.accept(&needles);
        if matches.is_none() {
            self.add_error();
        }
        self.next_symbol();
        return !matches.is_none();
    }

    fn next_symbol(&mut self) {
        self.symbol = self.lexer.get_symbol();
        //dbg!(&self.symbol);
    }

    fn stmt_label_definition(&mut self) {
        if self.accept(&vec![Symbol::Ident]).is_none() {
            // not a label
            return;
        }

        /* validate label definition */
        self.next_symbol();
        if !self.expect(&vec![Symbol::Colon]) {
            return;
        }

        // valid label definition
    }

    fn stmt_opcode(&mut self) {
        let opcode_list = vec![
            Symbol::MacroORG,  Symbol::OpcodeMVI, Symbol::OpcodeMOV,
            Symbol::OpcodeADD, Symbol::OpcodeJMP, Symbol::OpcodeRET,
        ];

        let reg8 = vec![
            Symbol::RegB, Symbol::RegC, Symbol::RegD, Symbol::RegE,
            Symbol::RegH, Symbol::RegL, Symbol::RegM, Symbol::RegA,
        ];

        let reg16_push = vec![
            Symbol::RegPairPSW, Symbol::RegPairBC, 
            Symbol::RegPairDE,  Symbol::RegPairHL,
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
                if !self.expect(&vec![Symbol::NumberWord]) {
                    return;
                }
            }
            Symbol::OpcodeLXI => {
                if !(self.expect(&reg16_alu)
                     && self.expect(&vec![Symbol::Comma])
                     && self.expect(&vec![Symbol::NumberWord]))
                {
                    return;
                }
            }
            Symbol::OpcodeINX | Symbol::OpcodeDCX | Symbol::OpcodeDAD => {
                if !self.expect(&reg16_alu) {
                    return;
                }
            }
            Symbol::OpcodeSTAX | Symbol::OpcodeLDAX => {
                if !self.expect(&vec![Symbol::RegPairBC, Symbol::RegPairDE]) {
                    return;
                }
            }
            Symbol::OpcodePUSH | Symbol::OpcodePop => {
                if !self.expect(reg16_push) {
                    return;
                }
            }
            Symbol::OpcodeOUT  | Symbol::OpcodeIN   |
            Symbol::OpcodeADI  | Symbol::OpcodeACI  |
            Symbol::OpcodeSUI  | Symbol::OpcodeSBI  |
            Symbol::OpcodeANI  | Symbol::OpcodeXRI  |
            Symbol::OpcodeORI  | Symbol::OpcodeCPI => {
                if !self.expect(&vec![Symbol::NumberByte]) {
                    return;
                }
            }
            Symbol::OpcodeRST => {
                if self.accept(&vec!{Symbol::NumberByte]).is_none()
                   || self.lexer.number > 7
                {
                    self.next_symbol();
                    self.add_error();
                    return;
                }
            }
            Symbol::OpcodeMVI = {
                if !(self.expect(&reg8)
                     && self.expect(&vec![Symbol::Comma])
                     && self.expect(&vec![Symbol::NumberByte]))
                {
                    return;
                }
            }
            Symbol::OpcodeMOV => {
                if !(self.expect(&reg8)
                     && self.expect(&vec![Symbol::Comma])
                     && self.expect(&reg8))
                {
                    return;
                }
            }
            Symbol::OpcodeINR  | Symbol::OpcodeDCR  |
            Symbol::OpcodeADD  | Symbol::OpcodeADC  |
            Symbol::OpcodeSUB  | Symbol::OpcodeSBB  |
            Symbol::OpcodeANA  | Symbol::OpcodeXRA  |
            Symbol::OpcodeORA  | Symbol::OpcodeCMP => {
                if !self.expect(&reg8) {
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
                if !self.expect(&vec![Symbol::NumberWord]) {
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
                self.add_error();
                return;
            }
        }
    }

    fn stmt_line(&mut self) {
        self.stmt_label_definition();
        self.stmt_opcode();

        if !self.accept(&vec![Symbol::Comment]).is_none() {
            self.next_symbol();
        }

        self.expect(&vec![Symbol::Newline]);
    }


    pub fn parse(&mut self) {
        self.next_symbol();

        while self.accept(&vec![Symbol::EOF]).is_none() {
            self.stmt_line();
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
        self.parser.lexer.read_ch();
        self.parser.parse();

        // add debugging things
        self.add_debug_macros();
    }
}

// end of file
