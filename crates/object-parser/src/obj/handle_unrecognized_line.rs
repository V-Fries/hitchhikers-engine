pub fn handle_unrecognized_line(first_word: &str, line_count: usize, line: &str) {
    if first_word.starts_with('#') {
        return;
    }
    if cfg!(debug_assertions) {
        eprintln!("WARNING: PPM parser(");
        eprintln!("\tline: {line_count},");
        eprintln!("\tline_content: \"{line}\"");
        eprintln!("\terror: \"{first_word}\" is not supported");
        eprintln!(")");
    }
}
