//! Vector PDF export consumes the same page SVGs as the editor.
use crate::studio::PageLayout;
use krilla::{Document, geom::Size, page::PageSettings};
use krilla_svg::{SurfaceExt, SvgSettings};
use std::sync::Arc;

pub fn pdf(layout: &PageLayout) -> Result<Vec<u8>, String> {
    let mut database = usvg::fontdb::Database::new();
    database.load_font_data(include_bytes!("../../../assets/fonts/Leland.otf").to_vec());
    database.load_font_data(include_bytes!("../../../assets/fonts/Edwin-Roman.otf").to_vec());
    let options = usvg::Options {
        fontdb: Arc::new(database),
        ..Default::default()
    };
    let mut document = Document::new();
    let size = Size::from_wh(layout.width as f32 * 0.75, layout.height as f32 * 0.75)
        .ok_or("Invalid paper dimensions")?;
    for score_page in &layout.pages {
        let tree = usvg::Tree::from_str(&score_page.svg, &options).map_err(|e| e.to_string())?;
        let mut page = document.start_page_with(PageSettings::new(size));
        let mut surface = page.surface();
        surface
            .draw_svg(&tree, size, SvgSettings::default())
            .ok_or("SVG could not be drawn into the PDF page")?;
        surface.finish();
        page.finish();
    }
    document.finish().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn exports_vector_pages_with_bundled_music_fonts() {
        let document = crate::studio::new_score();
        let layout = crate::studio::layout(&document, None).unwrap();
        let pdf = super::pdf(&layout).unwrap();
        assert!(pdf.starts_with(b"%PDF-"));
        assert!(pdf.ends_with(b"%%EOF\n") || pdf.ends_with(b"%%EOF"));
        assert!(pdf.len() > 1000);
    }
}
