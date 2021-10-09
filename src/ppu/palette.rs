use std::ops::{Index, IndexMut};

use super::color::{Color, ColorScheme, ColorType};

pub type Palette = [(Color, ColorType); 4];

/// Wraps the three GB palettes (background, object0 and object1)
pub struct Palettes {
    color_scheme: ColorScheme,

    bg: Palette,
    obj0: Palette,
    obj1: Palette,
}

impl Palettes {
    /// Get a given palette
    pub fn get(&self, palette: PaletteKind) -> &Palette {
        match palette {
            PaletteKind::BG => &self.bg,
            PaletteKind::OBJ0 => &self.obj0,
            PaletteKind::OBJ1 => &self.obj1,
        }
    }

    pub fn update(&mut self, palette: PaletteKind, color_index: usize, color_type: ColorType) {
        let palette_color = (self.color_scheme.get(color_type), color_type);

        match palette {
            PaletteKind::BG => self.bg[color_index] = palette_color,
            PaletteKind::OBJ0 => self.obj0[color_index] = palette_color,
            PaletteKind::OBJ1 => self.obj1[color_index] = palette_color,
        }
    }

    pub fn set_color_scheme(&mut self, color_scheme: ColorScheme) {
        for (color, ref color_type) in self.bg.iter_mut() {
            *color = color_scheme.get(*color_type)
        }
        for (color, ref color_type) in self.obj0.iter_mut() {
            *color = color_scheme.get(*color_type)
        }
        for (color, ref color_type) in self.obj1.iter_mut() {
            *color = color_scheme.get(*color_type)
        }

        self.color_scheme = color_scheme;
    }
}

impl Default for Palettes {
    fn default() -> Self {
        let colors = ColorScheme::default();

        let bg_palette = [
            (colors.white, ColorType::White),
            (colors.light_gray, ColorType::LightGray),
            (colors.dark_gray, ColorType::DarkGray),
            (colors.black, ColorType::Black),
        ];
        let obj0_palette = [(colors.white, ColorType::White); 4];
        let obj1_palette = [(colors.white, ColorType::White); 4];

        Palettes {
            color_scheme: colors,

            bg: bg_palette,
            obj0: obj0_palette,
            obj1: obj1_palette,
        }
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
