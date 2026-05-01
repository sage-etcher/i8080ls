
use crate::data_types::FxDashMap;
use crate::code_elements::*;

#[derive(Debug)]
enum Symbol {
    Unknown,
    RegA,
    RegB,
    RegC,
    RegD,
    RegE,
    RegF,
    RegH,
    RegL,
    RegM,
    RegPairSP,
    RegPairPSW,
    RegPairBC,
    RegPairDE,
    RegPairHL,
    Number,
    Comma,
    Colon,
    Ident,
    Newline,
    Comment,
    EOF,
    OpcodeACI,
    OpcodeADC,
    OpcodeADD,
    OpcodeADI,
    OpcodeANA,
    OpcodeANI,
    OpcodeCALL,
    OpcodeCC,
    OpcodeCM,
    OpcodeCMA,
    OpcodeCMC,
    OpcodeCMP,
    OpcodeCNC,
    OpcodeCNZ,
    OpcodeCP,
    OpcodeCPE,
    OpcodeCPI,
    OpcodeCPO,
    OpcodeCZ,
    OpcodeDAA,
    OpcodeDAD,
    OpcodeDCR,
    OpcodeDCX,
    OpcodeDI,
    OpcodeEI,
    OpcodeHLT,
    OpcodeIN,
    OpcodeINR,
    OpcodeINX,
    OpcodeJC,
    OpcodeJM,
    OpcodeJMP,
    OpcodeJNC,
    OpcodeJNZ,
    OpcodeJP,
    OpcodeJPE,
    OpcodeJPO,
    OpcodeJZ,
    OpcodeLDA,
    OpcodeLDAX,
    OpcodeLHLD,
    OpcodeLXI,
    OpcodeMOV,
    OpcodeMVI,
    OpcodeNOP,
    OpcodeORA,
    OpcodeORI,
    OpcodeOUT,
    OpcodePCHL,
    OpcodePOP,
    OpcodePUSH,
    OpcodeRAL,
    OpcodeRAR,
    OpcodeRC,
    OpcodeRET,
    OpcodeRLC,
    OpcodeRM,
    OpcodeRNC,
    OpcodeRNZ,
    OpcodeRP,
    OpcodeRPE,
    OpcodeRPO,
    OpcodeRRC,
    OpcodeRST,
    OpcodeRZ,
    OpcodeSBB,
    OpcodeSBI,
    OpcodeSHLD,
    OpcodeSPHL,
    OpcodeSTA,
    OpcodeSTAX,
    OpcodeSTC,
    OpcodeSUB,
    OpcodeSUI,
    OpcodeXCHG,
    OpcodeXRA,
    OpcodeXRI,
    OpcodeXTHL,
}

#[derive(Debug)]
struct Lexer {
    file_content: String,
    index: usize,
    ch: Option<char>,
    ch_lower: Option<char>,

    line:   u32,
    column: u32,

    kw_sym: FxDashMap<String, Vec<Symbol>>,
    number: u32,
    ident: String,
}

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

impl Symbol {
    pub fn get_keywords() -> FxDashMap<String, Vec<Symbol>> {
        let kw_map: FxDashMap<String, Vec<Symbol>> = FxDashMap::default();

        kw_map.insert(String::from("a"),    vec![Symbol::RegA]);
        kw_map.insert(String::from("b"),    vec![Symbol::RegB, Symbol::RegPairBC]);
        kw_map.insert(String::from("c"),    vec![Symbol::RegC]);
        kw_map.insert(String::from("d"),    vec![Symbol::RegD, Symbol::RegPairDE]);
        kw_map.insert(String::from("e"),    vec![Symbol::RegE]);
        kw_map.insert(String::from("h"),    vec![Symbol::RegH, Symbol::RegPairHL]);
        kw_map.insert(String::from("l"),    vec![Symbol::RegL]);
        kw_map.insert(String::from("m"),    vec![Symbol::RegM]);
        kw_map.insert(String::from("sp"),   vec![Symbol::RegPairSP]);
        kw_map.insert(String::from("psw"),  vec![Symbol::RegPairPSW]);

        kw_map.insert(String::from("aci"),  vec![Symbol::OpcodeACI]);
        kw_map.insert(String::from("adc"),  vec![Symbol::OpcodeADC]);
        kw_map.insert(String::from("add"),  vec![Symbol::OpcodeADD]);
        kw_map.insert(String::from("adi"),  vec![Symbol::OpcodeADI]);
        kw_map.insert(String::from("ana"),  vec![Symbol::OpcodeANA]);
        kw_map.insert(String::from("ani"),  vec![Symbol::OpcodeANI]);
        kw_map.insert(String::from("call"), vec![Symbol::OpcodeCALL]);
        kw_map.insert(String::from("cc"),   vec![Symbol::OpcodeCC]);
        kw_map.insert(String::from("cm"),   vec![Symbol::OpcodeCM]);
        kw_map.insert(String::from("cma"),  vec![Symbol::OpcodeCMA]);
        kw_map.insert(String::from("cmc"),  vec![Symbol::OpcodeCMC]);
        kw_map.insert(String::from("cmp"),  vec![Symbol::OpcodeCMP]);
        kw_map.insert(String::from("cnc"),  vec![Symbol::OpcodeCNC]);
        kw_map.insert(String::from("cnz"),  vec![Symbol::OpcodeCNZ]);
        kw_map.insert(String::from("cp"),   vec![Symbol::OpcodeCP]);
        kw_map.insert(String::from("cpe"),  vec![Symbol::OpcodeCPE]);
        kw_map.insert(String::from("cpi"),  vec![Symbol::OpcodeCPI]);
        kw_map.insert(String::from("cpo"),  vec![Symbol::OpcodeCPO]);
        kw_map.insert(String::from("cz"),   vec![Symbol::OpcodeCZ]);
        kw_map.insert(String::from("daa"),  vec![Symbol::OpcodeDAA]);
        kw_map.insert(String::from("dad"),  vec![Symbol::OpcodeDAD]);
        kw_map.insert(String::from("dcr"),  vec![Symbol::OpcodeDCR]);
        kw_map.insert(String::from("dcx"),  vec![Symbol::OpcodeDCX]);
        kw_map.insert(String::from("di"),   vec![Symbol::OpcodeDI]);
        kw_map.insert(String::from("ei"),   vec![Symbol::OpcodeEI]);
        kw_map.insert(String::from("hlt"),  vec![Symbol::OpcodeHLT]);
        kw_map.insert(String::from("in"),   vec![Symbol::OpcodeIN]);
        kw_map.insert(String::from("inr"),  vec![Symbol::OpcodeINR]);
        kw_map.insert(String::from("inx"),  vec![Symbol::OpcodeINX]);
        kw_map.insert(String::from("jc"),   vec![Symbol::OpcodeJC]);
        kw_map.insert(String::from("jm"),   vec![Symbol::OpcodeJM]);
        kw_map.insert(String::from("jmp"),  vec![Symbol::OpcodeJMP]);
        kw_map.insert(String::from("jnc"),  vec![Symbol::OpcodeJNC]);
        kw_map.insert(String::from("jnz"),  vec![Symbol::OpcodeJNZ]);
        kw_map.insert(String::from("jp"),   vec![Symbol::OpcodeJP]);
        kw_map.insert(String::from("jpe"),  vec![Symbol::OpcodeJPE]);
        kw_map.insert(String::from("jpo"),  vec![Symbol::OpcodeJPO]);
        kw_map.insert(String::from("jz"),   vec![Symbol::OpcodeJZ]);
        kw_map.insert(String::from("lda"),  vec![Symbol::OpcodeLDA]);
        kw_map.insert(String::from("ldax"), vec![Symbol::OpcodeLDAX]);
        kw_map.insert(String::from("lhld"), vec![Symbol::OpcodeLHLD]);
        kw_map.insert(String::from("lxi"),  vec![Symbol::OpcodeLXI]);
        kw_map.insert(String::from("mov"),  vec![Symbol::OpcodeMOV]);
        kw_map.insert(String::from("mvi"),  vec![Symbol::OpcodeMVI]);
        kw_map.insert(String::from("nop"),  vec![Symbol::OpcodeNOP]);
        kw_map.insert(String::from("ora"),  vec![Symbol::OpcodeORA]);
        kw_map.insert(String::from("ori"),  vec![Symbol::OpcodeORI]);
        kw_map.insert(String::from("out"),  vec![Symbol::OpcodeOUT]);
        kw_map.insert(String::from("pchl"), vec![Symbol::OpcodePCHL]);
        kw_map.insert(String::from("pop"),  vec![Symbol::OpcodePOP]);
        kw_map.insert(String::from("push"), vec![Symbol::OpcodePUSH]);
        kw_map.insert(String::from("ral"),  vec![Symbol::OpcodeRAL]);
        kw_map.insert(String::from("rar"),  vec![Symbol::OpcodeRAR]);
        kw_map.insert(String::from("rc"),   vec![Symbol::OpcodeRC]);
        kw_map.insert(String::from("ret"),  vec![Symbol::OpcodeRET]);
        kw_map.insert(String::from("rlc"),  vec![Symbol::OpcodeRLC]);
        kw_map.insert(String::from("rm"),   vec![Symbol::OpcodeRM]);
        kw_map.insert(String::from("rnc"),  vec![Symbol::OpcodeRNC]);
        kw_map.insert(String::from("rnz"),  vec![Symbol::OpcodeRNZ]);
        kw_map.insert(String::from("rp"),   vec![Symbol::OpcodeRP]);
        kw_map.insert(String::from("rpe"),  vec![Symbol::OpcodeRPE]);
        kw_map.insert(String::from("rpo"),  vec![Symbol::OpcodeRPO]);
        kw_map.insert(String::from("rrc"),  vec![Symbol::OpcodeRRC]);
        kw_map.insert(String::from("rst"),  vec![Symbol::OpcodeRST]);
        kw_map.insert(String::from("rz"),   vec![Symbol::OpcodeRZ]);
        kw_map.insert(String::from("sbb"),  vec![Symbol::OpcodeSBB]);
        kw_map.insert(String::from("sbi"),  vec![Symbol::OpcodeSBI]);
        kw_map.insert(String::from("shld"), vec![Symbol::OpcodeSHLD]);
        kw_map.insert(String::from("sphl"), vec![Symbol::OpcodeSPHL]);
        kw_map.insert(String::from("sta"),  vec![Symbol::OpcodeSTA]);
        kw_map.insert(String::from("stax"), vec![Symbol::OpcodeSTAX]);
        kw_map.insert(String::from("stc"),  vec![Symbol::OpcodeSTC]);
        kw_map.insert(String::from("sub"),  vec![Symbol::OpcodeSUB]);
        kw_map.insert(String::from("sui"),  vec![Symbol::OpcodeSUI]);
        kw_map.insert(String::from("xchg"), vec![Symbol::OpcodeXCHG]);
        kw_map.insert(String::from("xra"),  vec![Symbol::OpcodeXRA]);
        kw_map.insert(String::from("xri"),  vec![Symbol::OpcodeXRI]);
        kw_map.insert(String::from("xthl"), vec![Symbol::OpcodeXTHL]);

        return kw_map;
    }
}

impl Lexer {
    pub fn new(file_content: &str) -> Self {
        Self {
            file_content: String::from(file_content),
            index:0,
            ch: None,
            ch_lower: None,

            line:   0,
            column: 0,

            kw_sym: Symbol::get_keywords(),
            number: 0,
            ident:  String::new(),


        }
    }

    fn read_ch(&mut self) -> Option<char> {
        if self.index >= self.file_content.len()
        {
            self.ch = None;
            self.ch_lower = None;
            return None;
        }

        let ch = self.file_content.chars().nth(self.index).unwrap();
        self.index += 1;

        if ch == '\n' {
            self.line += 1;
            self.column = 0;
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
        let mut base = 0;

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
                            return vec![Symbol::UNKNWON];
                        }
                    }
                }
            }
        }

        /* convert iterable to number */
        let mut number: u32 = 0;
        for xdigit in xdigit_arr.into_iter() {
            number *= base + xdigit.;

            if xdigit >= 'a' {
                number += xdigit - 'a' + 10;
            } else {
                number += xdigit - '0';
            }
        }

        self.number = number;

        return vec![Symbol::NUMBER];
    }

    fn parse_alpha(&mut self) -> Vec<Symbol> {

        /* collect ident */
        let mut ident_arr: Vec<char> = Vec::new();

        ident_arr.insert(self.ch_lower);
        loop {
            match self.read_ch() {
                None      => break,
                Some('$') => continue,
                Some('_') => continue,
            }

            if !self.ch_lower.unwrap().is_alphanumeric() {
                break;
            }

            ident_arr.insert(self.ch_lower.unwrap().to_lowercase());
        }
        let ident: String = ident_arr.into_iter().collect();

        /* check for keywords */
        let kw_sym: Option<Vec<Symbol>> = self.keywords.get(ident);
        if kw_sym != None {
            return kw_sym.unwrap();
        }

        /* assume to be ident */
        self.ident = ident;

        return vec![Symbol::IDENT];
    }

    fn get_symbol(&mut self) -> Vec<Symbol> {
        while self.read_ch().unwrap().is_ascii_whitespace() {}

        // symbols
        match self.ch {
            None       => { self.read_ch(); return vec![Symbol::EOF]; }
            Some('\n') => { self.read_ch(); return vec![Symbol::NEWLINE]; }
            Some(',')  => { self.read_ch(); return vec![Symbol::COMMA]; }
            Some(':')  => { self.read_ch(); return vec![Symbol::COLON]; }
        }

        // number
        if self.ch.unwrap().is_digit(10) {
            return self.parse_number();
        }

        // ident, register, or opcode
        if self.ch.unwrap().is_alphabetic() {
            return self.parse_alpha();
        }

        return vec![Symbol::UNKNOWN];
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
