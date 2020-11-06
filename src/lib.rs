#![deny(missing_docs)]

//! This crate turns (a subset of) latex into html, with syntax errors
//! reported using span elements.

use wasm_bindgen::prelude::*;

#[cfg(test)]
mod tests;

/// A version of html_string suitable for export to C and python.
#[no_mangle]
pub extern "C" fn convert_html(s: *const std::os::raw::c_char) -> *const std::os::raw::c_char {
    if s.is_null() {
        return std::ptr::null();
    }
    let c_str = unsafe { std::ffi::CStr::from_ptr(s) };
    if let Ok(my_str) = c_str.to_str() {
        let output = html_string(my_str);
        std::ffi::CString::new(output).unwrap().into_raw()
    } else {
        std::ptr::null()
    }
}

/// Cut out comments
///
/// This is only useful with html_section and friends, since html and
/// html_string do this automatically.
pub fn strip_comments(latex: &str) -> String {
    let temp = latex.replace(r"\%", r"\percent_holder");
    let mut out = String::with_capacity(temp.len() + 1);
    for x in temp.split('\n') {
        if x.chars().next() == Some('%') {
            continue; // skip this line entirely
        }
        if let Some(i) = x.find('%') {
            out.push_str(&x[..i]);
            out.push(' '); // comment "blocks" line ending.
        } else {
            out.push_str(x);
            out.push('\n')
        }
    }
    out.pop();
    out.replace(r"\percent_holder", r"\%")
}

/// Convert some LaTeX into an HTML `String`.
pub fn html_string(latex: &str) -> String {
    let mut s: Vec<u8> = Vec::with_capacity(latex.len());
    html(&mut s, latex).unwrap();
    String::from_utf8(s).expect("should be no problem with utf8 conversion")
}

macro_rules! ffi_str {
    ($mkstr:expr) => {
        |s: *const std::os::raw::c_char| -> *const std::os::raw::c_char {
            if s.is_null() {
                return std::ptr::null();
            }
            let c_str = unsafe { std::ffi::CStr::from_ptr(s) };
            if let Ok(my_str) = c_str.to_str() {
                let output = $mkstr(my_str);
                std::ffi::CString::new(output).unwrap().into_raw()
            } else {
                std::ptr::null()
            }
        }
    };
}

/// Convert some LaTeX into an HTML `String`.
#[wasm_bindgen]
#[cfg(target_arch = "wasm32")]
pub fn html_with_solution(latex: &str) -> String {
    set_panic_hook();
    html_string(&physics_macros(latex))
}

/// A version of html_with_solution suitable for export to C and python.
#[no_mangle]
pub extern "C" fn latex_to_html_with_solution(
    s: *const std::os::raw::c_char,
) -> *const std::os::raw::c_char {
    ffi_str!(|latex| { html_string(&physics_macros(latex)) })(s)
}

/// Convert some LaTeX into an HTML `String`.
#[wasm_bindgen]
#[cfg(target_arch = "wasm32")]
pub fn html_omit_solution(latex: &str) -> String {
    set_panic_hook();
    html_string(&omit_solutions(&physics_macros(latex)))
}

/// A version of html_with_solution suitable for export to C and python.
#[no_mangle]
pub extern "C" fn latex_to_html_omit_solution(
    s: *const std::os::raw::c_char,
) -> *const std::os::raw::c_char {
    ffi_str!(|latex| { html_string(&omit_solutions(&physics_macros(latex))) })(s)
}

/// A version of omit_solution suitable for export to C and python.
#[no_mangle]
pub extern "C" fn c_omit_solution(s: *const std::os::raw::c_char) -> *const std::os::raw::c_char {
    ffi_str!(|latex| { omit_solutions(&physics_macros(latex)) })(s)
}
/// A version of omit_guide suitable for export to C and python.
#[no_mangle]
pub extern "C" fn c_omit_guide(s: *const std::os::raw::c_char) -> *const std::os::raw::c_char {
    ffi_str!(|latex| { omit_guide(&physics_macros(latex)) })(s)
}
/// A version of omit_handout suitable for export to C and python.
#[no_mangle]
pub extern "C" fn c_omit_handout(s: *const std::os::raw::c_char) -> *const std::os::raw::c_char {
    ffi_str!(|latex| { omit_handout(&physics_macros(latex)) })(s)
}
/// A version of only_handout suitable for export to C and python.
#[no_mangle]
pub extern "C" fn c_only_handout(s: *const std::os::raw::c_char) -> *const std::os::raw::c_char {
    ffi_str!(|latex| { only_handout(&physics_macros(latex)) })(s)
}
/// A version of physics_macros suitable for export to C and python.
#[no_mangle]
pub extern "C" fn c_physics_macros(s: *const std::os::raw::c_char) -> *const std::os::raw::c_char {
    ffi_str!(|latex| { physics_macros(latex) })(s)
}

/// Convert some LaTeX into an HTML `String`, including figures.
#[wasm_bindgen]
#[cfg(target_arch = "wasm32")]
pub fn html_with_figures_and_solution(latex: &str, figure_directory: &str) -> String {
    set_panic_hook();
    html_string(&with_image_directory(
        &physics_macros(latex),
        figure_directory,
    ))
}

/// Convert some LaTeX into an HTML `String`, including figures.
#[wasm_bindgen]
#[cfg(target_arch = "wasm32")]
pub fn html_with_figures_omit_solution(latex: &str, figure_directory: &str) -> String {
    set_panic_hook();
    html_string(&omit_solutions(&with_image_directory(
        &physics_macros(latex),
        figure_directory,
    )))
}

fn needs_quoting_at_start(x: &str) -> Option<usize> {
    if x.len() == 0 {
        return None;
    }
    let badstuff = "<>&\"'/";
    if x.starts_with("``") || x.starts_with("''") {
        Some(2)
    } else if badstuff.contains(x.chars().next().unwrap()) {
        Some(1)
    } else {
        None
    }
}

fn find_next_quoting(x: &str) -> Option<(usize, usize)> {
    for i in 0..x.len() {
        if let Some(substr) = x.get(i..) {
            if let Some(len) = needs_quoting_at_start(substr) {
                return Some((i, i + len));
            }
        }
    }
    None
}

#[test]
fn test_find_next_quoting() {
    assert_eq!(find_next_quoting("  ''  "), Some((2, 4)));
}

/// This just does simple textual formatting
fn fmt_as_html(fmt: &mut impl std::io::Write, mut latex: &str) -> Result<(), std::io::Error> {
    if latex.contains("amp#x7b;") || latex.contains("amp#x7d;") {
        return fmt_as_html(
            fmt,
            &latex.replace("amp#x7b;", r"\{").replace("amp#x7d;", r"\}"),
        );
    }

    while let Some((start, end)) = find_next_quoting(latex) {
        fmt.write_all(latex[..start].as_bytes())?;
        let needs_quote = &latex[start..end];
        latex = &latex[end..];
        fmt.write_all(
            match needs_quote {
                // needs_quote is constrained by needs_quoting_at_start above.
                "<" => "&lt;",
                ">" => "&gt;",
                "&" => "&amp;",
                "\"" => "&quot;",
                "'" => "&#x27;",
                "/" => "&#x2f;",
                "``" => "“",
                "''" => "”",
                _ => panic!("invalid needs_quote in fmt_as_html: '{}'", needs_quote),
            }
            .as_bytes(),
        )?;
    }
    fmt.write_all(latex.as_bytes())
}

/// This just does simple textual formatting
fn fmt_math_as_html(fmt: &mut impl std::io::Write, mut latex: &str) -> Result<(), std::io::Error> {
    if latex.contains("amp#x7b;") || latex.contains("amp#x7d;") {
        return fmt_math_as_html(
            fmt,
            &latex.replace("amp#x7b;", r"\{").replace("amp#x7d;", r"\}"),
        );
    }

    while let Some((start, end)) = find_next_quoting(latex) {
        fmt.write_all(latex[..start].as_bytes())?;
        let needs_quote = &latex[start..end];
        latex = &latex[end..];
        fmt.write_all(
            match needs_quote {
                // needs_quote is constrained by needs_quoting_at_start above.
                "<" => "&lt;",
                ">" => "&gt;",
                "&" => "&amp;",
                "\"" => "&quot;",
                "'" => "&#x27;",
                "/" => "&#x2f;",
                "``" => "``",
                "''" => "''",
                _ => panic!("invalid needs_quote in fmt_as_html: '{}'", needs_quote),
            }
            .as_bytes(),
        )?;
    }
    fmt.write_all(latex.as_bytes())
}

#[test]
fn test_fmt_double_quotes() {
    let mut s: Vec<u8> = Vec::new();
    fmt_as_html(&mut s, "  ''  ").unwrap();
    assert_eq!(&String::from_utf8(s).unwrap(), "  ”  ");
    let mut s: Vec<u8> = Vec::new();
    fmt_as_html(&mut s, "  ``  ").unwrap();
    assert_eq!(&String::from_utf8(s).unwrap(), "  “  ");
}

/// This just does error formatting with html escaping
fn fmt_error(fmt: &mut impl std::io::Write, latex: &str) -> Result<(), std::io::Error> {
    fmt.write_all(br#"<span class="error">"#)?;
    fmt_as_html(fmt, latex)?;
    fmt.write_all(br#"</span>"#)
}

/// This just does error formatting with html escaping
fn fmt_errors(fmt: &mut impl std::io::Write, latex: &[&str]) -> Result<(), std::io::Error> {
    fmt.write_all(br#"<span class="error">"#)?;
    for x in latex {
        fmt_as_html(fmt, x)?;
    }
    fmt.write_all(br#"</span>"#)
}

/// Convert some LaTeX into HTML, and send the results to a `std::io::Write`.
pub fn html(fmt: &mut impl std::io::Write, latex: &str) -> Result<(), std::io::Error> {
    let latex = pull_sections_out(&strip_comments(latex));
    let mut latex: &str = &latex;
    if let Some(i) = latex.find(r"\section") {
        html_section(fmt, &latex[..i])?;
        latex = &latex[i + r"\section".len()..];
        if latex.chars().next() == Some('*') {
            latex = &latex[1..];
        }
        let title = parse_title(latex);
        latex = &latex[title.len()..];
        if title == "{" {
            fmt.write_all(br#"<span class="error">\section{</span>"#)?;
        } else {
            fmt.write_all(b"<section><h2>")?;
            html_paragraph(fmt, title)?;
            fmt.write_all(b"</h2>")?;
        }
    } else {
        html_section(fmt, latex)?;
        latex = "";
    }
    while latex.len() > 0 {
        if let Some(i) = latex.find(r"\section") {
            html_section(fmt, &latex[..i])?;
            fmt.write_all(b"</section>")?; // We finished a section.
            latex = &latex[i + r"\section".len()..];
            if latex.chars().next() == Some('*') {
                latex = &latex[1..];
            }
            let title = parse_title(latex);
            latex = &latex[title.len()..];
            if title == "{" {
                fmt.write_all(br#"<span class="error">\section{</span>"#)?;
            } else {
                fmt.write_all(b"<section><h2>")?;
                html_paragraph(fmt, title)?;
                fmt.write_all(b"</h2>")?;
            }
        } else {
            html_section(fmt, latex)?;
            fmt.write_all(b"</section>")?; // We finished a section.
            latex = "";
        }
    }
    Ok(())
}

/// Convert some LaTeX into HTML, and send the results to a `std::io::Write`.
pub fn html_section(fmt: &mut impl std::io::Write, mut latex: &str) -> Result<(), std::io::Error> {
    if let Some(i) = latex.find(r"\subsection") {
        html_subsection(fmt, &latex[..i])?;
        latex = &latex[i + r"\subsection".len()..];
        if latex.chars().next() == Some('*') {
            latex = &latex[1..];
        }
        let title = parse_title(latex);
        latex = &latex[title.len()..];
        if title == "{" {
            fmt.write_all(br#"<span class="error">\subsection{</span>"#)?;
        } else {
            fmt.write_all(b"<section><h3>")?;
            html_paragraph(fmt, title)?;
            fmt.write_all(b"</h3>")?;
        }
    } else {
        html_subsection(fmt, latex)?;
        latex = "";
    }
    while latex.len() > 0 {
        if let Some(i) = latex.find(r"\subsection") {
            html_subsection(fmt, &latex[..i])?;
            fmt.write_all(b"</section>")?; // We finished a section.
            latex = &latex[i + r"\subsection".len()..];
            if latex.chars().next() == Some('*') {
                latex = &latex[1..];
            }
            let title = parse_title(latex);
            latex = &latex[title.len()..];
            if title == "{" {
                fmt.write_all(br#"<span class="error">\subsection{</span>"#)?;
            } else {
                fmt.write_all(b"<section><h3>")?;
                html_paragraph(fmt, title)?;
                fmt.write_all(b"</h3>")?;
            }
        } else {
            html_subsection(fmt, latex)?;
            fmt.write_all(b"</section>")?; // We finished a section.
            latex = "";
        }
    }
    Ok(())
}

fn process_url_argument(url: &str) -> String {
    let mut url = url.to_string();
    url = url.replace('{', "");
    url = url.replace('}', "");
    url
}

/// Convert some LaTeX into HTML, and send the results to a `std::io::Write`.
pub fn html_subsection(
    fmt: &mut impl std::io::Write,
    mut latex: &str,
) -> Result<(), std::io::Error> {
    if let Some(i) = latex.find(r"\subsubsection") {
        html_subsubsection(fmt, &latex[..i])?;
        latex = &latex[i + r"\subsubsection".len()..];
        if latex.chars().next() == Some('*') {
            latex = &latex[1..];
        }
        let title = parse_title(latex);
        latex = &latex[title.len()..];
        if title == "{" {
            fmt.write_all(br#"<span class="error">\subsubsection{</span>"#)?;
        } else {
            fmt.write_all(b"<section><h4>")?;
            html_paragraph(fmt, title)?;
            fmt.write_all(b"</h4>")?;
        }
    } else {
        html_subsubsection(fmt, latex)?;
        latex = "";
    }
    while latex.len() > 0 {
        if let Some(i) = latex.find(r"\subsubsection") {
            html_subsubsection(fmt, &latex[..i])?;
            fmt.write_all(b"</section>")?; // We finished a section.
            latex = &latex[i + r"\subsubsection".len()..];
            if latex.chars().next() == Some('*') {
                latex = &latex[1..];
            }
            let title = parse_title(latex);
            latex = &latex[title.len()..];
            if title == "{" {
                fmt.write_all(br#"<span class="error">\subsubsection{</span>"#)?;
            } else {
                fmt.write_all(b"<section><h4>")?;
                html_paragraph(fmt, title)?;
                fmt.write_all(b"</h4>")?;
            }
        } else {
            html_subsubsection(fmt, latex)?;
            fmt.write_all(b"</section>")?; // We finished a section.
            latex = "";
        }
    }
    Ok(())
}

/// Convert some LaTeX into HTML, and send the results to a `std::io::Write`.
pub fn html_subsubsection(
    fmt: &mut impl std::io::Write,
    mut latex: &str,
) -> Result<(), std::io::Error> {
    let am_alone = finish_paragraph(latex).len() == latex.len();
    loop {
        let p = finish_paragraph(latex);
        latex = &latex[p.len()..];
        if p.len() == 0 {
            return Ok(());
        }
        if p.trim().len() == 0 {
            continue;
        }
        if !am_alone {
            fmt.write_all(b"<p>")?;
        }
        html_paragraph(fmt, p)?;
        if !am_alone {
            fmt.write_all(b"</p>")?;
        }
    }
}

/// Convert some LaTeX into HTML, and send the results to a `std::io::Write`.
pub fn html_paragraph(fmt: &mut impl std::io::Write, latex: &str) -> Result<(), std::io::Error> {
    let mut latex = latex;
    let latex_sans_stuff: String;
    if latex.contains(r"\{") || latex.contains(r"\}") {
        latex_sans_stuff = latex.replace(r"\{", "amp#x7b;").replace(r"\}", "amp#x7d;");
        latex = &latex_sans_stuff;
    }
    let math_environs = &[
        "{equation}",
        "{equation*}",
        "{align}",
        "{align*}",
        "{eqnarray}",
        "{eqnarray*}",
        "{multline}",
        "{multline*}",
    ];
    loop {
        if latex.len() == 0 {
            return Ok(());
        }
        if let Some(i) = latex.find(|c| c == '~' || c == '\\' || c == '{' || c == '$') {
            fmt_as_html(fmt, &latex[..i])?;
            latex = &latex[i..];
            let c = latex.chars().next().unwrap();
            if c == '~' {
                latex = &latex[1..];
                fmt.write_all(b"&nbsp;")?;
            } else if c == '\\' {
                let name = macro_name(latex);
                latex = &latex[name.len()..];
                match name {
                    r"\\" => {
                        fmt.write_all(b"<br/>")?;
                    }
                    r"\newpage" => {
                        fmt.write_all(b"<br/>")?;
                    }
                    r"\vspace" | r"\vfill" => {
                        let arg = argument(latex);
                        latex = &latex[arg.len()..];
                        if arg == "{" {
                            fmt.write_all(br#"<span class="error">\vspace{</span>"#)?;
                        } else {
                            fmt.write_all(b"<br/>")?; // just treat a \vspace as a line break
                        }
                    }
                    r"\textbackslash" => {
                        fmt.write_all(b"\\")?;
                    }
                    r"\label" => {
                        let arg = argument(latex);
                        latex = &latex[arg.len()..];
                        if arg == "{" {
                            fmt.write_all(br#"<span class="error">\label{</span>"#)?;
                        } else {
                            write!(fmt, r#"<span class="error">\label{}</span>"#, arg)?;
                        }
                    }
                    // r"\ref" => {
                    //     let arg = argument(latex);
                    //     latex = &latex[arg.len()..];
                    //     if arg == "{" {
                    //         fmt.write_all(br#"<span class="error">\ref{</span>"#)?;
                    //     } else {
                    //         write!(fmt, r##"<a class="ref" href="#{}">{}</a>"##, arg, arg)?;
                    //     }
                    // }
                    r"\eqref" => {
                        let arg = argument(latex);
                        latex = &latex[arg.len()..];
                        if arg == "{" {
                            fmt.write_all(br#"<span class="error">\eqref{</span>"#)?;
                        } else {
                            fmt.write_all(br"\eqref")?;
                            fmt.write_all(arg.as_bytes())?;
                        }
                    }
                    r"\ref" => {
                        let arg = argument(latex);
                        latex = &latex[arg.len()..];
                        if arg == "{" {
                            fmt.write_all(br#"<span class="error">\ref{</span>"#)?;
                        } else {
                            fmt.write_all(br"\ref")?;
                            fmt.write_all(arg.as_bytes())?;
                        }
                    }
                    r"\emph" => {
                        let arg = argument(latex);
                        latex = &latex[arg.len()..];
                        if arg == "{" {
                            fmt.write_all(br#"<span class="error">\emph{</span>"#)?;
                        } else {
                            fmt.write_all(b"<em>")?;
                            html_subsubsection(fmt, arg)?;
                            fmt.write_all(b"</em>")?;
                        }
                    }
                    r"\underline" => {
                        let arg = argument(latex);
                        latex = &latex[arg.len()..];
                        if arg == "{" {
                            fmt.write_all(br#"<span class="error">\underline{</span>"#)?;
                        } else {
                            fmt.write_all(b"<u>")?;
                            html_subsubsection(fmt, arg)?;
                            fmt.write_all(b"</u>")?;
                        }
                    }
                    r"\textit" => {
                        let arg = argument(latex);
                        latex = &latex[arg.len()..];
                        if arg == "{" {
                            fmt.write_all(br#"<span class="error">\textit{</span>"#)?;
                        } else {
                            fmt.write_all(b"<i>")?;
                            html_subsubsection(fmt, arg)?;
                            fmt.write_all(b"</i>")?;
                        }
                    }
                    r"\textcolor" => {
                        let color = argument(latex);
                        latex = &latex[color.len()..];
                        if color == "{" {
                            fmt.write_all(br#"<span class="error">\textcolor{</span>"#)?;
                        } else {
                            let color = color.replace("{", "").replace("}", "");
                            let arg = argument(latex);
                            latex = &latex[arg.len()..];
                            if arg == "{" {
                                fmt.write_all(br#"<span class="error">\textcolor{"#)?;
                                fmt.write_all(color.as_bytes())?;
                                fmt.write_all(br#"}{</span>"#)?;
                            } else {
                                if [
                                    "red",
                                    "blue",
                                    "forestgreen",
                                    "purple",
                                    "brown",
                                    "gray",
                                    "orange",
                                ]
                                .contains(&color.as_ref())
                                {
                                    fmt.write_all(br#"<span style="color:"#)?;
                                    fmt.write_all(color.as_bytes())?;
                                    fmt.write_all(br#";">"#)?;
                                    html_subsubsection(fmt, arg)?;
                                    fmt.write_all(b"</span>")?;
                                } else {
                                    fmt.write_all(
                                        br#"<span class="error">\textcolor{Invalid color "#,
                                    )?;
                                    fmt.write_all(color.as_bytes())?;
                                    fmt.write_all(br#" Allowed colors: red, blue, forestgreen, purple, gray, brown}</span>"#)?;
                                    html_subsubsection(fmt, arg)?;
                                }
                            }
                        }
                    }
                    r"\footnote" => {
                        let arg = argument(latex);
                        latex = &latex[arg.len()..];
                        if arg == "{" {
                            fmt.write_all(br#"<span class="error">\footnote{</span>"#)?;
                        } else {
                            fmt.write_all(b"<sup>*</sup><aside><sup>*</sup>")?;
                            html_subsubsection(fmt, arg)?;
                            fmt.write_all(b"</aside>")?;
                        }
                    }
                    r"\textbf" => {
                        let arg = argument(latex);
                        latex = &latex[arg.len()..];
                        if arg == "{" {
                            fmt.write_all(br#"<span class="error">\textbf{</span>"#)?;
                        } else {
                            fmt.write_all(b"<b>")?;
                            html_subsubsection(fmt, arg)?;
                            fmt.write_all(b"</b>")?;
                        }
                    }
                    r"\url" => {
                        let arg = argument(latex); // strip of {} create function to validate url
                        let url = process_url_argument(arg);
                        latex = &latex[arg.len()..];
                        if arg == "{" {
                            fmt.write_all(
                                br#"<span class="error" data-error="bad-argument">\url{</span>"#,
                            )?;
                        } else {
                            if url.starts_with("https://") {
                                fmt.write_all(b"<a target=\"_parent\" href=\"")?;
                            } else {
                                fmt.write_all(b"<a href=\"")?;
                            }
                            fmt.write_all(url.as_bytes())?;
                            fmt.write_all(b"\">")?;
                            fmt.write_all(url.as_bytes())?;
                            fmt.write_all(b"</a>")?;
                        }
                    }
                    r"\href" => {
                        let url_arg = argument(latex); // strip of {}
                        latex = &latex[url_arg.len()..];
                        let url = process_url_argument(url_arg);
                        if url == "{" {
                            fmt.write_all(br#"<span class="error">\href{</span>"#)?;
                        } else {
                            let arg = argument(latex); // strip of {}
                            latex = &latex[arg.len()..];
                            if arg == "{" {
                                fmt.write_all(br#"<span class="error">\href{"#)?;
                                fmt.write_all(url.as_bytes())?;
                                fmt.write_all(br#"}{</span>"#)?;
                            } else {
                                if url.starts_with("https://") {
                                    fmt.write_all(b"<a target=\"_parent\" href=\"")?;
                                } else {
                                    fmt.write_all(b"<a href=\"")?;
                                }
                                fmt.write_all(url.as_bytes())?;
                                fmt.write_all(b"\">")?;
                                html_subsubsection(fmt, arg)?;
                                fmt.write_all(b"</a>")?;
                            }
                        }
                    }
                    r"\includegraphics" => {
                        let opt = optional_argument(latex);
                        let width = parse_width(opt);
                        latex = &latex[opt.len()..];
                        let arg = argument(latex);
                        latex = &latex[arg.len()..];
                        if arg == "{" {
                            fmt.write_all(br#"<span class="error">\includegraphics{</span>"#)?;
                        } else {
                            fmt.write_all(br#"<img"#)?;
                            fmt.write_all(width.as_bytes())?;
                            fmt.write_all(br#" src=""#)?;
                            fmt_as_html(fmt, &arg[1..arg.len() - 1])?;
                            fmt.write_all(br#""/>"#)?;
                        }
                    }
                    r"\caption" => {
                        let arg = argument(latex);
                        latex = &latex[arg.len()..];
                        if arg == "{" {
                            fmt.write_all(br#"<span class="error">\caption{</span>"#)?;
                        } else {
                            fmt.write_all(b"<figcaption>")?;
                            html_subsubsection(fmt, arg)?;
                            fmt.write_all(b"</figcaption>")?;
                        }
                    }
                    r"\warning" => {
                        let arg = argument(latex);
                        latex = &latex[arg.len()..];
                        if arg == "{" {
                            fmt.write_all(br#"<span class="error">\warning{</span>"#)?;
                        } else {
                            fmt.write_all(br#"<span class="warning">"#)?;
                            html(fmt, arg)?;
                            fmt.write_all(b"</span>")?;
                        }
                    }
                    r"\error" => {
                        let arg = argument(latex);
                        latex = &latex[arg.len()..];
                        if arg == "{" {
                            fmt.write_all(br#"<span class="error">\error{</span>"#)?;
                        } else {
                            fmt.write_all(br#"<span> class="error">"#)?;
                            html(fmt, arg)?;
                            fmt.write_all(b"</span>")?;
                        }
                    }
                    r"\paragraph" | r"\paragraph*" => {
                        let arg = parse_title(latex);
                        latex = latex[arg.len()..].trim_start();
                        if arg == "{" {
                            fmt.write_all(br#"<span class="error">\paragraph{</span>"#)?;
                        } else {
                            fmt.write_all(b"<h5>")?;
                            html(fmt, arg)?;
                            fmt.write_all(b"</h5>")?;
                        }
                    }
                    r"\it" => {
                        latex = finish_standalone_macro(latex);
                        fmt.write_all(b"<i>")?;
                        html_subsubsection(fmt, latex)?;
                        return fmt.write_all(b"</i>");
                    }
                    r"\centering" => {
                        latex = finish_standalone_macro(latex);
                        fmt.write_all(br#"<div class="center">"#)?;
                        html_subsubsection(fmt, latex)?;
                        return fmt.write_all(b"</div>");
                    }
                    r"\ " => {
                        fmt.write_all(b" ")?;
                    }
                    r"\noindent" => {
                        // Nothing to do?
                    }
                    r"\%" => {
                        fmt.write_all(b"%")?;
                    }
                    r"\#" => {
                        fmt.write_all(b"#")?;
                    }
                    r"\$" => {
                        fmt.write_all(br"<span>$</span>")?;
                    }
                    r"\&" => {
                        fmt.write_all(b"&amp;")?;
                    }
                    r"\_" => {
                        fmt.write_all(b"_")?;
                    }
                    r"\(" => {
                        if let Some(i) = latex.find(r"\)") {
                            fmt.write_all(br"\(")?;
                            fmt_math_as_html(fmt, &latex[..i + 2])?;
                            latex = &latex[i + 2..];
                        } else {
                            fmt_error(fmt, r"\(")?;
                        }
                    }
                    r"\[" => {
                        if let Some(i) = latex.find(r"\]") {
                            fmt.write_all(br"\[")?;
                            fmt_math_as_html(fmt, &latex[..i + 2])?;
                            latex = &latex[i + 2..];
                        } else {
                            fmt_error(fmt, r"\(")?;
                        }
                    }
                    r"\bf" => {
                        latex = finish_standalone_macro(latex);
                        fmt.write_all(b"<b>")?;
                        html(fmt, latex)?;
                        return fmt.write_all(b"</b>");
                    }
                    r"\sc" => {
                        latex = finish_standalone_macro(latex);
                        fmt.write_all(br#"<font style="font-variant: small-caps">"#)?;
                        html(fmt, latex)?;
                        return fmt.write_all(b"</font>");
                    }
                    r"\begin" => {
                        // We are looking at an environment...
                        let name = env_name(latex);
                        latex = &latex[name.len()..];
                        if name.chars().last() != Some('}') {
                            fmt_errors(fmt, &[r"\begin", name])?;
                        } else if name == "{figure}" {
                            // Just skip any figure placement parameters
                            if latex.chars().next().unwrap() == '[' {
                                if let Some(i) = latex.find(']') {
                                    latex = &latex[i + 1..];
                                }
                            }
                            if let Some(i) = latex.find(r"\end{figure}") {
                                if latex.starts_with(r"\centering ")
                                    || latex.starts_with(r"\centering\n")
                                {
                                    fmt.write_all(br#"<figure class="center">"#)?;
                                    html_paragraph(fmt, &latex[r"\centering ".len()..i])?;
                                } else {
                                    fmt.write_all(b"<figure>")?;
                                    html_paragraph(fmt, &latex[..i])?;
                                }
                                fmt.write_all(b"</figure>")?;
                                latex = &latex[i + br"\end{figure}".len()..];
                            } else {
                                fmt.write_all(br#"<span class="error">\begin{figure}</span>"#)?;
                            }
                        } else if name == "{wrapfigure}" {
                            let align = argument(latex);
                            latex = &latex[align.len()..];
                            let width = argument(latex);
                            latex = &latex[width.len()..];
                            let width = parse_width(width);
                            if let Some(i) = latex.find(r"\end{wrapfigure}") {
                                if latex.starts_with(r"\centering ")
                                    || latex.starts_with(r"\centering\n")
                                {
                                    fmt.write_all(br#"<figure class="wrapfigure center""#)?;
                                    fmt.write_all(width.as_bytes())?;
                                    fmt.write_all(b">")?;
                                    html_paragraph(fmt, &latex[r"\centering ".len()..i])?;
                                } else {
                                    fmt.write_all(br#"<figure class="wrapfigure""#)?;
                                    fmt.write_all(width.as_bytes())?;
                                    fmt.write_all(b">")?;
                                    html_paragraph(fmt, &latex[..i])?;
                                }
                                fmt.write_all(b"</figure>")?;
                                latex = &latex[i + br"\end{wrapfigure}".len()..];
                            } else {
                                fmt.write_all(br#"<span class="error">\begin{wrapfigure}</span>"#)?;
                            }
                        } else if ["{solution}", "{guide}", "{handout}"].contains(&name) {
                            let end = format!(r"\end{}", name);
                            if let Some(i) = latex.find(&end) {
                                if latex[..i].chars().all(|c| c.is_whitespace()) {
                                    // Nothing to do here, this solution is empty
                                } else {
                                    let kind = &name[1..name.len() - 1];
                                    fmt.write_all(br#"<blockquote class=""#)?;
                                    fmt.write_all(kind.as_bytes())?;
                                    fmt.write_all(br#"">"#)?;
                                    html_subsubsection(fmt, &latex[..i])?;
                                    fmt.write_all(b"</blockquote>")?;
                                }
                                latex = &latex[i + br"\end".len() + name.len()..];
                            } else {
                                fmt.write_all(br#"<span class="error">\begin"#)?;
                                fmt.write_all(name.as_bytes())?;
                                fmt.write_all(br#"</span>"#)?;
                            }
                        } else if name == "{tabular}" {
                            if let Some(i) = latex.find(r"\end{tabular}") {
                                let rest = &latex[i + br"\end{tabular}".len()..];
                                let arg = argument(latex);
                                latex = &latex[arg.len()..i];
                                // We just ignore the alignment marks.
                                fmt.write_all(br#"<table>"#)?;
                                for row in latex.split(r"\\") {
                                    let row = row.replace(r"\&", "myampersand");
                                    fmt.write_all(br#"<tr>"#)?;
                                    for column in row.split("&") {
                                        fmt.write_all(br#"<td>"#)?;
                                        let column = column.replace("myampersand", r"\&");
                                        html_paragraph(fmt, &column)?;
                                        fmt.write_all(br#"</td>"#)?;
                                    }
                                    fmt.write_all(br#"</tr>"#)?;
                                }
                                fmt.write_all(b"</table>")?;
                                latex = rest;
                            } else {
                                fmt.write_all(br#"<span class="error">\begin{tabular}</span>"#)?;
                            }
                        } else if name == "{center}" {
                            if let Some(i) = latex.find(r"\end{center}") {
                                fmt.write_all(br#"<div class="center">"#)?;
                                html_paragraph(fmt, &latex[..i])?;
                                fmt.write_all(b"</div>")?;
                                latex = &latex[i + br"\end{center}".len()..];
                            } else {
                                fmt.write_all(br#"<span class="error">\begin{center}</span>"#)?;
                            }
                        } else if name == "{quote}" {
                            if let Some(i) = latex.find(r"\end{quote}") {
                                fmt.write_all(b"<blockquote>")?;
                                html_paragraph(fmt, &latex[..i])?;
                                fmt.write_all(b"</blockquote>")?;
                                latex = &latex[i + br"\end{quote}".len()..];
                            } else {
                                fmt.write_all(br#"<span class="error">\begin{quote}</span>"#)?;
                            }
                        } else if name == "{quotation}" {
                            if let Some(i) = latex.find(r"\end{quotation}") {
                                fmt.write_all(b"<blockquote>")?;
                                html_paragraph(fmt, &latex[..i])?;
                                fmt.write_all(b"</blockquote>")?;
                                latex = &latex[i + br"\end{quotation}".len()..];
                            } else {
                                fmt.write_all(br#"<span class="error">\begin{quotation}</span>"#)?;
                            }
                        } else if math_environs.contains(&name) {
                            let env = end_env(name, latex);
                            latex = &latex[env.len()..];
                            if env == "" {
                                fmt_errors(fmt, &[r"\begin", name])?;
                            } else {
                                fmt.write_all(br#"\begin"#)?;
                                fmt_as_html(fmt, name)?;
                                fmt_math_as_html(fmt, env)?;
                            }
                        } else if name == "{itemize}" {
                            fmt.write_all(b"<ul>")?;
                            let li = finish_item(latex);
                            latex = &latex[li.len()..];
                            if li.trim().len() > 0 {
                                // Nothing should precede the first
                                // \item except whitespace.
                                fmt_error(fmt, li)?;
                            }
                            loop {
                                let li = finish_item(latex);
                                latex = &latex[li.len()..];
                                if li.len() == 0 {
                                    if latex.starts_with(r"\end{itemize}") {
                                        latex = &latex[r"\end{itemize}".len()..];
                                        fmt.write_all(b"</ul>")?;
                                        break;
                                    } else if latex.starts_with(r"\end{description}") {
                                        latex = &latex[r"\end{description}".len()..];
                                        fmt.write_all(
                                            br#"</ol><span class="error">\end{enumerate}</span>"#,
                                        )?;
                                        break;
                                    } else if latex.starts_with(r"\end{enumerate}") {
                                        latex = &latex[r"\end{enumerate}".len()..];
                                        fmt.write_all(
                                            br#"</ul><span class="error">\end{enumerate}</span>"#,
                                        )?;
                                        break;
                                    } else if latex.starts_with(r"\item") {
                                        // It must start with \item
                                        latex = &latex[r"\item".len()..];
                                        latex = finish_standalone_macro(latex);
                                    } else {
                                        fmt.write_all(
                                            br#"</ul><span class="error">MISSING \end{itemize}</span>"#,
                                        )?;
                                        break;
                                    }
                                } else {
                                    fmt.write_all(b"<li>")?;
                                    html(fmt, li)?;
                                    fmt.write_all(b"</li>")?;
                                }
                            }
                        } else if name == "{enumerate}" {
                            fmt.write_all(b"<ol>")?;
                            let li = finish_item(latex);
                            latex = &latex[li.len()..];
                            if li.trim().len() > 0 {
                                // Nothing should precede the first
                                // \item except whitespace.
                                fmt_error(fmt, li)?;
                            }
                            loop {
                                let li = finish_item(latex);
                                latex = &latex[li.len()..];
                                if li.len() == 0 {
                                    if latex.starts_with(r"\end{itemize}") {
                                        latex = &latex[r"\end{itemize}".len()..];
                                        fmt.write_all(
                                            br#"</ol><span class="error">\end{enumerate}</span>"#,
                                        )?;
                                        break;
                                    } else if latex.starts_with(r"\end{description}") {
                                        latex = &latex[r"\end{description}".len()..];
                                        fmt.write_all(
                                            br#"</ol><span class="error">\end{enumerate}</span>"#,
                                        )?;
                                        break;
                                    } else if latex.starts_with(r"\end{enumerate}") {
                                        latex = &latex[r"\end{enumerate}".len()..];
                                        fmt.write_all(b"</ol>")?;
                                        break;
                                    } else if latex.starts_with(r"\item") {
                                        // It must start with \item
                                        latex = &latex[r"\item".len()..];
                                        latex = finish_standalone_macro(latex);
                                    } else {
                                        fmt.write_all(
                                            br#"</ol><span class="error">MISSING \end{enumerate}</span>"#,
                                        )?;
                                        break;
                                    }
                                } else {
                                    fmt.write_all(b"<li>")?;
                                    html(fmt, li)?;
                                    fmt.write_all(b"</li>")?;
                                }
                            }
                        } else if name == "{description}" {
                            fmt.write_all(b"<dl>")?;
                            let li = finish_item(latex);
                            latex = &latex[li.len()..];
                            if li.trim().len() > 0 {
                                // Nothing should precede the first
                                // \item except whitespace.
                                fmt_error(fmt, li)?;
                            }
                            loop {
                                let li = finish_item(latex);
                                latex = &latex[li.len()..];
                                if li.len() == 0 {
                                    if latex.starts_with(r"\end{itemize}") {
                                        latex = &latex[r"\end{itemize}".len()..];
                                        fmt.write_all(
                                            br#"</ol><span class="error">\end{description}</span>"#,
                                        )?;
                                        break;
                                    } else if latex.starts_with(r"\end{enumerate}") {
                                        latex = &latex[r"\end{enumerate}".len()..];
                                        fmt.write_all(
                                            br#"</dl><span class="error">\end{description}</span>"#,
                                        )?;
                                        break;
                                    } else if latex.starts_with(r"\end{description}") {
                                        latex = &latex[r"\end{description}".len()..];
                                        fmt.write_all(b"</dl>")?;
                                        break;
                                    } else if latex.starts_with(r"\item") {
                                        // It must start with \item
                                        latex = &latex[r"\item".len()..];
                                        latex = finish_standalone_macro(latex);
                                    } else {
                                        fmt.write_all(
                                            br#"</dl><span class="error">MISSING \end{description}</span>"#,
                                        )?;
                                        break;
                                    }
                                } else {
                                    let o = optional_argument(li);
                                    let li = &li[o.len()..];
                                    if o.len() > 2 {
                                        fmt.write_all(b"<dt>")?;
                                        html(fmt, &o[1..o.len() - 1])?;
                                        fmt.write_all(b"</dt>")?;
                                    }

                                    fmt.write_all(b"<dd>")?;
                                    html(fmt, li)?;
                                    fmt.write_all(b"</dd>")?;
                                }
                            }
                        } else {
                            let env = end_env(name, latex);
                            latex = &latex[env.len()..];
                            fmt_errors(fmt, &[r"\begin", name, env])?;
                        }
                    }
                    r"\end" => {
                        let name = env_name(latex);
                        latex = &latex[name.len()..];
                        fmt_errors(fmt, &[r"\end", name])?;
                    }
                    _ => {
                        fmt_error(fmt, name)?;
                    }
                }
            } else if c == '$' {
                if let Some(i) = latex[1..].find('$') {
                    if i == 0 {
                        // It is a $$ actually
                        if let Some(i) = latex[2..].find("$$") {
                            fmt.write_all(br"\[")?;
                            fmt_math_as_html(fmt, &latex[2..i + 2])?;
                            fmt.write_all(br"\]")?;
                            latex = &latex[i + 4..];
                        } else {
                            fmt.write_all(br#"<span class="error">$$</span>"#)?;
                            latex = &latex[2..];
                        }
                    } else {
                        fmt.write_all(br"\(")?;
                        fmt_math_as_html(fmt, &latex[1..i + 1])?;
                        fmt.write_all(br"\)")?;
                        latex = &latex[i + 2..];
                    }
                } else {
                    fmt.write_all(br#"<span class="error">$</span>"#)?;
                    latex = &latex[1..];
                }
            } else {
                let arg = argument(latex);
                latex = &latex[arg.len()..];
                if arg == "{" {
                    fmt.write_all(br#"<span class="error">{</span>"#)?;
                } else {
                    html(fmt, &arg[1..arg.len() - 1])?;
                }
            }
        } else {
            return fmt_as_html(fmt, latex);
        }
    }
}

fn finish_standalone_macro(latex: &str) -> &str {
    if latex.len() == 0 {
        ""
    } else if latex.chars().next().unwrap() == ' ' {
        &latex[1..]
    } else {
        latex
    }
}

fn macro_name(latex: &str) -> &str {
    if let Some(i) = latex[1..].find(|c: char| !c.is_alphabetic() && c != '*') {
        if i == 0 {
            &latex[..2]
        } else {
            &latex[..i + 1]
        }
    } else {
        latex
    }
}

#[test]
fn test_macro_name() {
    assert_eq!(macro_name(r"\emph{foo"), r"\emph");
    assert_eq!(macro_name(r"\\ extra"), r"\\");
    assert_eq!(macro_name(r"\% extra"), r"\%");
}

fn env_name(latex: &str) -> &str {
    if let Some(i) = latex.find(|c: char| c == '}') {
        &latex[..i + 1]
    } else if let Some(i) = latex.find(|c: char| c != '{' && !c.is_alphabetic()) {
        &latex[..i + 1]
    } else {
        latex
    }
}

fn end_env<'a>(name: &str, latex: &'a str) -> &'a str {
    let end = format!(r"\end{}", name);
    if let Some(i) = latex.find(&end) {
        &latex[..i + end.len()]
    } else {
        ""
    }
}

fn earlier(a: Option<usize>, b: Option<usize>) -> bool {
    if let Some(b) = b {
        if let Some(a) = a {
            a < b
        } else {
            false
        }
    } else {
        true
    }
}

fn find_paragraph(latex: &str) -> Option<usize> {
    let paragraph = regex::Regex::new("\n\\s*\n").unwrap();
    paragraph.find(latex).map(|m| m.end())
}

#[test]
fn test_find_paragraph() {
    assert_eq!(Some(3), find_paragraph("\n\n\nHello world"));
    assert_eq!(Some(5), find_paragraph("\n\n\n\r\nHello world"));
}
#[test]
fn test_finish_paragraph() {
    assert_eq!(
        r"\begin{center}
contents
\end{center}

",
        finish_paragraph(
            r"\begin{center}
contents
\end{center}

"
        )
    );
    assert_eq!("\n\n\n", finish_paragraph("\n\n\nHello world"));
    assert_eq!("\n\n\n\r\n", finish_paragraph("\n\n\n\r\nHello world"));
    assert_eq!(
        "\nFirst me\n\n\n\r\n",
        finish_paragraph("\nFirst me\n\n\n\r\nHello world")
    );
}

fn finish_paragraph(latex: &str) -> &str {
    if latex.len() == 0 {
        return "";
    }
    let mut so_far = 0;
    let mut nestedness = 0;
    loop {
        let next_paragraph = find_paragraph(&latex[so_far..]);
        let next_end = latex[so_far..].find(r"\end{");
        let next_begin = latex[so_far..].find(r"\begin{");
        if earlier(next_paragraph, next_begin) && earlier(next_paragraph, next_end) {
            if nestedness == 0 {
                if let Some(i) = next_paragraph {
                    return &latex[..so_far + i];
                } else {
                    // There is no end to this
                    return latex;
                }
            } else {
                return latex;
            }
        } else if earlier(next_end, next_begin) {
            let i = next_end.unwrap();
            if nestedness == 0 {
                return &latex[..so_far + i];
            } else {
                nestedness -= 1;
                so_far += i + r"\\end{".len();
            }
        } else {
            let i = next_begin.unwrap();
            nestedness += 1;
            so_far += i + r"\\begin{".len();
        }
    }
}

fn finish_item(latex: &str) -> &str {
    if latex.len() == 0 {
        return "";
    }
    let end_list = regex::Regex::new(r"\\end\{(itemize|enumerate|description)\}").unwrap();
    let begin_list = regex::Regex::new(r"\\begin\{(itemize|enumerate|description)\}").unwrap();
    let mut so_far = 0;
    let mut nestedness = 0;
    loop {
        let next_item = latex[so_far..].find(r"\item");
        let next_end = end_list.find(&latex[so_far..]).map(|m| m.start());
        let next_begin = begin_list.find(&latex[so_far..]).map(|m| m.start());
        if nestedness == 0 && earlier(next_item, next_begin) && earlier(next_item, next_end) {
            if let Some(i) = next_item {
                return &latex[..so_far + i];
            } else {
                // There is no end to this
                return "";
            }
        } else if earlier(next_end, next_begin) {
            if let Some(i) = next_end {
                if nestedness == 0 {
                    return &latex[..so_far + i];
                } else {
                    nestedness -= 1;
                    so_far += i + r"\\end{".len();
                }
            } else {
                // There is no ending, but we are nested!!!
                return "";
            }
        } else {
            if let Some(i) = next_begin {
                nestedness += 1;
                so_far += i + r"\\begin{".len();
            } else {
                panic!("next_begin gives unexpected None");
            }
        }
    }
}

fn argument(latex: &str) -> &str {
    if latex.len() == 0 {
        ""
    } else if latex.chars().next().unwrap().is_digit(10) {
        &latex[..1]
    } else if latex.chars().next().unwrap() == '{' {
        let mut n = 0;
        let mut arg = String::from("{");
        for c in latex[1..].chars() {
            arg.push(c);
            if c == '{' {
                n += 1
            } else if c == '}' {
                if n == 0 {
                    return &latex[..arg.len()];
                }
                n -= 1
            }
        }
        // we must have unbalanced parentheses
        &latex[..1]
    } else {
        &latex[..1]
    }
}

fn optional_argument(latex: &str) -> &str {
    if latex.len() == 0 {
        ""
    } else if latex.chars().next().unwrap() == '[' {
        let mut n: isize = 0;
        let mut arg = String::from("[");
        for c in latex[1..].chars() {
            arg.push(c);
            if c == '{' {
                n += 1
            } else if c == '}' {
                n -= 1
            } else if c == ']' && n == 0 {
                return &latex[..arg.len()];
            }
        }
        // we must have unbalanced parentheses
        &latex[..1]
    } else {
        ""
    }
}

/// Returns the class to be used
fn parse_width(option: &str) -> String {
    let em = regex::Regex::new(r"[\[\{]width=(\d+)(.+)[\}\]]").unwrap();
    let other = regex::Regex::new(r"[\[\{](\d+)(.+)[\}\]]").unwrap();
    if let Some(c) = em.captures(option).or(other.captures(option)) {
        let value = c.get(1).unwrap().as_str();
        let units = c.get(2).unwrap().as_str();
        match units {
            "em" | "ex" | "cm" | "in" | "mm" | "pt" => {
                return format!(r#" style="width:{}{}""#, value, units);
            }
            _ => (),
        }
    }
    return "".to_string();
}

fn parse_title(latex: &str) -> &str {
    if latex.len() == 0 {
        return "";
    }
    if latex.chars().next().unwrap().is_digit(10) {
        &latex[..1]
    } else if latex.chars().next().unwrap() == '{' {
        let mut n = 0;
        let mut arg = String::from("{");
        for c in latex[1..].chars() {
            arg.push(c);
            if c == '{' {
                n += 1
            } else if c == '}' {
                if n == 0 {
                    return &latex[..arg.len()];
                }
                n -= 1
            }
        }
        // we must have unbalanced parentheses
        &latex[..1]
    } else {
        &latex[..1]
    }
}

#[test]
fn test_argument() {
    assert_eq!(argument(r"{foo"), r"{");
    assert_eq!(argument(r"{foo}  "), r"{foo}");
}

/// Substitute five physics macros
pub fn physics_macros(latex: &str) -> String {
    let mut latex = latex; // this makes the lifetime local to the function
    let mut refined = String::with_capacity(latex.len());
    while let Some(i) = latex.find(r"\ket{") {
        refined.push_str(&latex[..i]);
        latex = &latex[i + r"\ket".len()..];
        let arg = argument(latex);
        latex = &latex[arg.len()..];
        refined.push_str(r"\left|");
        refined.push_str(&physics_macros(arg));
        refined.push_str(r"\right\rangle ");
    }
    refined.push_str(latex);
    latex = &refined;

    let mut refined = String::with_capacity(latex.len());
    while let Some(i) = latex.find(r"\bra{") {
        refined.push_str(&latex[..i]);
        latex = &latex[i + r"\bra".len()..];
        let arg = argument(latex);
        latex = &latex[arg.len()..];
        refined.push_str(r"\left\langle ");
        refined.push_str(&physics_macros(arg));
        refined.push_str(r"\right|");
    }
    refined.push_str(latex);
    latex = &refined;

    let mut refined = String::with_capacity(latex.len());
    while let Some(i) = latex.find(r"\right|\left|") {
        refined.push_str(&latex[..i]);
        latex = &latex[i + r"\right|\left|".len()..];
        refined.push_str(r"\middle|");
    }
    refined.push_str(latex);
    latex = &refined;

    let mut refined = String::with_capacity(latex.len());
    while let Some(i) = latex.find(r"\dbar ") {
        refined.push_str(&latex[..i]);
        latex = &latex[i + r"\dbar".len()..];
        refined.push_str(r"{d\hspace{-0.08em}\bar{}\hspace{0.1em}}");
    }
    refined.push_str(latex);
    latex = &refined;

    let mut refined = String::with_capacity(latex.len());
    while let Some(i) = latex.find(r"\myderiv{") {
        refined.push_str(&latex[..i]);
        latex = &latex[i + r"\myderiv".len()..];
        let arg1 = argument(latex);
        latex = &latex[arg1.len()..];
        let arg2 = argument(latex);
        latex = &latex[arg2.len()..];
        let arg3 = argument(latex);
        latex = &latex[arg3.len()..];
        refined.push_str(r"\left(\frac{\partial ");
        refined.push_str(&physics_macros(arg1));
        refined.push_str(r"}{\partial ");
        refined.push_str(&physics_macros(arg2));
        refined.push_str(r"}\right)_");
        refined.push_str(&physics_macros(arg3));
    }
    refined.push_str(latex);
    latex = &refined;

    let mut refined = String::with_capacity(latex.len());
    while let Some(i) = latex.find(r"\thermoderivative{") {
        refined.push_str(&latex[..i]);
        latex = &latex[i + r"\thermoderivative".len()..];
        let arg1 = argument(latex);
        latex = &latex[arg1.len()..];
        let arg2 = argument(latex);
        latex = &latex[arg2.len()..];
        let arg3 = argument(latex);
        latex = &latex[arg3.len()..];
        refined.push_str(r"\left(\frac{\partial ");
        refined.push_str(&physics_macros(arg1));
        refined.push_str(r"}{\partial ");
        refined.push_str(&physics_macros(arg2));
        refined.push_str(r"}\right)_");
        refined.push_str(&physics_macros(arg3));
    }
    refined.push_str(latex);

    refined
}

/// Pull (sub)sections out of guide/solution/handout
pub fn pull_sections_out(latex: &str) -> String {
    let latex = pull_sections_out_of_environ(latex, "handout");
    let latex = pull_sections_out_of_environ(&latex, "guide");
    let latex = pull_sections_out_of_environ(&latex, "solution");
    latex
}

/// Pull (sub)sections out of guide/solution/handout
fn pull_sections_out_of_environ(latex: &str, environ: &str) -> String {
    let mut latex = latex; // this makes the lifetime local to the function
    let mut refined = String::with_capacity(latex.len());
    let begin = format!(r"\begin{{{}}}", environ);
    let end = format!(r"\end{{{}}}", environ);
    let section = regex::Regex::new(r"\\[sub]*section\{[^\}]+\}").unwrap();
    while let Some(i) = latex.find(&begin) {
        refined.push_str(&latex[..i + begin.len()]);
        latex = &latex[i + begin.len()..];
        if let Some(mut i) = latex.find(&end) {
            while let Some(next_section) = section.find(latex) {
                if next_section.end() >= i {
                    break;
                }
                refined.push_str(&latex[..next_section.start()]);
                refined.push_str(&end);
                refined.push_str(&latex[next_section.start()..next_section.end()]);
                refined.push_str(&begin);
                latex = &latex[next_section.end()..];
                i -= next_section.end();
            }
        }
    }
    refined.push_str(latex);
    refined
}

/// Check latex against supported macros
#[wasm_bindgen]
pub fn check_latex(latex: &str) -> String {
    let mut refined = String::with_capacity(latex.len());
    let environments = regex::Regex::new(r"\\begin\{([^\}]+)\}").unwrap();
    let mut environments: std::collections::HashSet<String> = environments
        .captures_iter(latex)
        .map(|m| m[1].to_string())
        .collect();
    // The following are known good environments
    let good_environments: &[&'static str] = &[
        "solution",
        "enumerate",
        "description",
        "equation",
        "equation*",
        "align",
        "align*",
        "figure",
        "eqnarray",
        "eqnarray*",
        "multline",
        "multline*",
        "quote",
        "quotation",
        "center",
    ];
    for &e in good_environments.iter() {
        environments.remove(e);
    }
    // Unsupported environments.  I'm not actually aware of anything
    // that we cannot handle or that we will not want to permit.
    let bad_environments: &[&'static str] = &["buggy"];
    for &e in bad_environments.iter() {
        if environments.contains(e) {
            refined.push_str(&format!(
                r#"\error{{bad environment: {}}}\\
"#,
                e
            ));
            environments.remove(e);
        }
    }

    let macros = regex::Regex::new(r"\\([^0-9_/|><\\$\-+\s\(\)\[\]{}]+)").unwrap();
    let mut macros: std::collections::HashSet<String> = macros
        .captures_iter(latex)
        .map(|m| m[1].to_string())
        .collect();
    // The following is a whitelist of definitely non-problematic
    // macros.  I'm not sure when if ever we want to enforce only
    // macros on this whitelist.  For now I'm figuring to warn on
    // anything outside the list.  Ideally we'd have a list of macros
    // that pandoc understands and use that, but we also would need a
    // list of things MathJax understands, since pandoc can effectively
    // pass along any math symbols without understanding them, so long
    // as MathJax *does* understand them.
    let good_macros = &[
        "section",
        "eqref",
        "ref",
        "label",
        "centering",
        "subsection",
        "subsubsection",
        "footnote",
        "section*",
        "subsection*",
        "subsubsection*",
        "begin",
        "end",
        "includegraphics",
        "columnwidth",
        "emph",
        "paragraph",
        "noindent",
        "textwidth",
        "item",
        "textbf",
        "psi",
        "Psi",
        "phi",
        "Phi",
        "delta",
        "Delta",
        r#""o"#,
        r#""u"#,
        "&",
        "%",
        "left",
        "right",
        "frac",
        "pm",
        ";",
        ",",
        "text",
        "textit",
        "textrm",
        "it",
        "em",
        "textbackslash",
        "langle",
        "rangle",
    ];
    for &m in good_macros.iter() {
        macros.remove(m);
    }
    // Unsupported macros.
    let bad_macros = &[
        "newcommand",
        "mathchar", // unsupported by mathjax
        "renewcommand",
        "newenvironment", // have namespacing issues
        "usepackage",     // big can of worms
        "def",            // namespacing issues?
        "cases",          // old cases that doesn't work with amsmath
    ];
    for &m in bad_macros.iter() {
        if macros.contains(m) {
            refined.push_str(&format!(
                r#"\error{{bad macro: \textbackslash{{}}{}}}\\
"#,
                m
            ));
            macros.remove(m);
        }
    }
    for e in environments {
        refined.push_str(&format!(
            r#"\warning{{possibly bad environment: {}}}\\
"#,
            e
        ));
    }
    for m in macros {
        refined.push_str(&format!(
            r#"\warning{{possibly bad macro: \textbackslash{{}}{}}}\\
"#,
            m
        ));
    }
    refined.push_str(&latex);
    refined
}

/// Include solutions via \begin{solution}
pub fn include_solutions(mut latex: &str) -> String {
    let mut refined = String::with_capacity(latex.len());
    loop {
        if let Some(i) = latex.find(r"\begin{solution}") {
            refined.push_str(&latex[..i]);
            latex = &latex[i + r"\begin{solution}".len()..];

            refined.push_str(r"\begin{quotation}\paragraph*{Solution}");
            if let Some(i) = latex.find(r"\end{solution}") {
                refined.push_str(&latex[..i]);
                refined.push_str(r"\end{quotation}");
                latex = &latex[i + r"\end{solution}".len()..];
            } else {
                refined.push_str(latex);
                refined.push_str(r"\end{quotation}");
                break;
            }
        } else {
            refined.push_str(latex);
            break;
        }
    }
    refined
}

/// Strip out solutions
pub fn omit_solutions(mut latex: &str) -> String {
    let mut refined = String::with_capacity(latex.len());
    // need to strip out solutions...
    loop {
        if let Some(i) = latex.find(r"\begin{solution}") {
            refined.push_str(&latex[..i]);
            latex = &latex[i + r"\begin{solution}".len()..];
            if let Some(i) = latex.find(r"\end{solution}") {
                latex = &latex[i + r"\end{solution}".len()..];
            } else {
                break;
            }
        } else {
            refined.push_str(latex);
            break;
        }
    }
    refined
}

/// Strip out guides
pub fn omit_guide(mut latex: &str) -> String {
    let mut refined = String::with_capacity(latex.len());
    // need to strip out guides...
    loop {
        if let Some(i) = latex.find(r"\begin{guide}") {
            refined.push_str(&latex[..i]);
            latex = &latex[i + r"\begin{guide}".len()..];
            if let Some(i) = latex.find(r"\end{guide}") {
                latex = &latex[i + r"\end{guide}".len()..];
            } else {
                break;
            }
        } else {
            refined.push_str(latex);
            break;
        }
    }
    refined
}

/// Strip out handouts
pub fn omit_handout(mut latex: &str) -> String {
    let mut refined = String::with_capacity(latex.len());
    // need to strip out handouts...
    loop {
        if let Some(i) = latex.find(r"\begin{handout}") {
            refined.push_str(&latex[..i]);
            latex = &latex[i + r"\begin{handout}".len()..];
            if let Some(i) = latex.find(r"\end{handout}") {
                latex = &latex[i + r"\end{handout}".len()..];
            } else {
                break;
            }
        } else {
            refined.push_str(latex);
            break;
        }
    }
    refined
}

/// Keep just the handouts
pub fn only_handout(mut latex: &str) -> String {
    let mut refined = String::with_capacity(latex.len());
    // need to strip out handouts...
    loop {
        if let Some(i) = latex.find(r"\begin{handout}") {
            latex = &latex[i + r"\begin{handout}".len()..];
            if let Some(i) = latex.find(r"\end{handout}") {
                refined.push_str(&latex[..i]);
                latex = &latex[i + r"\end{handout}".len()..];
            } else {
                break;
            }
        } else {
            break;
        }
    }
    refined
}

/// Process `\includegraphics` with the specified image directory.
pub fn with_image_directory(mut latex: &str, img_dir: &str) -> String {
    let mut refined = String::with_capacity(latex.len());
    loop {
        if let Some(i) = latex.find(r"\includegraphics[") {
            refined.push_str(&latex[..i]);
            latex = &latex[i + r"\includegraphics[".len()..];
            if let Some(nxt) = latex.find("]{") {
                latex = &latex[nxt + r"]{".len()..];
                if let Some(endfn) = latex.find("}") {
                    refined.push_str(r#"<img src=""#);
                    refined.push_str(img_dir);
                    refined.push_str(&latex[..endfn]);
                    refined.push_str(r#""/>"#);
                    latex = &latex[endfn + r"}".len()..];
                } else {
                    refined.push_str(r#"<span class="error">\includegraphics[..]{</span>"#);
                }
            } else {
                refined.push_str(r#"<span class="error">\includegraphics[</span>"#);
            }
        } else if let Some(i) = latex.find(r"\includegraphics{") {
            refined.push_str(&latex[..i]);
            latex = &latex[i + r"\includegraphics{".len()..];
            if let Some(endfn) = latex.find("}") {
                refined.push_str(r#"<img src=""#);
                refined.push_str(img_dir);
                refined.push_str(&latex[..endfn]);
                refined.push_str(r#""/>"#);
                latex = &latex[endfn + r"}".len()..];
            } else {
                refined.push_str(r#"<span class="error">\includegraphics{</span>"#);
            }
        } else {
            refined.push_str(latex);
            break;
        }
    }
    refined
}

// The following are wasm-specific

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub fn set_panic_hook() {
    //! When the `console_error_panic_hook` feature is enabled, we can call the
    //! `set_panic_hook` function at least once during initialization, and then
    //! we will get better error messages if our code ever panics.
    //!
    //! For more details see
    //! https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
