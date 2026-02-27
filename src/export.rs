use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, XlsxError};
use std::fs::File;
use std::io::{BufWriter, Write};

use crate::checker::UrlStatus;

// ─── CSV Writer (real-time, appends each row immediately) ────────────────────

pub struct CsvWriter {
    writer: BufWriter<File>,
    row_count: u32,
}

impl CsvWriter {
    pub fn new(path: &str) -> std::io::Result<Self> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writeln!(writer, "No,URL,Status,Status Text,Redirect URL")?;
        writer.flush()?;

        Ok(Self { writer, row_count: 0 })
    }

    pub fn append_row(&mut self, result: &UrlStatus) -> std::io::Result<()> {
        self.row_count += 1;

        let status = match result.status_code {
            Some(code) => code.to_string(),
            None => "ERR".to_string(),
        };

        let redirect = result.redirect_url.as_deref().unwrap_or("");

        // Escape fields that might contain commas or quotes
        writeln!(
            self.writer,
            "{},\"{}\",{},\"{}\",\"{}\"",
            self.row_count,
            result.url.replace('"', "\"\""),
            status,
            result.status_text.replace('"', "\"\""),
            redirect.replace('"', "\"\""),
        )?;
        self.writer.flush()?;

        Ok(())
    }

    pub fn row_count(&self) -> u32 {
        self.row_count
    }
}

// ─── XLSX Writer (final export) ──────────────────────────────────────────────

pub fn export_to_xlsx(results: &[UrlStatus], path: &str) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    worksheet.set_name("Sitemap")?;
    worksheet.set_column_width(0, 8)?;
    worksheet.set_column_width(1, 80)?;
    worksheet.set_column_width(2, 12)?;
    worksheet.set_column_width(3, 22)?;
    worksheet.set_column_width(4, 60)?;

    let header_format = Format::new()
        .set_bold()
        .set_font_size(12)
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x2E86AB))
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin);

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

    worksheet.write_string_with_format(0, 0, "No.", &header_format)?;
    worksheet.write_string_with_format(0, 1, "URL", &header_format)?;
    worksheet.write_string_with_format(0, 2, "Status", &header_format)?;
    worksheet.write_string_with_format(0, 3, "Status Text", &header_format)?;
    worksheet.write_string_with_format(0, 4, "Redirect URL", &header_format)?;

    for (i, result) in results.iter().enumerate() {
        let row = (i + 1) as u32;

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
