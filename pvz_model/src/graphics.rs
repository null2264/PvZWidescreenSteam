#![allow(non_snake_case)]

use core::arch::asm;

use std::ffi::c_void;

use crate::{stub::StdList, Color, Image, TRect};

#[repr(C)]
pub struct Edge {
    mX: f64,
    mDX: f64,
    i: i32,
    b: f64,
}

#[repr(C)]
pub struct GraphicsState {
    pub mDestImage: *mut Image,
    pub mTransX: f32,
    pub mTransY: f32,
    pub mScaleX: f32,
    pub mScaleY: f32,
    pub mScaleOrigX: f32,
    pub mScaleOrigY: f32,
    pub mClipRect: TRect<i32>,
    pub mColor: Color,
    pub mFont: *const c_void,
    pub mDrawMode: i32,
    pub mColorizeImages: bool,
    pub mFastStretch: bool,
    pub mWriteColoredString: bool,
    pub mLinearBlend: bool,
    pub mIs3D: bool,
}

#[repr(C)]
pub struct Graphics {
    unknown: [u8; 4],
    pub state: GraphicsState,
    pub mPFActiveEdgeList: *const Edge,
    pub mPFNumActiveEdges: i32,
    pub mPFNumVertices: i32,
    pub mStateStack: StdList,
}

impl Graphics {
    #[inline]
    pub unsafe fn DrawImage(&mut self, image: *const Image, x: i32, y: i32) {
        // let func: extern "C" fn(this: &mut Graphics, image: *const Image, x: i32, y: i32) = std::mem::transmute(0x59f180);
        // func(self, image, x, y);
        asm!(
            "pushad",
            "push {4}",
            "push {3}",
            "push {2}",
            "mov ecx, {1}",
            "call {0}",
            "popad",
            in(reg) 0x59f180,
            in(reg) self,
            in(reg) image,
            in(reg) x,
            in(reg) y,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::Edge;

    #[test]
    fn check_Edge_size() {
        assert_eq!(size_of::<Edge>(), 32);
    }
}
