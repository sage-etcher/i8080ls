
use crate::data_types::FxDashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Symbol {
    Unknown,
    MacroORG,
    MacroEND,
    MacroEQU,
    MacroSET,
    MacroDB,
    MacroDW,
    MacroDS,
    MacroPC,
    MacroAdd,
    MacroSub,
    MacroMult,
    MacroDiv,
    MacroMod,
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
    NumberByte,
    NumberWord,
    NumberOverflow,
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

impl Symbol {
    pub fn get_keywords() -> FxDashMap<String, Vec<Symbol>> {
        let kw_map: FxDashMap<String, Vec<Symbol>> = FxDashMap::default();

        kw_map.insert(String::from("org"),  vec![Symbol::MacroORG]);
        kw_map.insert(String::from("end"),  vec![Symbol::MacroEND]);
        kw_map.insert(String::from("equ"),  vec![Symbol::MacroEQU]);
        kw_map.insert(String::from("set"),  vec![Symbol::MacroSET]);
        kw_map.insert(String::from("db"),   vec![Symbol::MacroDB]);
        kw_map.insert(String::from("dw"),   vec![Symbol::MacroDW]);
        kw_map.insert(String::from("ds"),   vec![Symbol::MacroDS]);

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

// end of file
