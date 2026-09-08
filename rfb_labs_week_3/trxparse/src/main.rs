use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

// ┌──────────────────────────────┐
// │ Version          4 bytes     │
// ├──────────────────────────────┤
// │ Marker           1 byte      │
// │ Flag             1 byte      │
// ├──────────────────────────────┤
// │ Input count      VarInt      │
// │ Inputs           Variable    │
// ├──────────────────────────────┤
// │ Output count     VarInt      │
// │ Outputs          Variable    │
// ├──────────────────────────────┤
// │ Witness          Variable    │
// ├──────────────────────────────┤
// │ Locktime         4 bytes  ←  │
// └──────────────────────────────┘


//  outputs: [
//         TxOut1 {
//             value: 100,
//             script_pubkey: ...
//         },
//         TxOut2 {
//             value: 100,
//             script_pubkey: ...
//         },
//  ]

// read_u64: read the next 8 bytes
// read_u32: read the next 4 bytes
// read_u16: read the next 2 bytes
// read_u8: read the next 1 byte 


// This function is a fundamental building block in a Bitcoin parser because CompactSize integers are used throughout the protocol to 
// encode the number of transaction inputs, outputs, script lengths, witness element counts, and many other variable-length fields.

fn read_varint(r: &mut Cursor<Vec<u8>>) -> u64 {
    let n = r.read_u8().unwrap();
    match n {
        0x00..=0xfc => n as u64,
        0xfd => r.read_u16::<LittleEndian>().unwrap() as u64,
        0xfe => r.read_u32::<LittleEndian>().unwrap() as u64,
        _ => r.read_u64::<LittleEndian>().unwrap(),
    }
}



fn read_bytes(r: &mut Cursor<Vec<u8>>, n: usize) -> Vec<u8> {
    let mut b = vec![0; n]; // creates a Vec<u8> with 32 elements, where each element is one byte (u8).
    r.read_exact(&mut b).unwrap(); // read exactly 32 bytes from the last position of the cursor, 
    // The cursor moves forward by 32 bytes.
    b
}

fn hex(v: &[u8]) -> String {
    v.iter().map(|b| format!("{:02x}", b)).collect()
}

fn main() {
    let raw = "020000000001018fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc8210100000000ffffffff02fe10010000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b02730000000000001600149831122b93d21715c70db626ccc844d3c21f968702483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000";
    // version: 02000000
  
   let bytes = hex::decode(raw).unwrap();

   println!("bytes = {:?}", bytes);

     let mut r = Cursor::new(bytes);

       // When we read 4 bytes, the cursor automatically moves forward:
    // read_u32 : Read the next 4 bytes and interpret them as an unsigned 32-bit integer.
    // LittleEndian:  With little-endian, the least significant byte comes first.
    // Bitcoin  transaction version is a 4-byte little-endian integer.
    // we have 02 00 00 00 little-endian  turn it into 00 00 00 02: 0x00000002

     let version = r.read_u32::<LittleEndian>().unwrap();
     println!("Version: {}", version);

    // // read_u8() consumes exactly one byte : 00
    // // u8 means an unsigned 8-bit integer : 8 bits = 1 byte

    let marker = r.read_u8().unwrap();
    let flag = r.read_u8().unwrap();
    //   // marker and flag are each one byte.
    // // marker and flag tell us that this is SegWit transaction, and witness data is present.

    println!("SegWit marker={} flag={}", marker, flag);

    let in_count = read_varint(&mut r);

     println!("Inputs: {}", in_count);

    for i in 0..in_count {
        println!("Input {}", i);
         // 32-byte previous transaction ID from the input. (from the current position of the cursor)
        // From the current cursor position, read exactly 32 bytes
        // Bitcoin transaction input contains a 32-byte previous transaction hash (TXID).
        let prev = read_bytes(&mut r, 32);
          println!("  Prev TXID (LE): {}", hex(&prev));
        let vout = r.read_u32::<LittleEndian>().unwrap(); 
         println!("  Vout: {}", vout);
       let slen = read_varint(&mut r) as usize;
        println!("  script length: {}", slen);
        
        let script = read_bytes(&mut r,slen);
        println!("  ScriptSig: {}", hex(&script)); 
        let seq = r.read_u32::<LittleEndian>().unwrap();
        println!("  Sequence: {:08x}", seq);
    }

    let out_count = read_varint(&mut r);

    println!("Outputs: {}", out_count);
    for i in 0..out_count {
        println!("Output {}", i);
        let value = r.read_u64::<LittleEndian>().unwrap(); // Read the next 8 bytes and interpret them as a little-endian u64.
        println!("  Value: {} sats", value);
        let slen = read_varint(&mut r) as usize;
        println!("  script length: {}", slen);
        let script = read_bytes(&mut r,slen);
        println!("  ScriptPubKey: {}", hex(&script));
    }

       // each input has its own witness field,
    for i in 0..in_count {
        let items = read_varint(&mut r);
        println!("Witness for input {} ({} item(s))", i, items);
        for j in 0..items {
            let len = read_varint(&mut r) as usize;
            let item = read_bytes(&mut r,len);
            println!("  Item {}: {}", j, hex(&item));
        }
    }

    let locktime = r.read_u32::<LittleEndian>().unwrap();
    println!("Locktime: {}", locktime);


}