use egui::{Context, FontId};

use crate::types::Size;

#[derive(Debug, Clone)]
pub struct FontSettings {
    pub font_type: FontId,
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            font_type: FontId::monospace(30.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TermFont {
    font_type: FontId,
}

impl Default for TermFont {
    fn default() -> Self {
        Self {
            font_type: FontSettings::default().font_type,
        }
    }
}

impl TermFont {
    pub fn new(settings: FontSettings) -> Self {
        Self {
            font_type: settings.font_type,
        }
    }

    pub fn font_type(&self) -> FontId {
        self.font_type.clone()
    }

    pub fn font_measure(&self, ctx: &Context) -> Size {
        let (width, height) = ctx.fonts(|f| (
            f.glyph_width(&self.font_type, 'M'),
            f.row_height(&self.font_type))
        );

        Size::new(width, height)
    }
}
