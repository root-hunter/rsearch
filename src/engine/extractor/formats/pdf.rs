use pdfium_render::prelude::*;

pub fn read_pdf_text(path: &str) -> Result<String, PdfiumError> {
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library("vendor/pdfium/lib/libpdfium.so")?
    );

    let document = pdfium.load_pdf_from_file(path, None)?;
    let mut text = String::new();

    for page in document.pages().iter() {
        let page_text = page.text().unwrap().to_string();
        text.push_str(&page_text);
        text.push('\n');
    }

    Ok(text)
}