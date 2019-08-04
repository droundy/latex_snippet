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
    let mut s = String::with_capacity(latex.len());
    html(&mut s, latex).unwrap();
    s
}

/// Convert some LaTeX into HTML, and send the results to a `std::fmt::Write`.
pub fn html(fmt: &mut impl std::fmt::Write, mut latex: &str) -> Result<(),std::fmt::Error> {
    let math_environs = &["{equation}", "{align}"];
    loop {
        if latex.len() == 0 {
            return Ok(());
        }
        if let Some(i) = latex.find(|c| c == '\\' || c == '{' || c == '$') {
            fmt.write_str(&latex[..i])?;
            latex = &latex[i..];
            let c = latex.chars().next().unwrap();
            if c == '\\' {
                let name = macro_name(latex);
                latex = &latex[name.len()..];
                match name {
                    r"\emph" => {
                        let arg = argument(latex);
                        latex = &latex[arg.len()..];
                        if arg == "{" {
                            fmt.write_str(r#"<span class="error">\emph{</span>"#)?;
                        } else {
                            fmt.write_str("<em>")?;
                            html(fmt, arg)?;
                            fmt.write_str("</em>")?;
                        }
                    }
                    r"\it" => {
                        latex = finish_standalone_macro(latex);
                        fmt.write_str("<i>")?;
                        html(fmt, latex)?;
                        return fmt.write_str("</i>");
                    }
                    r"\ " => {
                        fmt.write_str(" ")?;
                    }
                    r"\%" => {
                        fmt.write_str("%")?;
                    }
                    r"\bf" => {
                        latex = finish_standalone_macro(latex);
                        fmt.write_str("<b>")?;
                        html(fmt, latex)?;
                        return fmt.write_str("</b>");
                    }
                    r"\sc" => {
                        latex = finish_standalone_macro(latex);
                        fmt.write_str(r#"<font style="font-variant: small-caps">"#)?;
                        html(fmt, latex)?;
                        return fmt.write_str("</font>");
                    }
                    r"\begin" => {
                        // We are looking at an environment...
                        let name = env_name(latex);
                        latex = &latex[name.len()..];
                        if name.chars().last() != Some('}') {
                            write!(fmt,r#"<span class="error">\begin{}</span>"#, name)?;
                        } else if math_environs.contains(&name) {
                            let env = end_env(name, latex);
                            latex = &latex[env.len()..];
                            if env == "" {
                                write!(fmt,r#"<span class="error">\begin{}</span>"#, name)?;
                            } else {
                                write!(fmt,r#"\begin{}{}"#, name, env)?;
                            }
                        } else if name == "{itemize}" {
                            fmt.write_str("<ul>")?;
                            let li = finish_item(latex);
                            latex = &latex[li.len()..];
                            if li.trim().len() > 0 {
                                // Nothing should precede the first
                                // \item except whitespace.
                                write!(fmt,r#"<span class="error">{}</span>"#, li)?;
                            }
                            loop {
                                let li = finish_item(latex);
                                latex = &latex[li.len()..];
                                if li.len() == 0 {
                                    if latex.starts_with(r"\end{itemize}") {
                                        latex = &latex[r"\end{itemize}".len()..];
                                        fmt.write_str("</ul>")?;
                                        break;
                                    } else if latex.starts_with(r"\end{enumerate}") {
                                        latex = &latex[r"\end{enumerate}".len()..];
                                        fmt.write_str(r#"</ul><span class="error">\end{enumerate}</span>"#)?;
                                        break;
                                    } else if latex.starts_with(r"\item") {
                                        // It must start with \item
                                        latex = &latex[r"\item".len()..];
                                        latex = finish_standalone_macro(latex);
                                    } else {
                                        fmt.write_str(r#"</ul><span class="error">MISSING END</span>"#)?;
                                        break;
                                    }
                                } else {
                                    fmt.write_str("<li>")?;
                                    html(fmt, li)?;
                                    fmt.write_str("</li>")?;
                                }
                            }
                        } else if name == "{enumerate}" {
                            fmt.write_str("<ol>")?;
                            let li = finish_item(latex);
                            latex = &latex[li.len()..];
                            if li.trim().len() > 0 {
                                // Nothing should precede the first
                                // \item except whitespace.
                                write!(fmt,r#"<span class="error">{}</span>"#, li)?;
                            }
                            loop {
                                let li = finish_item(latex);
                                latex = &latex[li.len()..];
                                if li.len() == 0 {
                                    if latex.starts_with(r"\end{itemize}") {
                                        latex = &latex[r"\end{itemize}".len()..];
                                        fmt.write_str(r#"</ol><span class="error">\end{enumerate}</span>"#)?;
                                        break;
                                    } else if latex.starts_with(r"\end{enumerate}") {
                                        latex = &latex[r"\end{enumerate}".len()..];
                                        fmt.write_str("</ol>")?;
                                        break;
                                    } else if latex.starts_with(r"\item") {
                                        // It must start with \item
                                        latex = &latex[r"\item".len()..];
                                        latex = finish_standalone_macro(latex);
                                    } else {
                                        fmt.write_str(r#"</ol><span class="error">MISSING END</span>"#)?;
                                        break;
                                    }
                                } else {
                                    fmt.write_str("<li>")?;
                                    html(fmt, li)?;
                                    fmt.write_str("</li>")?;
                                }
                            }
                        } else {
                            let env = end_env(name, latex);
                            latex = &latex[env.len()..];
                            write!(fmt, r#"<span class="error">\begin{}{}</span>"#,
                                   name, env)?;
                        }
                    }
                    r"\end" => {
                        let name = env_name(latex);
                        latex = &latex[name.len()..];
                        write!(fmt,r#"<span class="error">\end{}</span>"#, name)?;
                    }
                    _ => {
                        write!(fmt, r#"<span class="error">{}</span>"#, name)?;
                    }
                }
            } else if c == '$' {
                if let Some(i) = latex[1..].find('$') {
                    fmt.write_str(&latex[..i+2])?;
                    latex = &latex[i+2..];
                } else {
                    fmt.write_str(r#"<span class="error">$</span>"#)?;
                    latex = &latex[1..];
                }
            } else {
                let arg = argument(latex);
                latex = &latex[arg.len()..];
                if arg == "{" {
                    fmt.write_str(r#"<span class="error">{</span>"#)?;
                } else {
                    html(fmt, &arg[1..arg.len()-1])?;
                }
            }
        } else {
            return fmt.write_str(latex);
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
            &latex[..i+1]
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
        &latex[..i+1]
    } else if let Some(i) = latex.find(|c: char| c != '{' && !c.is_alphabetic()) {
        &latex[..i+1]
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

fn finish_item<'a>(latex: &'a str) -> &'a str {
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
                return &latex[..so_far + i]
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
                    return &latex[..arg.len()]
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
fn hello_world() {
    assert_eq!("hello world", &html_string("hello world"));
}
#[test]
fn emph_hello() {
    assert_eq!("<em>hello</em>", &html_string(r"\emph{hello}"));
}
#[test]
fn hello_it() {
    assert_eq!("hello good <i>world</i>", &html_string(r"hello {good \it world}"));
}
#[test]
fn inline_math() {
    assert_eq!(r"hello good $\cos^2x$ math", &html_string(r"hello good $\cos^2x$ math"));
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
fn unrecognized_env() {
    assert_eq!(r#"hello <span class="error">\begin{broken}
stuff
\end{broken}</span>"#, &html_string(r"hello \begin{broken}
stuff
\end{broken}"));
}
#[test]
fn unrecognized_unbalanced_env() {
    assert_eq!(r#"hello <span class="error">\begin{broken}</span>
stuff
<span class="error">\end{broke}</span>"#, &html_string(r"hello \begin{broken}
stuff
\end{broke}"));
}
#[test]
fn equation() {
    assert_eq!(r"
\begin{equation}
 y = x^2
\end{equation}
some more math
",
               &html_string(r"
\begin{equation}
 y = x^2
\end{equation}
some more math
"));
}

#[test]
fn itemize() {
    assert_eq!(r"
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
               &html_string(r"
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
"));
}

#[test]
fn enumerate() {
    assert_eq!(r#"
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
               &html_string(r"
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
"));
}

#[test]
fn incomplet_begin() {
    assert_eq!(r#"
<span class="error">\begin{enum</span>"#,
               &html_string(r"
\begin{enum"));
}

#[test]
fn incomplet_end() {
    assert_eq!(r#"
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
               &html_string(r"
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
"));
}

#[test]
fn incomplet_end_itemize() {
    assert_eq!(r#"
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
               &html_string(r"
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
"));
}
