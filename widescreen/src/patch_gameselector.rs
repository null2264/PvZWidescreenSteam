use core::intrinsics::transmute;

use std::error::Error;

use iced_x86::code_asm::*;

use crate::{
    memory::{inject, patch},
    PAD, PAD_CONST_PTR,
};

pub(crate) unsafe fn move_string(inject_address: u32, jump_address: u64) -> Result<(), Box<dyn Error>> {
    let mut code = CodeAssembler::new(32)?;
    code.fld(dword_ptr(esp + 0x4))?;
    code.fiadd(dword_ptr(PAD_CONST_PTR))?;
    code.fstp(dword_ptr(esp + 0x4))?;
    code.call(0x520960)?;
    code.jmp(jump_address)?;
    inject(inject_address, code);
    Ok(())
}

// FIXME: Everything but the buttons is misaligned
pub(crate) unsafe fn patch_gameselector() -> Result<(), Box<dyn Error>> {
    // Move GameSelector by PAD (LawnApp::ShowGameSelector)
    /*
     * 004546c5  6a00               push    0x0 {var_20}
     * 004546c7  8bc8               mov     ecx, eax
     * 004546c9  8b82a0000000       mov     eax, dword [edx+0xa0]
     * 004546cf  6a00               push    0x0 {var_24}
     * 004546d1  ffd0               call    eax
     */
    let mut code = CodeAssembler::new(32)?;
    code.push(0)?;
    code.mov(ecx, eax)?;
    code.mov(eax, dword_ptr(edx + 0xa0))?;
    code.push(PAD as i32)?;
    code.jmp(0x4546d1)?;
    inject(0x4546c5, code);

    // Move AdventureButton by PAD (unknown)
    /*
     * 00450360  83ec2c             sub     esp, 0x2c
     * 00450363  8b88a4000000       mov     ecx, dword [eax+0xa4]
     * 00450369  8b80f4000000       mov     eax, dword [eax+0xf4]
     */
    let mut code = CodeAssembler::new(32)?;
    code.fld(dword_ptr(esp + 0x8))?;
    code.fiadd(dword_ptr(PAD_CONST_PTR))?;
    code.fstp(dword_ptr(esp + 0x8))?;
    code.sub(esp, 0x2C)?;
    code.mov(ecx, dword_ptr(eax + 0xa4))?;
    code.jmp(0x450369)?;
    inject(0x450360, code);

    // // Move TodDrawStringMatrix argument by PAD (GameSelector::Draw)
    // let mut code = CodeAssembler::new(32)?;
    // code.push(eax)?;
    // code.mov(eax, dword_ptr(esp + 0xC))?;
    // code.fld(dword_ptr(eax + 0x8))?;
    // code.fiadd(dword_ptr(PAD_CONST_PTR))?;
    // code.fstp(dword_ptr(eax + 0x8))?;
    // code.pop(eax)?;
    // code.call(0x59f130)?;
    // code.jmp(0x44f55c)?;
    // inject(0x44f557, code);

    // move_string(0x44f2af, 0x44f2b4);
    // move_string(0x44f366, 0x44f36b);
    // move_string(0x44f327, 0x44f32c);

    // let mut code = CodeAssembler::new(32)?;
    // code.fld(dword_ptr(edi + 0x8))?;
    // code.fiadd(dword_ptr(PAD_CONST_PTR))?;
    // code.fstp(dword_ptr(edi + 0x8))?;
    // code.call(0x47adf0)?;
    // code.fld(dword_ptr(edi + 0x8))?;
    // code.fisub(dword_ptr(PAD_CONST_PTR))?;
    // code.fstp(dword_ptr(edi + 0x8))?;
    // code.jmp(0x44f4c9)?;
    // inject(0x44f4c4, code);

    // let mut code = CodeAssembler::new(32)?;
    // code.fld(dword_ptr(edi + 0x8))?;
    // code.fiadd(dword_ptr(PAD_CONST_PTR))?;
    // code.fstp(dword_ptr(edi + 0x8))?;
    // code.call(0x47adf0)?;
    // code.fld(dword_ptr(edi + 0x8))?;
    // code.fisub(dword_ptr(PAD_CONST_PTR))?;
    // code.fstp(dword_ptr(edi + 0x8))?;
    // code.jmp(0x44f42e)?;
    // inject(0x44f429, code);

    // let mut code = CodeAssembler::new(32)?;
    // code.fld(dword_ptr(edi + 0x8))?;
    // code.fiadd(dword_ptr(PAD_CONST_PTR))?;
    // code.fstp(dword_ptr(edi + 0x8))?;
    // code.call(0x47adf0)?;
    // code.fld(dword_ptr(edi + 0x8))?;
    // code.fisub(dword_ptr(PAD_CONST_PTR))?;
    // code.fstp(dword_ptr(edi + 0x8))?;
    // code.jmp(0x44f477)?;
    // inject(0x44f472, code);

    // let mut code = CodeAssembler::new(32)?;
    // code.mov(dword_ptr(esi + 0x20), 0)?;
    // code.mov(dword_ptr(esi + 0x28), 800 as i32)?;
    // code.call(0x47aeb0)?;
    // code.jmp(0x4516af)?;
    // inject(0x4516aa, code);

    // let mut code = CodeAssembler::new(32)?;
    // code.fld(dword_ptr(esp + 0x4))?;
    // code.fiadd(dword_ptr(PAD_CONST_PTR))?;
    // code.fstp(dword_ptr(esp + 0x4))?;
    // code.call(0x520960)?;
    // code.jmp(0x44f725)?;
    // inject(0x44f714, code);

    // let mut code = CodeAssembler::new(32)?;
    // code.mov(eax, dword_ptr(eax + 0x108))?;
    // code.sub(eax, PAD as i32)?;
    // code.jmp(0x44f7fb)?;
    // inject(0x44f7f5, code);

    patch(0x44f874, &transmute::<i16, [u8; 2]>(32 + PAD));
    patch(0x44f8e3, &transmute::<i16, [u8; 2]>(20 + PAD));

    Ok(())
}
