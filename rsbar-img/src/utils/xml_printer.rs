use std::path::Path;

const INDENT_WIDTH: usize = 4;
const INDENT_CHARACTER: char = ' ';

pub struct XmlPrinter {}

impl XmlPrinter {
    pub fn print_head() {
        println!("<barcodes xmlns=\"http://zbar.sourceforge.net/2008/barcode\">");
    }

    pub fn print_foot() {
        println!("</barcodes>");
    }

    fn print_xml(xml: String, indent_level: usize) {
        println!(
            "{INDENT_CHARACTER:>indent$}{xml}",
            indent = indent_level * INDENT_WIDTH
        );
    }

    pub fn print_source_head(file_path: &Path) {
        Self::print_xml(format!("<source href=\"{}\">", file_path.display()), 1);
    }

    pub fn print_source_foot() {
        Self::print_xml("</source>".to_string(), 1);
    }

    pub fn print_index_head(num: u8) {
        Self::print_xml(format!("<index num=\"{num}\">"), 2);
    }

    pub fn print_index_foot() {
        Self::print_xml("</index>".to_string(), 2);
    }

    pub fn print_symbol(symbol_xml: &str) {
        Self::print_xml(symbol_xml.to_string(), 3);
    }
}
