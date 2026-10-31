//! Small tests of the bytecode format: encoding, decoding, the image form,
//! the trusted form, and the assembler's round trip (target R0T-6).

use crate::*;

const PROBE: &str = r#"
.function setup params=0 registers=8
.const k0 key "weakRef"
.const k1 key "registry"
.slot s0 store
.slot s1 store
        LoadNull
        StoreReg r0                           ; r0 = null, the prototype
        CallIntrinsic %ObjectCreate, r0:1
        StoreReg r1                           ; r1 = target
        CallIntrinsic %WeakRefCreate, r1:1
        StoreReg r2
        LoadInt #7
        StoreReg r3
        CallIntrinsic %CreateHostFunction, r3:1
        StoreReg r3
        CallIntrinsic %FinalizationRegistryCreate, r3:1
        StoreReg r3
        Copy r1, r4
        LoadInt #1
        StoreReg r5
        LoadUndefined
        StoreReg r6
        CallIntrinsic %FinalizationRegistryRegister, r3:4
        CallIntrinsic %ObjectCreate, r0:1
        StoreReg r7
        LoadReg r2
        SetNamed r7, k0, s0
        LoadReg r3
        SetNamed r7, k1, s1
        LoadReg r7
        Return
.end

.function churn params=1 registers=3
.const k0 key "weakRef"
.slot s0 unary
.slot s1 compare
.slot s2 loop
.slot s3 load
        LoadNull
        StoreReg r0
        LoadInt #0
        StoreReg r1                           ; i = 0
loop:   CallIntrinsic %ObjectCreate, r0:1     ; garbage: 4 canonical units each
        LoadReg r1
        Inc s0
        StoreReg r1
        LoadInt #10000
        Lt r1, s1                             ; i < 10000
        BranchFalse @done
        LoopBack @loop, s2
done:   GetNamed a0, k0, s3                   ; a0 = the holder
        StoreReg r2
        CallIntrinsic %WeakRefDeref, r2:1
        Return
.end
"#;

#[test]
fn the_opcode_table_follows_the_bytecode_note() {
    // Section 5's numbering: 0x01 to 0x31, in declaration order.
    assert_eq!(OP_COUNT, 49);
    assert_eq!(Op::LoadUndefined.byte(), 0x01);
    assert_eq!(Op::LoadInt.byte(), 0x05);
    assert_eq!(Op::Add.byte(), 0x10);
    assert_eq!(Op::LooseEq.byte(), 0x20);
    assert_eq!(Op::Jump.byte(), 0x26);
    assert_eq!(Op::LoopBack.byte(), 0x27);
    assert_eq!(Op::Return.byte(), 0x2C);
    assert_eq!(Op::GetNamed.byte(), 0x2D);
    assert_eq!(Op::CallIntrinsic.byte(), 0x31);
    for (i, op) in Op::ALL.iter().enumerate() {
        assert_eq!(usize::from(op.byte()), i + 1);
        assert_eq!(Op::from_byte(op.byte()), Some(*op));
        assert_eq!(Op::from_mnemonic(op.info().mnemonic), Some(*op));
    }
    for b in [INVALID, EXTENDED, WIDE2, WIDE4, 0x32, 0xFC] {
        assert_eq!(Op::from_byte(b), None);
    }
    assert_eq!(Intrinsic::ALL.len(), 9);
    assert_eq!(Intrinsic::FinalizationRegistryRegister.arity(), 4);
    assert!(
        Op::Return.info().terminator
            && Op::Jump.info().terminator
            && Op::LoopBack.info().terminator
    );
    assert!(!Op::BranchFalse.info().terminator && !Op::Add.info().terminator);
    assert!(Op::Add.info().may_call_user && !Op::StrictEq.info().may_call_user);
}

#[test]
fn decoding_reads_prefixes_and_signed_operands() {
    // LoadReg a0 (−14, signed byte 0xF2); Wide2 LoadInt #−300; Wide4
    // LoadInt #70000; then Return.
    let code = [
        0x07, 0xF2, WIDE2, 0x05, 0xD4, 0xFE, WIDE4, 0x05, 0x70, 0x11, 0x01, 0x00, 0x2C,
    ];
    let a = decode(&code, 0).unwrap();
    assert_eq!(
        (a.op, a.scale, a.reg(0), a.end),
        (Op::LoadReg, 1, Reg(-14), 2)
    );
    let b = decode(&code, 2).unwrap();
    assert_eq!(
        (b.op, b.scale, b.imm(0), b.start, b.end),
        (Op::LoadInt, 2, -300, 2, 6)
    );
    let c = decode(&code, 6).unwrap();
    assert_eq!(
        (c.op, c.scale, c.imm(0), c.end),
        (Op::LoadInt, 4, 70000, 12)
    );
    assert_eq!(decode(&code, 12).unwrap().op, Op::Return);
    assert_eq!(decode(&code, 13), Err(DecodeError::End));
    // Invalid, reserved, a prefix on a prefix or on an operandless opcode,
    // and truncation.
    assert_eq!(decode(&[0x00], 0), Err(DecodeError::Invalid(0)));
    assert_eq!(decode(&[0x99], 0), Err(DecodeError::Invalid(0x99)));
    assert_eq!(decode(&[EXTENDED, 0x01], 0), Err(DecodeError::Reserved));
    assert_eq!(
        decode(&[WIDE2, WIDE4, 0x05], 0),
        Err(DecodeError::BadPrefix)
    );
    assert_eq!(decode(&[WIDE2, 0x01], 0), Err(DecodeError::BadPrefix));
    assert_eq!(decode(&[WIDE2, 0x05, 0x00], 0), Err(DecodeError::Truncated));
    assert_eq!(decode(&[0x10, 0x00], 0), Err(DecodeError::Truncated));
}

#[test]
fn every_byte_string_decodes_or_errs_without_panicking() {
    let mut x = 0x1234_5678_9abc_def0u64;
    for _ in 0..2000 {
        let mut code = Vec::new();
        for _ in 0..16 {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            code.push(x.to_le_bytes()[0]);
        }
        let mut pc = 0;
        while let Ok(i) = decode(&code, pc) {
            assert!(i.end > pc && usize::try_from(i.end).unwrap() <= code.len());
            pc = i.end;
        }
    }
}

#[test]
fn the_probe_assembles_and_round_trips() {
    let units = assemble(PROBE).unwrap();
    assert_eq!(units.len(), 2);
    let (setup, churn) = (&units[0], &units[1]);
    assert_eq!(
        (setup.params, setup.registers, setup.slots.len()),
        (0, 8, 2)
    );
    assert_eq!(
        churn.constants,
        vec![Constant::Key("weakRef".encode_utf16().collect())]
    );
    for u in &units {
        let text = disassemble(u);
        let again = assemble(&text).unwrap_or_else(|e| panic!("{e}\n{text}"));
        assert_eq!(&again[0], u, "{text}");
        // The image form round trips too.
        let img = u.to_image();
        let v = UnitView::parse(&img).unwrap();
        assert_eq!(&v.to_unit(), u);
        assert_eq!(v.frame_charge(), 32 + 4 * u64::from(u.params + u.registers));
    }
    // `LoopBack @loop` is a backward distance; `BranchFalse @done` forward.
    let code = &churn.code;
    let mut pc = 0;
    let mut saw_back = false;
    while let Ok(i) = decode(code, pc) {
        if i.op == Op::LoopBack {
            saw_back = true;
            let t = decode(code, i.backward_target(0)).unwrap();
            assert_eq!(t.op, Op::CallIntrinsic);
        }
        if i.op == Op::BranchFalse {
            let t = decode(code, i.forward_target(0)).unwrap();
            assert_eq!(t.op, Op::GetNamed);
        }
        pc = i.end;
    }
    assert!(saw_back);
}

#[test]
fn jumps_widen_until_they_fit_and_forced_widths_are_kept() {
    // 200 three-byte instructions between a jump and its target need Wide2.
    let mut text = String::from(".function f params=0 registers=2\n.slot s0 binary\n Jump @end\n");
    for _ in 0..200 {
        text.push_str(" Add r0, s0\n");
    }
    text.push_str("end: Return\n.end\n");
    let u = &assemble(&text).unwrap()[0];
    assert_eq!(u.code[0], WIDE2);
    let j = decode(&u.code, 0).unwrap();
    assert_eq!(j.forward_target(0), 4 + 600);
    assert_eq!(&assemble(&disassemble(u)).unwrap()[0], u);
    // A forced `.x` is kept even though narrow fits, and round trips.
    let w =
        &assemble(".function g params=0 registers=1\n LoadReg.x r0\n Return\n.end\n").unwrap()[0];
    assert_eq!(&w.code[..6], &[WIDE4, 0x07, 0, 0, 0, 0]);
    assert!(disassemble(w).contains("LoadReg.x r0"));
    assert_eq!(&assemble(&disassemble(w)).unwrap()[0], w);
}

#[test]
fn the_assembler_rejects_what_the_verifier_will() {
    let bad = [
        (
            ".function f params=0 registers=1\n LoadReg r1\n Return\n.end",
            "outside the frame",
        ),
        (
            ".function f params=1 registers=1\n LoadReg a1\n Return\n.end",
            "outside the frame",
        ),
        (
            ".function f params=0 registers=1\n LoadInt #1073741824\n Return\n.end",
            "small-integer",
        ),
        (
            ".function f params=0 registers=1\n LoadConst k0\n Return\n.end",
            "not declared",
        ),
        (
            ".function f params=0 registers=1\n.const k0 key \"x\"\n LoadConst k0\n Return\n.end",
            "does not accept",
        ),
        (
            ".function f params=0 registers=1\n.slot s0 unary\n Add r0, s0\n Return\n.end",
            "needs a `binary` slot",
        ),
        (
            ".function f params=0 registers=2\n CallIntrinsic %ObjectCreate, r0:2\n Return\n.end",
            "takes 1 arguments",
        ),
        (
            ".function f params=0 registers=1\nl: LoadUndefined\n Jump @l\n.end",
            "not after",
        ),
        (
            ".function f params=0 registers=1\n.slot s0 loop\n LoopBack @l, s0\nl: Return\n.end",
            "after this instruction",
        ),
        (
            ".function f params=0 registers=1\n LoadUndefined\n.end",
            "ends in",
        ),
        (
            ".function f params=0 registers=1\n LoadUndefined.w\n Return\n.end",
            "no operands",
        ),
        (
            ".function f params=0 registers=1\n Frobnicate\n Return\n.end",
            "unknown mnemonic",
        ),
        (
            ".function f params=0 registers=1\n CallIntrinsic %Nope, r0:1\n Return\n.end",
            "unknown intrinsic",
        ),
        (
            ".function f params=0 registers=1\n Jump @nowhere\n.end",
            "undefined label",
        ),
        (
            ".function f params=0 registers=1\n CallIntrinsic %MathRandom, r99999:0\n Return\n.end",
            "empty register list",
        ),
    ];
    for (text, want) in bad {
        let e = assemble(text).unwrap_err();
        assert!(e.msg.contains(want), "{text}: {e}");
    }
}

#[test]
fn constants_keep_every_number_and_code_unit() {
    let text = ".function f params=0 registers=1\n\
        .const k0 number -0\n.const k1 number NaN\n.const k2 number -Infinity\n\
        .const k3 number 1e21\n.const k4 number 0.1\n.const k5 string \"a\\u0000\\ud800\\\"\\\\é😀\"\n\
        LoadConst k0\n Return\n.end";
    let u = &assemble(text).unwrap()[0];
    let Constant::Number(z) = u.constants[0] else {
        panic!()
    };
    assert!(z == 0.0 && z.is_sign_negative());
    assert!(matches!(u.constants[1], Constant::Number(n) if n.is_nan()));
    let want: Vec<u16> = vec![0x61, 0, 0xD800, 0x22, 0x5C, 0xE9, 0xD83D, 0xDE00];
    assert_eq!(u.constants[5], Constant::String(want));
    let again = &assemble(&disassemble(u)).unwrap()[0];
    assert_eq!(again.code, u.code);
    assert_eq!(again.constants.len(), u.constants.len());
    for (a, b) in again.constants.iter().zip(&u.constants) {
        match (a, b) {
            (Constant::Number(x), Constant::Number(y)) => assert_eq!(x.to_bits(), y.to_bits()),
            _ => assert_eq!(a, b),
        }
    }
}

#[test]
fn images_are_parsed_as_hostile_input() {
    let u = &assemble(PROBE).unwrap()[0];
    let img = u.to_image();
    assert!(UnitView::parse(&img).is_ok());
    assert_eq!(
        UnitView::parse(&img[..40]).unwrap_err(),
        ImageError::Truncated
    );
    let mut wrong = img.clone();
    wrong[4] ^= 1;
    assert_eq!(
        UnitView::parse(&wrong).unwrap_err(),
        ImageError::WrongFormat
    );
    // Every single-byte corruption of the header parses or errs cleanly.
    for at in 0..88 {
        for v in [0u8, 0xFF, 0x80] {
            let mut c = img.clone();
            c[at] = v;
            let _ = UnitView::parse(&c).map(|x| x.to_unit());
        }
    }
}

#[test]
fn units_seal_into_the_trusted_region() {
    let mem = std::sync::Arc::new(hg_heap::testing::TestMemory::default());
    let mut region = hg_heap::TrustedRegion::new(mem, 1 << 20).unwrap();
    let units = assemble(PROBE).unwrap();
    let id = seal(&mut region, &units[1]).unwrap();
    let v = open(&region, id).unwrap();
    assert_eq!(v.to_unit(), units[1]);
    assert_eq!(v.code(), &units[1].code[..]);
    assert!(open(&region, hg_heap::TrustedId::from_bits(u64::MAX)).is_none());
}
