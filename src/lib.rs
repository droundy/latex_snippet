#![deny(missing_docs)]

//! This crate turns (a subset of) latex into html, with syntax errors
//! reported using span elements.

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
                "<" => "&lt;",
                ">" => "&gt;",
                "&" => "&amp;",
                "\"" => "&quot;",
                "'" => "&#x27;",
                "/" => "&#x2f;",
                "``" => "“",
                "''" => "”",
                _ => unreachable!(),
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
    let latex = strip_comments(latex);
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
        if let Some(i) = latex.find(|c| c == '\\' || c == '{' || c == '$') {
            fmt_as_html(fmt, &latex[..i])?;
            latex = &latex[i..];
            let c = latex.chars().next().unwrap();
            if c == '\\' {
                let name = macro_name(latex);
                latex = &latex[name.len()..];
                match name {
                    r"\\" => {
                        fmt.write_all(b"<br/>")?;
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
                            fmt.write_all(br#"<span class="error">\label{</span>"#)?;
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
                    r"\includegraphics" => {
                        let opt = optional_argument(latex);
                        latex = &latex[opt.len()..];
                        let arg = argument(latex);
                        latex = &latex[arg.len()..];
                        if arg == "{" {
                            fmt.write_all(br#"<span class="error">\includegraphics{</span>"#)?;
                        } else {
                            fmt.write_all(br#"<img src=""#)?;
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
                        fmt.write_all(b"$")?;
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
                            fmt_as_html(fmt, &latex[..i + 2])?;
                            latex = &latex[i + 2..];
                        } else {
                            fmt_error(fmt, r"\(")?;
                        }
                    }
                    r"\[" => {
                        if let Some(i) = latex.find(r"\]") {
                            fmt.write_all(br"\[")?;
                            fmt_as_html(fmt, &latex[..i + 2])?;
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
                                fmt_as_html(fmt, env)?;
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
                                            br#"</ul><span class="error">MISSING END</span>"#,
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
                                            br#"</ol><span class="error">MISSING END</span>"#,
                                        )?;
                                        break;
                                    }
                                } else {
                                    fmt.write_all(b"<li>")?;
                                    html(fmt, li)?;
                                    fmt.write_all(b"</li>")?;
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
                            fmt_as_html(fmt, &latex[2..i + 2])?;
                            fmt.write_all(br"\]")?;
                            latex = &latex[i + 4..];
                        } else {
                            fmt.write_all(br#"<span class="error">$$</span>"#)?;
                            latex = &latex[2..];
                        }
                    } else {
                        fmt.write_all(br"\(")?;
                        fmt_as_html(fmt, &latex[1..i + 1])?;
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

fn finish_paragraph(latex: &str) -> &str {
    if latex.len() == 0 {
        return "";
    }
    let mut so_far = 0;
    let mut nestedness = 0;
    loop {
        let next_paragraph = latex[so_far..].find("\n\n");
        let next_end = latex[so_far..].find(r"\end{");
        let next_begin = latex[so_far..].find(r"\begin{");
        if earlier(next_paragraph, next_begin) && earlier(next_paragraph, next_end) {
            if nestedness == 0 {
                if let Some(i) = next_paragraph {
                    so_far += i;
                    while latex.len() > so_far + 1
                        && latex[so_far + 1..].chars().next() == Some('\n')
                    {
                        so_far += 1;
                    }
                    return &latex[..so_far + 1];
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
    let end_list = regex::Regex::new(r"\\end\{(itemize|enumerate)\}").unwrap();
    let begin_list = regex::Regex::new(r"\\begin\{(itemize|enumerate)\}").unwrap();
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
        &latex[..1]
    }
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
        refined.push_str("|");
        refined.push_str(&physics_macros(arg));
        refined.push_str(r"\rangle ");
    }
    refined.push_str(latex);
    latex = &refined;

    let mut refined = String::with_capacity(latex.len());
    while let Some(i) = latex.find(r"\bra{") {
        refined.push_str(&latex[..i]);
        latex = &latex[i + r"\bra".len()..];
        let arg = argument(latex);
        latex = &latex[arg.len()..];
        refined.push_str(r"\langle ");
        refined.push_str(&physics_macros(arg));
        refined.push_str(r"|");
    }
    refined.push_str(latex);
    latex = &refined;

    let mut refined = String::with_capacity(latex.len());
    while let Some(i) = latex.find(r"\dbar ") {
        refined.push_str(&latex[..i]);
        latex = &latex[i + r"\dbar".len()..];
        refined.push_str(r"{d\hspace{-0.28em}\bar{}}");
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
        refined.push_str(
            r"\left( % \myderiv
\frac",
        );
        refined.push_str(&physics_macros(arg1));
        refined.push_str(&physics_macros(arg2));
        refined.push_str(r"\right)_");
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
        refined.push_str(r"\left(\frac");
        refined.push_str(&physics_macros(arg1));
        refined.push_str(&physics_macros(arg2));
        refined.push_str(r"\right)_");
        refined.push_str(&physics_macros(arg3));
    }
    refined.push_str(latex);

    refined
}

/// Check latex against supported macros
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

            refined.push_str(r"\paragraph*{Solution}{\it ");
            if let Some(i) = latex.find(r"\end{solution}") {
                refined.push_str(&latex[..i]);
                refined.push_str("}\n\n");
                latex = &latex[i + r"\end{solution}".len()..];
            } else {
                refined.push_str(latex);
                refined.push_str("}");
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
