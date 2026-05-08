
use tower_lsp_server::ls_types::Range;


#[derive(Debug, Copy, Clone)]
pub enum InternalErrorCode {
    Okay,
    SyntaxGeneric,
    SyntaxUnknown,
    SyntaxUnknownLabel,
    SyntaxMissingColon,
    SyntaxMissingComma,
    SyntaxRegister,
    SyntaxRegisterPair,
    SyntaxRegisterPairPush,
    SyntaxRegisterPairBD,
    SyntaxIntermediateByte,
    SyntaxIntermediateWord,
    SyntaxIntermediateRST,
    SyntaxDataByte,
    SyntaxDataWord,
    SyntaxNewline,
    EndOffsetMismatch,
    OffsetNotSet,
    UnknownMacro,
    DuplicateIdent,
}

#[derive(Debug)]
pub struct InternalError {
    pub range: Range,
    pub errcode: InternalErrorCode,
}

impl InternalError {
    pub fn new(range: Range, errcode: InternalErrorCode) -> Self {
        Self {
            range,
            errcode,
        }
    }

    pub fn get_errstr(&self) -> String {
        let default_errcode = InternalErrorCode::SyntaxGeneric;
        let errstr_list: Vec<&str> = vec![
            "no error",
            "syntax error",
            "unknown symbol",
            "unknown label",
            "expected colon suffix",
            "expected comma seperator",
            "expected register: a, b, c, d, e, h, l, or m",
            "expected register pair: b, d, h, or sp",
            "expected register pair: psw, b, d, or h",
            "expected register pair: b or d",
            "expected intermediate byte: 0-255",
            "expected intermediate word: 0-65535",
            "expected intermediate value: 0-7",
            "expected intermediate string or byte: 0-255",
            "expected intermediate word: 0-65535",
            "expected newline",
            "end offset differs from previous org statement",
            "cannot updaate offset, no previous org statement",
            "unknown macro/label",
            "duplicate ident",
        ];

        if self.errcode as usize >= errstr_list.len() {
            return String::from(errstr_list[default_errcode as usize]);
        } else {
            return String::from(errstr_list[self.errcode as usize]);
        }
    }
}

// end of file
