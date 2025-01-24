use core::intrinsics::transmute;

use std::error::Error;

use crate::{memory::patch, PAD};

pub(crate) unsafe fn patch_app() -> Result<(), Box<dyn Error>> {
    /*
     * Make app width to 800 + 2 * PAD (LawnApp::LawnApp)
     * 0045384b        arg1[0x30] = 800  // Width
     * 00453855        arg1[0x31] = 600  // Height
     */
    patch(0x453851, &transmute::<i16, [u8; 2]>(800 + 2 * PAD));

    Ok(())
}
