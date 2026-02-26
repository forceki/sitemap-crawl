use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, XlsxError};

use crate::checker::UrlStatus;

pub fn export_to_xlsx(results: &[UrlStatus], path: &str) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    worksheet.set_name("Sitemap")?;
    worksheet.set_column_width(0, 8)?;   // No
    worksheet.set_column_width(1, 80)?;  // URL
    worksheet.set_column_width(2, 12)?;  // Status Code
    worksheet.set_column_width(3, 22)?;  // Status Text
    worksheet.set_column_width(4, 60)?;  // Redirect URL

    let header_format = Format::new()
        .set_bold()
        .set_font_size(12)
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x2E86AB))
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin);

    worksheet.write_string_with_format(0, 0, "No.", &header_format)?;
    worksheet.write_string_with_format(0, 1, "URL", &header_format)?;
    worksheet.write_string_with_format(0, 2, "Status", &header_format)?;
    worksheet.write_string_with_format(0, 3, "Status Text", &header_format)?;
    worksheet.write_string_with_format(0, 4, "Redirect URL", &header_format)?;

    for (i, result) in results.iter().enumerate() {
        let row = (i + 1) as u32;


        // Temp Disable
        // let bg_color = match result.status_code {
        //     Some(200..=299) => Color::RGB(0xD4EDDA), // green
        //     Some(300..=399) => Color::RGB(0xFFF3CD), // yellow
        //     Some(400..=499) => Color::RGB(0xF8D7DA), // red
        //     Some(500..=599) => Color::RGB(0xF5C6CB), // dark red
        //     _               => Color::RGB(0xE2E3E5), // gray (error/timeout)
        // };

        let idx_fmt = Format::new()
            .set_font_size(11)
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin);

        let cell_fmt = Format::new()
            .set_font_size(11)
            .set_border(FormatBorder::Thin);

        let code_fmt = Format::new()
            .set_font_size(11)
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin);

        worksheet.write_number_with_format(row, 0, (i + 1) as f64, &idx_fmt)?;
        worksheet.write_string_with_format(row, 1, &result.url, &cell_fmt)?;

        match result.status_code {
            Some(code) => {
                worksheet.write_number_with_format(row, 2, code as f64, &code_fmt)?;
            }
            None => {
                worksheet.write_string_with_format(row, 2, "ERR", &code_fmt)?;
            }
        }

        worksheet.write_string_with_format(row, 3, &result.status_text, &cell_fmt)?;

        let redirect = result.redirect_url.as_deref().unwrap_or("");
        worksheet.write_string_with_format(row, 4, redirect, &cell_fmt)?;
    }

    workbook.save(path)?;
    Ok(())
}
