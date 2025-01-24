use core::intrinsics::transmute;

use std::error::Error;

use iced_x86::code_asm::*;
use windows::Win32::System::Memory::PAGE_READWRITE;

use crate::{
    memory::{change_permission, inject, patch, float_ptr, float64_ptr},
    PAD,
};

pub(crate) unsafe fn patch_titlescreen() -> Result<(), Box<dyn Error>> {
    // Move IMAGE_PVZ_LOGO by PAD (TitleScreen::Draw)
    /*
     * 0049c8a3  83c40c             add     esp, 0xc
     * 0049c8a6  50                 push    eax {var_fc_20}
     * 0049c8a7  a1f8f27200         mov     eax, dword [img_pvzLogo]
     * ---
     * 0049c8ac  6a32               push    0x32 {var_100}
     * 0049c8ae  50                 push    eax {var_104_8}
     * 0049c8af  8bcf               mov     ecx, edi
     * 0049c8b1  e8ca281000         call    render
     * 0049c8b6  8b8ea4000000       mov     ecx, dword [esi+0xa4]
     * ---
     */
    let mut code = CodeAssembler::new(32)?;
    code.push(0x32 + PAD as i32)?;
    code.push(eax)?;
    code.mov(ecx, edi)?;
    code.call(0x59f180)?;
    code.jmp(0x49c8b6)?;
    inject(0x49c8ac, code);

    // Move IMAGE_LOADBAR_DIRT by PAD (TitleScreen::Draw)
    /*
     * 0049c8b6  8b8ea4000000       mov     ecx, dword [esi+0xa4]
     * 0049c8bc  8b5944             mov     ebx, dword [ecx+0x44]
     * 0049c8bf  a1a0f67200         mov     eax, dword [img_loadBar_dirt]
     * 0049c8c4  83eb11             sub     ebx, 0x11
     * 0049c8c7  8d5312             lea     edx, [ebx+0x12]
     * 0049c8ca  52                 push    edx {var_fc_21}
     * ---
     * 0049c8cb  68f4000000         push    0xf4 {var_100}
     * 0049c8d0  50                 push    eax {var_104_9}
     * 0049c8d1  8bcf               mov     ecx, edi
     * 0049c8d3  895c2424           mov     dword [esp+0x24 {var_e0_1}], ebx
     * 0049c8d7  e8a4281000         call    render
     * 0049c8dc  d986a8000000       fld     st0, dword [esi+0xa8]
     * ---
     */
    let mut code = CodeAssembler::new(32)?;
    code.push(0xf4 + PAD as i32)?;
    code.push(eax)?;
    code.mov(ecx, edi)?;
    code.mov(dword_ptr(esp + 0x24), ebx)?;
    code.call(0x59f180)?;
    code.jmp(0x49c8dc)?;
    inject(0x49c8cb, code);

    // Move IMAGE_LOADBAR_GRASS by PAD (TitleScreen::Draw)
    patch(0x49c8f9, &transmute::<i16, [u8; 2]>(240 + PAD));
    patch(0x49c963, &transmute::<i16, [u8; 2]>(240 + PAD));
    patch(0x49c978, &transmute::<i16, [u8; 2]>(240 + PAD));

    // Move loading button by PAD (TitleScreen::Update)
    patch(0x49cd39, &transmute::<i16, [u8; 2]>(240 + PAD));

    // Move IMAGE_PLANT_SHADOW to 2000.0 (TitleScreen::Draw)
    patch(0x49c923, &transmute::<u32, [u8; 4]>(float_ptr(2000.0)));

    // Move IMAGE_SODROLL_CAP by PAD (TitleScreen::Draw)
    patch(0x49ca37, &transmute::<u32, [u8; 4]>(float64_ptr(251.0 + PAD as f64)));

    // Move loadbar flowers (TitleScreen::Update)
    patch(0x49d30e, &transmute::<u32, [u8; 4]>(float64_ptr(225.0 + PAD as f64)));

    Ok(())
}
