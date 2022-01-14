use zydis;

/// data passed to each formatter callback.
struct UserData {
    /// extra data passed to formatting callbacks.
    foo: u64,
}

fn main() -> () {
    let decoder = zydis::Decoder::new(zydis::MachineMode::LEGACY_32, zydis::AddressWidth::_32)
        .expect("failed to create decoder");

    let code = &b"\xB8\x01\x00\x00\x00";
    let insn = decoder
        .decode(&code[..])
        .expect("failed to disassemble")
        .expect("invalid instruction");

    let mut fmt = zydis::Formatter::new(zydis::FormatterStyle::INTEL).unwrap();

    fmt.set_pre_instruction(Box::new(
        |_formatter: &zydis::Formatter,
         _buf: &mut zydis::FormatterBuffer,
         _ctx: &mut zydis::FormatterContext,
         userdata: Option<&mut dyn core::any::Any>|
         -> zydis::Result<()> {
            // demonstrate that userdata contains what we think it does.
            // access data provided via userdata and then call original hook.

            let userdata = userdata.expect("no userdata");
            let userdata = userdata
                .downcast_ref::<UserData>()
                .expect("incorrect userdata type");

            // reference various parameters so they're not elided by the compiler.
            println!("foo: {}", userdata.foo);

            // don't actually format anything

            Ok(())
        },
    ))
    .expect("failed to set pre instruction");

    let mut userdata = UserData { foo: 42 };

    let mut buffer = [0u8; 200];
    let mut buffer = zydis::OutputBuffer::new(&mut buffer[..]);
    fmt.format_instruction(&insn, &mut buffer, Some(0x0), Some(&mut userdata))
        .expect("failed to format");

    println!(
        "{}",
        buffer.as_str().expect("failed to get string").to_string()
    );

    ()
}
