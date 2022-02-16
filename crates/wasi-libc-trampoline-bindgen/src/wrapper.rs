use heck::*;
use std::io::{Read, Write};
use std::mem;
use std::path::Path;
use std::process::{Command, Stdio};
use witx::*;

pub fn generate<P: AsRef<Path>>(witx_paths: &[P]) -> String {
    let doc = witx::load(witx_paths).unwrap();

    let mut raw = String::new();
    raw.push_str(
        "\
// This file is automatically generated, DO NOT EDIT
//
// To regenerate this file run the `crates/wasi-libc-trampoline-bindgen` command
#![allow(unused_variables)]
use wasi::*;
use crate::UserFd;

",
    );
    for m in doc.modules() {
        m.render(&mut raw);
        raw.push('\n');
    }

    let mut rustfmt = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    rustfmt
        .stdin
        .take()
        .unwrap()
        .write_all(raw.as_bytes())
        .unwrap();
    let mut ret = String::new();
    rustfmt
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut ret)
        .unwrap();
    let status = rustfmt.wait().unwrap();
    assert!(status.success());
    ret
}

trait Render {
    fn render(&self, src: &mut String);
}

impl Render for IntRepr {
    fn render(&self, src: &mut String) {
        match self {
            IntRepr::U8 => src.push_str("u8"),
            IntRepr::U16 => src.push_str("u16"),
            IntRepr::U32 => src.push_str("u32"),
            IntRepr::U64 => src.push_str("u64"),
        }
    }
}

impl Render for TypeRef {
    fn render(&self, src: &mut String) {
        match self {
            TypeRef::Name(t) => {
                src.push_str(&t.name.as_str().to_camel_case());
                if let Type::List(_) = &**t.type_() {
                    src.push_str("<'_>");
                }
            }
            TypeRef::Value(v) => match &**v {
                Type::Builtin(t) => t.render(src),
                Type::List(t) => match &**t.type_() {
                    Type::Builtin(BuiltinType::Char) => src.push_str("&str"),
                    _ => {
                        src.push_str("&'a [");
                        t.render(src);
                        src.push(']');
                    }
                },
                Type::Pointer(t) => {
                    src.push_str("*mut ");
                    t.render(src);
                }
                Type::ConstPointer(t) => {
                    src.push_str("*const ");
                    t.render(src);
                }
                Type::Variant(v) if v.is_bool() => src.push_str("bool"),
                Type::Variant(v) => match v.as_expected() {
                    Some((ok, err)) => {
                        src.push_str("Result<");
                        match ok {
                            Some(ty) => ty.render(src),
                            None => src.push_str("()"),
                        }
                        src.push(',');
                        match err {
                            Some(ty) => ty.render(src),
                            None => src.push_str("()"),
                        }
                        src.push('>');
                    }
                    None => {
                        panic!("unsupported anonymous variant")
                    }
                },
                Type::Record(r) if r.is_tuple() => {
                    src.push('(');
                    for member in r.members.iter() {
                        member.tref.render(src);
                        src.push(',');
                    }
                    src.push(')');
                }
                t => panic!("reference to anonymous {} not possible!", t.kind()),
            },
        }
    }
}

impl Render for BuiltinType {
    fn render(&self, src: &mut String) {
        match self {
            // A C `char` in Rust we just interpret always as `u8`. It's
            // technically possible to use `std::os::raw::c_char` but that's
            // overkill for the purposes that we'll be using this type for.
            BuiltinType::U8 { lang_c_char: _ } => src.push_str("u8"),
            BuiltinType::U16 => src.push_str("u16"),
            BuiltinType::U32 {
                lang_ptr_size: false,
            } => src.push_str("u32"),
            BuiltinType::U32 {
                lang_ptr_size: true,
            } => src.push_str("usize"),
            BuiltinType::U64 => src.push_str("u64"),
            BuiltinType::S8 => src.push_str("i8"),
            BuiltinType::S16 => src.push_str("i16"),
            BuiltinType::S32 => src.push_str("i32"),
            BuiltinType::S64 => src.push_str("i64"),
            BuiltinType::F32 => src.push_str("f32"),
            BuiltinType::F64 => src.push_str("f64"),
            BuiltinType::Char => src.push_str("char"),
        }
    }
}

impl Render for Module {
    fn render(&self, src: &mut String) {
        for f in self.funcs() {
            if !crate::WASI_HOOK_FUNCTIONS.contains(&f.name.as_str()) {
                continue;
            }
            let mut f_name = String::new();
            f.name.render(&mut f_name);

            render_trampoline(
                &*f,
                &format!("wasi_vfs_{}", f_name.to_snake_case()),
                &self.name,
                src,
            );
            src.push('\n');
        }
    }
}

fn render_trace_syscall_entry_format_args(func: &InterfaceFunc, src: &mut String) {
    src.push_str("format_args!(\"");
    src.push_str(func.name.as_str());
    src.push('(');

    let mut raw_arg_names = vec![];
    for param in func.params.iter() {
        match &**param.tref.type_() {
            Type::List(_) => {
                raw_arg_names.push(String::from(param.name.as_str()));
                raw_arg_names.push(format!("{}_len", param.name.as_str()));
            }
            _ => {
                raw_arg_names.push(String::from(param.name.as_str()));
            }
        }
    }

    src.push_str(
        &raw_arg_names
            .iter()
            .map(|name| format!("{}: {{}}", name))
            .collect::<Vec<_>>()
            .join(", "),
    );
    src.push_str(")\\n\"");
    for (i, _) in raw_arg_names.iter().enumerate() {
        src.push_str(", ");
        src.push_str("arg");
        src.push_str(&i.to_string());
    }
    src.push(')');
}

fn render_trampoline(func: &InterfaceFunc, name: &str, module: &Id, src: &mut String) {
    src.push_str(" #[no_mangle]\n");
    src.push_str("pub unsafe extern \"C\" fn ");
    src.push_str(name);

    let (params, results) = func.wasm_signature();
    assert!(results.len() <= 1);
    src.push('(');

    for (i, param) in params.iter().enumerate() {
        src.push_str(&format!("arg{}: ", i));
        param.render(src);
        src.push(',');
    }
    src.push(')');

    if func.noreturn {
        src.push_str(" -> !");
    } else if let Some(result) = results.get(0) {
        src.push_str(" -> ");
        result.render(src);
    }
    src.push_str("{\n");

    {
        src.push_str("#[cfg(feature = \"trace-syscall\")]\n");
        src.push_str("crate::trace::trace_syscall_entry(");
        render_trace_syscall_entry_format_args(func, src);
        src.push_str(");\n");
    }
    {
        src.push_str(
            "let fs = if let Some(fs) = crate::GLOBAL_STATE.overlay_fs.as_mut() { fs } else {\n",
        );
        src.push_str(&format!(
            "return wasi::{}::{}(\n",
            module.as_str(),
            func.name.as_str()
        ));
        for (i, _) in params.iter().enumerate() {
            src.push_str(&format!("arg{}, ", i));
        }
        src.push_str(");\n");
        src.push_str("};\n");
    }
    func.call_interface(
        module,
        &mut Rust {
            src,
            func_name: func.name.as_str(),
            block_storage: vec![],
            blocks: vec![],
        },
    );

    src.push('}');
}

struct Rust<'a> {
    src: &'a mut String,
    func_name: &'a str,
    block_storage: Vec<String>,
    blocks: Vec<String>,
}

impl Bindgen for Rust<'_> {
    type Operand = String;

    fn emit(
        &mut self,
        inst: &Instruction<'_>,
        operands: &mut Vec<Self::Operand>,
        results: &mut Vec<Self::Operand>,
    ) {
        let mut top_as = |cvt: &str| {
            let mut s = operands.pop().unwrap();
            s.push_str(" as ");
            s.push_str(cvt);
            results.push(s);
        };

        match inst {
            Instruction::GetArg { nth } => {
                results.push(format!("arg{}", nth));
            }
            Instruction::AddrOf => todo!(),
            Instruction::I32FromChar => todo!(),
            Instruction::I64FromU64 => todo!(),
            Instruction::I64FromS64 => todo!(),
            Instruction::I32FromU32 => todo!(),
            Instruction::I32FromS32 => todo!(),
            Instruction::I32FromUsize => todo!(),
            Instruction::I32FromU16 => todo!(),
            Instruction::I32FromS16 => todo!(),
            Instruction::I32FromU8 => todo!(),
            Instruction::I32FromS8 => todo!(),
            Instruction::I32FromChar8 => todo!(),
            Instruction::I32FromPointer => todo!(),
            Instruction::I32FromConstPointer => todo!(),
            Instruction::I32FromHandle { .. } => todo!(),
            Instruction::I32FromBitflags { .. } => todo!(),
            Instruction::I64FromBitflags { .. } => todo!(),
            Instruction::ListPointerLength => todo!(),
            Instruction::ListFromPointerLength { ty } => {
                let ptr = &operands[0];
                let len = &operands[1];
                match &**ty.type_() {
                    witx::Type::Builtin(witx::BuiltinType::Char) => {
                        results.push(format!("{{
                            let str_bytes = core::slice::from_raw_parts({} as *const u8, ({} + 1) as usize);
                            std::ffi::CStr::from_bytes_with_nul_unchecked(str_bytes)
                        }}", ptr, len));
                    }
                    _ => {
                        results.push(format!(
                            "core::slice::from_raw_parts({} as *const {}, {} as usize)",
                            ptr,
                            ty.to_rust_ident(),
                            len
                        ));
                    }
                };
            }
            Instruction::F32FromIf32 => todo!(),
            Instruction::F64FromIf64 => todo!(),
            Instruction::CallInterface { module, func } => {
                results.push(format!(
                    "crate::{}::{}(fs, {})",
                    module,
                    func.name.as_str(),
                    operands.join(", ")
                ));
            }
            Instruction::S8FromI32 => todo!(),
            Instruction::U8FromI32 => todo!(),
            Instruction::S16FromI32 => todo!(),
            Instruction::U16FromI32 => todo!(),
            Instruction::S32FromI32 => todo!(),
            Instruction::U32FromI32 => top_as("u32"),
            Instruction::UsizeFromI32 => top_as("usize"),
            Instruction::S64FromI64 => top_as("i64"),
            Instruction::U64FromI64 => top_as("u64"),
            Instruction::CharFromI32 => todo!(),
            Instruction::Char8FromI32 => todo!(),
            Instruction::If32FromF32 => todo!(),
            Instruction::If64FromF64 => todo!(),
            Instruction::HandleFromI32 { ty } => {
                if ty.name.as_str() == "fd" {
                    top_as("UserFd");
                } else {
                    top_as(&ty.to_rust_ident())
                }
            }
            Instruction::PointerFromI32 { ty } => top_as(&format!("*mut {}", ty.to_rust_ident())),
            Instruction::ConstPointerFromI32 { ty } => {
                top_as(&format!("*const {}", ty.to_rust_ident()))
            }
            Instruction::BitflagsFromI32 { ty } => top_as(&ty.to_rust_ident()),
            Instruction::BitflagsFromI64 { ty } => top_as(&ty.to_rust_ident()),
            Instruction::ReturnPointerGet { .. } => todo!(),
            Instruction::Load { .. } => todo!(),
            Instruction::Store { ty } => {
                let ptr = operands.pop().unwrap();
                let val = operands.pop().unwrap();
                self.src.push_str(&format!(
                    "core::ptr::write({} as *mut {}, {})",
                    ptr,
                    ty.to_rust_ident(),
                    val
                ));
            }
            Instruction::ResultLift => todo!(),
            Instruction::ResultLower { .. } => {
                let err = self.blocks.pop().unwrap();
                let ok = self.blocks.pop().unwrap();
                let val = operands.pop().unwrap();
                results.push(format!(
                    "{{
                    match {} {{
                        Ok(e) => {{ {}; wasi::ERRNO_SUCCESS.raw() as i32 }}
                        Err(e) => {{
                            #[cfg(feature = \"trace-syscall\")]
                            crate::trace::trace_syscall_error(\"{}\", e.clone());

                            {}
                        }}
                    }}
                }}",
                    val, ok, self.func_name, err
                ));
            }
            Instruction::EnumLift { .. } => {
                // noop because some enum's constructor are invisible
                results.push(operands.pop().unwrap())
            }
            Instruction::EnumLower { ty } => {
                // noop because some enum's constructor are invisible
                assert_eq!(ty.name.as_str(), "errno");
                let val = operands.pop().unwrap();
                results.push(format!("{}.raw() as i32", val));
            }
            Instruction::TupleLift { .. } => todo!(),
            Instruction::TupleLower { .. } => todo!(),
            Instruction::ReuseReturn => todo!(),
            Instruction::Return { amt: 1 } => {
                let ret = operands.pop().unwrap();
                self.src.push_str(&ret);
            }
            Instruction::Return { .. } => todo!(),
            Instruction::VariantPayload => results.push(String::from("e")),
            other => panic!("no implementation for {:?}", other),
        }
    }

    fn allocate_space(&mut self, _slot: usize, _ty: &NamedType) {
        unimplemented!();
    }

    fn push_block(&mut self) {
        let prev = std::mem::take(self.src);
        self.block_storage.push(prev);
    }

    fn finish_block(&mut self, operand: Option<Self::Operand>) {
        let to_restore = self.block_storage.pop().unwrap();
        let src = mem::replace(self.src, to_restore);
        match operand {
            None => {
                self.blocks.push(src);
            }
            Some(s) => {
                if src.is_empty() {
                    self.blocks.push(s);
                } else {
                    self.blocks.push(format!("{{ {}; {} }}", src, s));
                }
            }
        }
    }
}

fn to_rust_ident(name: &str) -> &str {
    match name {
        "in" => "in_",
        "type" => "type_",
        "yield" => "yield_",
        s => s,
    }
}

impl Render for Id {
    fn render(&self, src: &mut String) {
        src.push_str(to_rust_ident(self.as_str()))
    }
}

impl Render for WasmType {
    fn render(&self, src: &mut String) {
        match self {
            WasmType::I32 => src.push_str("i32"),
            WasmType::I64 => src.push_str("i64"),
            WasmType::F32 => src.push_str("f32"),
            WasmType::F64 => src.push_str("f64"),
        }
    }
}

trait ToRustIdent {
    fn to_rust_ident(&self) -> String;
}

impl ToRustIdent for NamedType {
    fn to_rust_ident(&self) -> String {
        let mut buf = String::new();
        let src = &mut buf;
        src.push_str(&self.name.as_str().to_camel_case());
        if let Type::List(_) = &**self.type_() {
            src.push_str("<'_>");
        }
        buf
    }
}

impl ToRustIdent for TypeRef {
    fn to_rust_ident(&self) -> String {
        let mut buf = String::new();
        let src = &mut buf;
        match self {
            TypeRef::Name(t) => {
                src.push_str(&t.name.as_str().to_camel_case());
                if let Type::List(_) = &**t.type_() {
                    src.push_str("<'_>");
                }
            }
            TypeRef::Value(v) => match &**v {
                Type::Builtin(t) => t.render(src),
                Type::List(t) => match &**t.type_() {
                    Type::Builtin(BuiltinType::Char) => src.push_str("&str"),
                    _ => {
                        src.push_str("&'a [");
                        t.render(src);
                        src.push(']');
                    }
                },
                Type::Pointer(t) => {
                    src.push_str("*mut ");
                    t.render(src);
                }
                Type::ConstPointer(t) => {
                    src.push_str("*const ");
                    t.render(src);
                }
                Type::Variant(v) if v.is_bool() => src.push_str("bool"),
                Type::Variant(v) => match v.as_expected() {
                    Some((ok, err)) => {
                        src.push_str("Result<");
                        match ok {
                            Some(ty) => ty.render(src),
                            None => src.push_str("()"),
                        }
                        src.push(',');
                        match err {
                            Some(ty) => ty.render(src),
                            None => src.push_str("()"),
                        }
                        src.push('>');
                    }
                    None => {
                        panic!("unsupported anonymous variant")
                    }
                },
                Type::Record(r) if r.is_tuple() => {
                    src.push('(');
                    for member in r.members.iter() {
                        member.tref.render(src);
                        src.push(',');
                    }
                    src.push(')');
                }
                t => panic!("reference to anonymous {} not possible!", t.kind()),
            },
        }
        buf
    }
}
