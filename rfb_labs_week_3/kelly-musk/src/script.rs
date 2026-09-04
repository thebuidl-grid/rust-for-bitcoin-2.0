//! Minimal Script tooling: turn a scriptPubKey/scriptSig into a readable `asm`
//! string and classify the common output templates.
//!
//! The `asm` rendering is a faithful disassembly (opcodes by name, data pushes as
//! hex). It is intentionally *not* byte-identical to Bitcoin Core's numeric
//! rendering of tiny pushes; the hex form is less surprising for learning.

/// A single decoded script element.
enum Op<'a> {
    /// A data push: the opcode byte plus the bytes it pushed.
    Push { data: &'a [u8] },
    /// A non-push opcode.
    Code { byte: u8 },
    /// A push opcode whose declared length ran past the end of the script.
    Truncated,
}

fn opcode_name(byte: u8) -> String {
    let name = match byte {
        0x00 => "OP_0",
        0x4c => "OP_PUSHDATA1",
        0x4d => "OP_PUSHDATA2",
        0x4e => "OP_PUSHDATA4",
        0x4f => "OP_1NEGATE",
        0x50 => "OP_RESERVED",
        0x51 => "OP_1",
        0x52 => "OP_2",
        0x53 => "OP_3",
        0x54 => "OP_4",
        0x55 => "OP_5",
        0x56 => "OP_6",
        0x57 => "OP_7",
        0x58 => "OP_8",
        0x59 => "OP_9",
        0x5a => "OP_10",
        0x5b => "OP_11",
        0x5c => "OP_12",
        0x5d => "OP_13",
        0x5e => "OP_14",
        0x5f => "OP_15",
        0x60 => "OP_16",
        0x61 => "OP_NOP",
        0x62 => "OP_VER",
        0x63 => "OP_IF",
        0x64 => "OP_NOTIF",
        0x65 => "OP_VERIF",
        0x66 => "OP_VERNOTIF",
        0x67 => "OP_ELSE",
        0x68 => "OP_ENDIF",
        0x69 => "OP_VERIFY",
        0x6a => "OP_RETURN",
        0x6b => "OP_TOALTSTACK",
        0x6c => "OP_FROMALTSTACK",
        0x6d => "OP_2DROP",
        0x6e => "OP_2DUP",
        0x6f => "OP_3DUP",
        0x70 => "OP_2OVER",
        0x71 => "OP_2ROT",
        0x72 => "OP_2SWAP",
        0x73 => "OP_IFDUP",
        0x74 => "OP_DEPTH",
        0x75 => "OP_DROP",
        0x76 => "OP_DUP",
        0x77 => "OP_NIP",
        0x78 => "OP_OVER",
        0x79 => "OP_PICK",
        0x7a => "OP_ROLL",
        0x7b => "OP_ROT",
        0x7c => "OP_SWAP",
        0x7d => "OP_TUCK",
        0x7e => "OP_CAT",
        0x7f => "OP_SUBSTR",
        0x80 => "OP_LEFT",
        0x81 => "OP_RIGHT",
        0x82 => "OP_SIZE",
        0x83 => "OP_INVERT",
        0x84 => "OP_AND",
        0x85 => "OP_OR",
        0x86 => "OP_XOR",
        0x87 => "OP_EQUAL",
        0x88 => "OP_EQUALVERIFY",
        0x89 => "OP_RESERVED1",
        0x8a => "OP_RESERVED2",
        0x8b => "OP_1ADD",
        0x8c => "OP_1SUB",
        0x8d => "OP_2MUL",
        0x8e => "OP_2DIV",
        0x8f => "OP_NEGATE",
        0x90 => "OP_ABS",
        0x91 => "OP_NOT",
        0x92 => "OP_0NOTEQUAL",
        0x93 => "OP_ADD",
        0x94 => "OP_SUB",
        0x95 => "OP_MUL",
        0x96 => "OP_DIV",
        0x97 => "OP_MOD",
        0x98 => "OP_LSHIFT",
        0x99 => "OP_RSHIFT",
        0x9a => "OP_BOOLAND",
        0x9b => "OP_BOOLOR",
        0x9c => "OP_NUMEQUAL",
        0x9d => "OP_NUMEQUALVERIFY",
        0x9e => "OP_NUMNOTEQUAL",
        0x9f => "OP_LESSTHAN",
        0xa0 => "OP_GREATERTHAN",
        0xa1 => "OP_LESSTHANOREQUAL",
        0xa2 => "OP_GREATERTHANOREQUAL",
        0xa3 => "OP_MIN",
        0xa4 => "OP_MAX",
        0xa5 => "OP_WITHIN",
        0xa6 => "OP_RIPEMD160",
        0xa7 => "OP_SHA1",
        0xa8 => "OP_SHA256",
        0xa9 => "OP_HASH160",
        0xaa => "OP_HASH256",
        0xab => "OP_CODESEPARATOR",
        0xac => "OP_CHECKSIG",
        0xad => "OP_CHECKSIGVERIFY",
        0xae => "OP_CHECKMULTISIG",
        0xaf => "OP_CHECKMULTISIGVERIFY",
        0xb0 => "OP_NOP1",
        0xb1 => "OP_CHECKLOCKTIMEVERIFY",
        0xb2 => "OP_CHECKSEQUENCEVERIFY",
        0xb3 => "OP_NOP4",
        0xb4 => "OP_NOP5",
        0xb5 => "OP_NOP6",
        0xb6 => "OP_NOP7",
        0xb7 => "OP_NOP8",
        0xb8 => "OP_NOP9",
        0xb9 => "OP_NOP10",
        0xba => "OP_CHECKSIGADD",
        0xff => "OP_INVALIDOPCODE",
        _ => return format!("OP_UNKNOWN({byte:#04x})"),
    };
    name.to_string()
}

/// Walk a script into a sequence of ops. Unknown/return opcodes are still emitted;
/// a push that claims more bytes than remain yields a single `Truncated`.
fn scan(script: &[u8]) -> Vec<Op<'_>> {
    let mut ops = Vec::new();
    let mut i = 0;
    while i < script.len() {
        let opcode = script[i];
        i += 1;
        let push_len = match opcode {
            0x01..=0x4b => opcode as usize,
            0x4c => {
                if i >= script.len() {
                    ops.push(Op::Truncated);
                    break;
                }
                let n = script[i] as usize;
                i += 1;
                n
            }
            0x4d => {
                if i + 2 > script.len() {
                    ops.push(Op::Truncated);
                    break;
                }
                let n = u16::from_le_bytes([script[i], script[i + 1]]) as usize;
                i += 2;
                n
            }
            0x4e => {
                if i + 4 > script.len() {
                    ops.push(Op::Truncated);
                    break;
                }
                let n = u32::from_le_bytes([script[i], script[i + 1], script[i + 2], script[i + 3]])
                    as usize;
                i += 4;
                n
            }
            _ => {
                ops.push(Op::Code { byte: opcode });
                continue;
            }
        };

        if i + push_len > script.len() {
            ops.push(Op::Truncated);
            break;
        }
        ops.push(Op::Push {
            data: &script[i..i + push_len],
        });
        i += push_len;
    }
    ops
}

/// Render a script as space-separated `asm`.
pub fn to_asm(script: &[u8]) -> String {
    if script.is_empty() {
        return String::new();
    }
    let mut parts = Vec::new();
    for op in scan(script) {
        match op {
            Op::Push { data: &[] } => parts.push("OP_0".to_string()),
            Op::Push { data } => parts.push(hex::encode(data)),
            Op::Code { byte } => parts.push(opcode_name(byte)),
            Op::Truncated => parts.push("[truncated]".to_string()),
        }
    }
    parts.join(" ")
}

/// Classify a scriptPubKey using Bitcoin Core's output-type names.
pub fn classify(script: &[u8]) -> &'static str {
    let s = script;

    // P2PK: <pubkey> OP_CHECKSIG
    if (s.len() == 35 && s[0] == 33 && s[34] == 0xac)
        || (s.len() == 67 && s[0] == 65 && s[66] == 0xac)
    {
        return "pubkey";
    }

    // P2PKH: OP_DUP OP_HASH160 <20> OP_EQUALVERIFY OP_CHECKSIG
    if s.len() == 25
        && s[0] == 0x76
        && s[1] == 0xa9
        && s[2] == 0x14
        && s[23] == 0x88
        && s[24] == 0xac
    {
        return "pubkeyhash";
    }

    // P2SH: OP_HASH160 <20> OP_EQUAL
    if s.len() == 23 && s[0] == 0xa9 && s[1] == 0x14 && s[22] == 0x87 {
        return "scripthash";
    }

    // Witness programs: <version opcode> <2..40 byte push>, whole script is program+2.
    if s.len() >= 4 && s.len() <= 42 {
        let version_ok = s[0] == 0x00 || (0x51..=0x60).contains(&s[0]);
        let push_len = s[1] as usize;
        if version_ok && s[1] >= 0x02 && s[1] <= 0x28 && s.len() == push_len + 2 {
            return match (s[0], push_len) {
                (0x00, 20) => "witness_v0_keyhash",
                (0x00, 32) => "witness_v0_scripthash",
                (0x00, _) => "nonstandard",
                (0x51, 32) => "witness_v1_taproot",
                _ => "witness_unknown",
            };
        }
    }

    // OP_RETURN ...
    if s.first() == Some(&0x6a) {
        return "nulldata";
    }

    if is_bare_multisig(s) {
        return "multisig";
    }

    "nonstandard"
}

/// Bare (non-P2SH) multisig: OP_m <pub>*n OP_n OP_CHECKMULTISIG, 1 <= m <= n <= 20.
fn is_bare_multisig(s: &[u8]) -> bool {
    if s.len() < 4 || *s.last().unwrap() != 0xae {
        return false;
    }
    let small_int = |b: u8| -> Option<u8> {
        if (0x51..=0x60).contains(&b) {
            Some(b - 0x50)
        } else {
            None
        }
    };
    let Some(m) = small_int(s[0]) else {
        return false;
    };
    let Some(n) = small_int(s[s.len() - 2]) else {
        return false;
    };
    if m == 0 || m > n || n > 20 {
        return false;
    }

    let mut i = 1;
    let mut keys = 0_u8;
    while i < s.len() - 2 {
        let len = match s[i] {
            33 | 65 => s[i] as usize,
            _ => return false,
        };
        i += 1 + len;
        keys += 1;
    }
    i == s.len() - 2 && keys == n
}
