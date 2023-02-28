use bitflags::bitflags;

/* VSO. ....
   |||| ||||
   |||+-++++- PPU open bus. Returns stale PPU bus contents.
   ||+------- Sprite overflow. The intent was for this flag to be set
   ||         whenever more than eight sprites appear on a scanline, but a
   ||         hardware bug causes the actual behavior to be more complicated
   ||         and generate false positives as well as false negatives; see
   ||         PPU sprite evaluation. This flag is set during sprite
   ||         evaluation and cleared at dot 1 (the second dot) of the
   ||         pre-render line.
   |+-------- Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
   |          a nonzero background pixel; cleared at dot 1 of the pre-render
   |          line.  Used for raster timing.
   +--------- Vertical blank has started (0: not in vblank; 1: in vblank).
              Set at dot 1 of line 241 (the line *after* the post-render
              line); cleared after reading $2002 and at dot 1 of the
              pre-render line.
*/
bitflags! {
  pub struct PpuFlags: u8 {
    const OVERFLOW        = 0b0010_0000;
    const SPRITE_ZERO_HIT = 0b0100_0000;
    const VBLANK_STARTED  = 0b1000_0000;
  }
}

pub struct StatusRegister {
    flags: PpuFlags,
}

impl StatusRegister {
    pub fn new() -> Self {
        Self {
            flags: PpuFlags::from_bits_truncate(0b1010_0000),
        }
    }
    pub fn set_flag(&mut self, flag: PpuFlags) {
        self.flags.insert(flag);
    }
}
