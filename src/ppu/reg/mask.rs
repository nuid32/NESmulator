use bitflags::bitflags;

/* BGRs bMmG
   |||| ||||
   |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
   |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
   |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
   |||| +---- 1: Show background
   |||+------ 1: Show sprites
   ||+------- Emphasize red (green on PAL/Dendy)
   |+-------- Emphasize green (red on PAL/Dendy)
   +--------- Emphasize blue
*/
bitflags! {
   pub struct MaskRegister: u8 {
      const GREYSCALE            = 0b0000_0001;
      const SHOW_BACKGROUND_LEFT = 0b0000_0010;
      const SHOW_SPRITES_LEFT    = 0b0000_0100;
      const SHOW_BACKGROUND      = 0b0000_1000;
      const SHOW_SPRITES         = 0b0001_0000;
      const EMPHASIZE_RED        = 0b0010_0000;
      const EMPHASIZE_GREEN      = 0b0100_0000;
      const EMPHASIZE_BLUE       = 0b1000_0000;
   }
}

impl MaskRegister {
    pub fn new() -> Self {
        Self::empty()
    }

    pub fn update(&mut self, value: u8) {
        self.bits = value;
    }
}
