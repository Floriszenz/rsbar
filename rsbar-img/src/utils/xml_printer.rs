use std::path::Path;

const XML_HEAD: &str = "<barcodes xmlns=\"http://zbar.sourceforge.net/2008/barcode\">\n";
const XML_FOOT: &str = "</barcodes>\n";
const INDENT_WIDTH: usize = 4;
const INDENT_CHARACTER: char = ' ';

pub struct XmlPrinter {}

impl XmlPrinter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn print_head(&self) {
        print!("{XML_HEAD}");
    }

    pub fn print_foot(&self) {
        print!("{XML_FOOT}");
    }

    fn print_xml(&self, xml: String, indent_level: usize) {
        println!(
            "{INDENT_CHARACTER:>indent$}{xml}",
            indent = indent_level * INDENT_WIDTH
        );
    }

    pub fn print_source_head(&self, file_path: &Path) {
        self.print_xml(format!("<source href=\"{}\">", file_path.display()), 1);
    }

    pub fn print_source_foot(&self) {
        self.print_xml("</source>".to_string(), 1);
    }

    pub fn print_index_head(&self, num: u8) {
        self.print_xml(format!("<index num=\"{num}\">"), 2);
    }

    pub fn print_index_foot(&self) {
        self.print_xml("</index>".to_string(), 2);
    }

    pub fn print_symbol(&self, symbol_xml: &str) {
        self.print_xml(symbol_xml.to_string(), 3);
    }
}
