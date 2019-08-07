#![deny(missing_docs)]

//! This crate turns (a subset of) latex into html, with syntax errors
//! reported using span elements.

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

/// Convert some LaTeX into an HTML `String`.
pub fn html_string(latex: &str) -> String {
    let mut s: Vec<u8> = Vec::with_capacity(latex.len());
    html(&mut s, latex).unwrap();
    String::from_utf8(s).expect("should be no problem with utf8 conversion")
}

/// Convert some LaTeX into HTML, and send the results to a `std::io::Write`.
pub fn html(fmt: &mut impl std::io::Write, mut latex: &str) -> Result<(), std::io::Error> {
    if let Some(i) = latex.find(r"\section") {
        html_section(fmt, &latex[..i])?;
        latex = &latex[i + r"\section".len()..];
        let title = parse_title(latex);
        latex = &latex[title.len()..];
        if title == "{" {
            fmt.write_all(br#"<span class="error">\emph{</span>"#)?;
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
            let title = parse_title(latex);
            latex = &latex[title.len()..];
            if title == "{" {
                fmt.write_all(br#"<span class="error">\emph{</span>"#)?;
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
        let title = parse_title(latex);
        latex = &latex[title.len()..];
        if title == "{" {
            fmt.write_all(br#"<span class="error">\emph{</span>"#)?;
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
            let title = parse_title(latex);
            latex = &latex[title.len()..];
            if title == "{" {
                fmt.write_all(br#"<span class="error">\emph{</span>"#)?;
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
pub fn html_subsection(fmt: &mut impl std::io::Write, mut latex: &str) -> Result<(), std::io::Error> {
    if let Some(i) = latex.find(r"\subsubsection") {
        html_subsubsection(fmt, &latex[..i])?;
        latex = &latex[i + r"\subsubsection".len()..];
        let title = parse_title(latex);
        latex = &latex[title.len()..];
        if title == "{" {
            fmt.write_all(br#"<span class="error">\emph{</span>"#)?;
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
            let title = parse_title(latex);
            latex = &latex[title.len()..];
            if title == "{" {
                fmt.write_all(br#"<span class="error">\emph{</span>"#)?;
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
pub fn html_subsubsection(fmt: &mut impl std::io::Write, mut latex: &str) -> Result<(), std::io::Error> {
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
pub fn html_paragraph(
    fmt: &mut impl std::io::Write,
    mut latex: &str,
) -> Result<(), std::io::Error> {
    let math_environs = &["{equation}", "{align}"];
    loop {
        if latex.len() == 0 {
            return Ok(());
        }
        if let Some(i) = latex.find(|c| c == '\\' || c == '{' || c == '$') {
            fmt.write_all(latex[..i].as_bytes())?;
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
                    r"\emph" => {
                        let arg = argument(latex);
                        latex = &latex[arg.len()..];
                        if arg == "{" {
                            fmt.write_all(br#"<span class="error">\emph{</span>"#)?;
                        } else {
                            fmt.write_all(b"<em>")?;
                            html(fmt, arg)?;
                            fmt.write_all(b"</em>")?;
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
                            fmt.write_all(br#"<span class="error">\emph{</span>"#)?;
                        } else {
                            fmt.write_all(br#"<span> class="error">"#)?;
                            html(fmt, arg)?;
                            fmt.write_all(b"</span>")?;
                        }
                    }
                    r"\paragraph" => {
                        let arg = parse_title(latex);
                        latex = latex[arg.len()..].trim_start();
                        if arg == "{" {
                            fmt.write_all(br#"<span class="error">\emph{</span>"#)?;
                        } else {
                            fmt.write_all(b"<h5>")?;
                            html(fmt, arg)?;
                            fmt.write_all(b"</h5>")?;
                        }
                    }
                    r"\it" => {
                        latex = finish_standalone_macro(latex);
                        fmt.write_all(b"<i>")?;
                        html(fmt, latex)?;
                        return fmt.write_all(b"</i>");
                    }
                    r"\ " => {
                        fmt.write_all(b" ")?;
                    }
                    r"\%" => {
                        fmt.write_all(b"%")?;
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
                            write!(fmt, r#"<span class="error">\begin{}</span>"#, name)?;
                        } else if math_environs.contains(&name) {
                            let env = end_env(name, latex);
                            latex = &latex[env.len()..];
                            if env == "" {
                                write!(fmt, r#"<span class="error">\begin{}</span>"#, name)?;
                            } else {
                                write!(fmt, r#"\begin{}{}"#, name, env)?;
                            }
                        } else if name == "{itemize}" {
                            fmt.write_all(b"<ul>")?;
                            let li = finish_item(latex);
                            latex = &latex[li.len()..];
                            if li.trim().len() > 0 {
                                // Nothing should precede the first
                                // \item except whitespace.
                                write!(fmt, r#"<span class="error">{}</span>"#, li)?;
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
                                write!(fmt, r#"<span class="error">{}</span>"#, li)?;
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
                            write!(fmt, r#"<span class="error">\begin{}{}</span>"#, name, env)?;
                        }
                    }
                    r"\end" => {
                        let name = env_name(latex);
                        latex = &latex[name.len()..];
                        write!(fmt, r#"<span class="error">\end{}</span>"#, name)?;
                    }
                    _ => {
                        write!(fmt, r#"<span class="error">{}</span>"#, name)?;
                    }
                }
            } else if c == '$' {
                if let Some(i) = latex[1..].find('$') {
                    fmt.write_all(latex[..i + 2].as_bytes())?;
                    latex = &latex[i + 2..];
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
            return fmt.write_all(latex.as_bytes());
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
    if let Some(i) = latex[1..].find(|c: char| !c.is_alphabetic()) {
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

fn parse_title(mut latex: &str) -> &str {
    if latex.len() == 0 {
        return ""
    }
    if latex.chars().next().unwrap() == '*' {
        latex = &latex[1..]; // just ignore a *
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

#[test]
fn test_section() {
    assert_eq!("xx
<section><h2>foo</h2>
bar
</section><section><h2>baz</h2>
baz
</section>",
               &html_string(r"xx
\section{foo}
bar
\section{baz}
baz
"));
}

#[test]
fn test_subsection() {
    assert_eq!("xx
<section><h3>foo</h3>
bar
</section><section><h3>baz</h3>
baz
</section>",
               &html_string(r"xx
\subsection{foo}
bar
\subsection{baz}
baz
"));

    assert_eq!("xx
<section><h3>foo</h3>
bar
<section><h4>baz</h4>
baz
</section></section>",
               &html_string(r"xx
\subsection{foo}
bar
\subsubsection{baz}
baz
"));

    assert_eq!("xx
<section><h2>foo</h2>
bar
<section><h4>baz</h4>
baz
</section></section>",
               &html_string(r"xx
\section{foo}
bar
\subsubsection{baz}
baz
"));
}

#[test]
fn test_subsubsection() {
    assert_eq!("xx
<section><h4>foo</h4>
bar
</section><section><h4>baz</h4>
baz
</section>",
               &html_string(r"xx
\subsubsection{foo}
bar
\subsubsection{baz}
baz
"));
    assert_eq!("xx
<section><h4>foo</h4>
bar
</section><section><h3>baz</h3>
baz
</section>",
               &html_string(r"xx
\subsubsection{foo}
bar
\subsection{baz}
baz
"));
}

#[test]
fn hello_world() {
    assert_eq!("hello world", &html_string("hello world"));
}
#[test]
fn emph_hello() {
    assert_eq!("<em>hello</em>", &html_string(r"\emph{hello}"));
}
#[test]
fn paragraph_test() {
    assert_eq!(
        "<p><h5>hello</h5>This is good
</p>",
        &html_string(
            r"

\paragraph{hello}
This is good
"
        )
    );
}
#[test]
fn hello_it() {
    assert_eq!(
        "hello good <i>world</i>",
        &html_string(r"hello {good \it world}")
    );
}
#[test]
fn inline_math() {
    assert_eq!(
        r"hello good $\cos^2x$ math",
        &html_string(r"hello good $\cos^2x$ math")
    );
}
#[test]
fn escape_space() {
    assert_eq!(r"hello<i> world</i>", &html_string(r"hello\it\ world"));
}
#[test]
fn escape_percent() {
    assert_eq!(r"50% full", &html_string(r"50\% full"));
}

#[test]
fn line_break() {
    assert_eq!(
        r"Hello world<br/>this is a new line",
        &html_string(r"Hello world\\this is a new line")
    );
}

#[test]
fn paragraphs() {
    assert_eq!(
        r"<p>The first paragraph

</p><p>The second paragraph</p>",
        &html_string(
            r"The first paragraph

The second paragraph"
        )
    );
}

#[test]
fn unrecognized_env() {
    assert_eq!(
        r#"hello <span class="error">\begin{broken}
stuff
\end{broken}</span>"#,
        &html_string(
            r"hello \begin{broken}
stuff
\end{broken}"
        )
    );
}
#[test]
fn unrecognized_unbalanced_env() {
    assert_eq!(
        r#"hello <span class="error">\begin{broken}</span>
stuff
<span class="error">\end{broke}</span>"#,
        &html_string(
            r"hello \begin{broken}
stuff
\end{broke}"
        )
    );
}
#[test]
fn equation() {
    assert_eq!(
        r"
\begin{equation}
 y = x^2
\end{equation}
some more math
",
        &html_string(
            r"
\begin{equation}
 y = x^2
\end{equation}
some more math
"
        )
    );
}

#[test]
fn itemize() {
    assert_eq!(
        r"
<ul><li>Apples
</li><li>Oranges
</li><li>Vegetables
<ul><li>Carrots
</li><li>Potatotes
</li></ul>
</li><li>Pears
</li></ul>
some more stuff
",
        &html_string(
            r"
\begin{itemize}
\item Apples
\item Oranges
\item Vegetables
\begin{itemize}
\item Carrots
\item Potatotes
\end{itemize}
\item Pears
\end{itemize}
some more stuff
"
        )
    );
}

#[test]
fn enumerate() {
    assert_eq!(
        r#"
<ol><span class="error">
buggy
</span><li>Apples
</li><li>Oranges
</li><li>Vegetables
<ul><li>Carrots
</li><li>Potatotes
</li></ul>
</li><li>Pears
</li></ol>
some more stuff
"#,
        &html_string(
            r"
\begin{enumerate}
buggy
\item Apples
\item Oranges
\item Vegetables
\begin{itemize}
\item Carrots
\item Potatotes
\end{itemize}
\item Pears
\end{enumerate}
some more stuff
"
        )
    );
}

#[test]
fn incomplet_begin() {
    assert_eq!(
        r#"
<span class="error">\begin{enum</span>"#,
        &html_string(
            r"
\begin{enum"
        )
    );
}

#[test]
fn incomplet_end() {
    assert_eq!(
        r#"
<ol><span class="error">
buggy
</span><li>Apples
</li><li>Oranges
</li><li>Vegetables
<ul><li>Carrots
</li><li>Potatotes
</li></ul>
</li></ol><span class="error">MISSING END</span>Pears
<span class="error">\end{enumerate
</span>some more stuff
"#,
        &html_string(
            r"
\begin{enumerate}
buggy
\item Apples
\item Oranges
\item Vegetables
\begin{itemize}
\item Carrots
\item Potatotes
\end{itemize}
\item Pears
\end{enumerate
some more stuff
"
        )
    );
}

#[test]
fn incomplet_end_itemize() {
    assert_eq!(
        r#"
<ul><span class="error">
buggy
</span><li>Apples
</li><li>Oranges
</li><li>Vegetables
<ul><li>Carrots
</li><li>Potatotes
</li></ul>
</li></ul><span class="error">MISSING END</span>Pears
<span class="error">\end{itemize
</span>some more stuff
"#,
        &html_string(
            r"
\begin{itemize}
buggy
\item Apples
\item Oranges
\item Vegetables
\begin{itemize}
\item Carrots
\item Potatotes
\end{itemize}
\item Pears
\end{itemize
some more stuff
"
        )
    );
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
        "section", "subsection", "subsubsection",
        "section*", "subsection*", "subsubsection*",
        r"begin", r"end", r"includegraphics", r"columnwidth",
        "emph", "paragraph", r"noindent", "textwidth", r"item",
        r"psi", r"Psi",
        r#""o"#, r#""u"#, r"&", r"%",
        r"left", r"right", r"frac",
        r"pm", r";", r",",
        r"text", "textit", "textrm", r"it", r"em",
        r"textbackslash",
    ];
    for &m in good_macros.iter() {
        macros.remove(m);
    }
    // Unsupported macros.
    let bad_macros = &[
        "newcommand",
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
"#, m));
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
"#, m
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
                latex = &latex[i + r"\end{solution}".len()..];
            } else {
                refined.push_str(latex);
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
