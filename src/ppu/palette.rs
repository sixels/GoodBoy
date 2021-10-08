use std::ops::{Index, IndexMut};

use crate::ppu::color::Color;

pub type Palette = [Color; 4];

/// Wraps the three GB palettes (background, object0 and object1)
pub struct Palettes([Palette; 3]);

impl Palettes {
    /// Get a given palette
    pub fn get(&self, palette: PaletteKind) -> &Palette {
        &self.0[usize::from(palette)]
    }
    pub fn get_mut(&mut self, palette: PaletteKind) -> &mut Palette {
        &mut self.0[usize::from(palette)]
    }
}

impl Default for Palettes {
    fn default() -> Self {
        let bg_palette = [Color::WHITE, Color::LGRAY, Color::DGRAY, Color::BLACK];
        let obj0_palette = [Color::WHITE; 4];
        let obj1_palette = [Color::WHITE; 4];

        Self([bg_palette, obj0_palette, obj1_palette])
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PaletteKind {
    BG = 0,
    OBJ0 = 1,
    OBJ1 = 2,
}

impl<T> Index<PaletteKind> for [T] {
    type Output = T;
    fn index(&self, index: PaletteKind) -> &Self::Output {
        &self[usize::from(index)]
    }
}
impl<T> IndexMut<PaletteKind> for [T] {
    fn index_mut(&mut self, index: PaletteKind) -> &mut Self::Output {
        &mut self[usize::from(index)]
    }
}

impl From<PaletteKind> for usize {
    fn from(palette: PaletteKind) -> Self {
        match palette {
            PaletteKind::BG => 0,
            PaletteKind::OBJ0 => 1,
            PaletteKind::OBJ1 => 2,
        }
    }
}
