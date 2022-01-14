use zydis;

/// formatter-scoped options, provided during construction.
#[derive(Default, Clone, Copy)]
pub struct FormatterOptions {
    colors: bool,
}

#[derive(Default, Clone)]
struct OriginalHooks {
    pre_instruction: Option<zydis::Hook>,
}

/// data passed to each formatter callback.
struct UserData {
    /// extra data passed to `Formatter.format_instruction`
    /// and along to the callbacks.
    foo: u64,
    orig: OriginalHooks,
    /// options provided during construction of formatter.
    options: FormatterOptions,
}

pub struct Formatter {
    options: FormatterOptions,
    inner: zydis::Formatter,
    orig: OriginalHooks,
}

impl Formatter {
    pub fn new(colors: bool) -> Formatter {
        let mut orig: OriginalHooks = Default::default();

        let mut inner = zydis::Formatter::new(zydis::FormatterStyle::INTEL).unwrap();

        orig.pre_instruction = Some(
            inner
                .set_pre_instruction(Box::new(
                    |formatter: &zydis::Formatter,
                     buf: &mut zydis::FormatterBuffer,
                     ctx: &mut zydis::FormatterContext,
                     userdata: Option<&mut dyn core::any::Any>|
                     -> zydis::Result<()> {
                        // demonstrate that userdata contains what we think it does.
                        // access data provided via userdata and then call original hook.

                        let userdata = userdata.expect("no userdata");
                        let userdata = userdata
                            .downcast_ref::<UserData>()
                            .expect("incorrect userdata type");

                        // reference various parameters so they're not elided by the compiler.
                        println!("ok: {}: {}", userdata.options.colors, userdata.foo);

                        // call original hook
                        let orig = userdata
                            .orig
                            .pre_instruction
                            .as_ref()
                            .expect("no original hook");
                        if let zydis::Hook::PreInstruction(orig) = orig {
                            if let Some(orig) = orig {
                                unsafe {
                                    orig(
                                        formatter as *const _ as *const zydis::ffi::ZydisFormatter,
                                        buf,
                                        ctx,
                                    )
                                };
                            } else {
                                // no original hook provided, skip.
                                // e.g. pre_instruction doesn't have a default hook.
                            }
                        } else {
                            panic!("unexpected original hook type");
                        }

                        Ok(())
                    },
                ))
                .expect("failed to set pre instruction"),
        );

        assert!(matches!(
            orig.pre_instruction,
            Some(zydis::Hook::PreInstruction(None)),
        ));

        Formatter {
            options: FormatterOptions { colors },
            inner,
            orig,
        }
    }

    pub fn format_instruction(
        &self,
        insn: &zydis::DecodedInstruction,
        va: u64,
        foo: u64,
    ) -> String {
        let mut userdata = UserData {
            foo,
            orig: self.orig.clone(),
            options: self.options.clone(),
        };

        let mut buffer = [0u8; 200];
        let mut buffer = zydis::OutputBuffer::new(&mut buffer[..]);

        self.inner
            .format_instruction(insn, &mut buffer, Some(va), Some(&mut userdata))
            .expect("failed to format");

        buffer.as_str().expect("failed to get string").to_string()
    }
}

fn main() -> () {
    let decoder = zydis::Decoder::new(zydis::MachineMode::LEGACY_32, zydis::AddressWidth::_32)
        .expect("failed to create decoder");

    let code = &b"\xB8\x01\x00\x00\x00";
    let insn = decoder
        .decode(&code[..])
        .expect("failed to disassemble")
        .expect("invalid instruction");

    let fmt = Formatter::new(true);
    let s = fmt.format_instruction(&insn, 0x0, 42);
    println!("{}", s);

    ()
}
