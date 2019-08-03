pub fn html(fmt: &mut impl std::fmt::Write, mut latex: &str) -> Result<(),std::fmt::Error> {
    let math_environs = &["{equation}", "{align}"];
    loop {
        if latex.len() == 0 {
            return Ok(());
        }
        if let Some(i) = latex.find(|c| c == '\\' || c == '{' || c == '$') {
            fmt.write_str(&latex[..i])?;
            println!("found i {}", i);
            latex = &latex[i..];
            let c = latex.chars().next().unwrap();
            println!("latex is now {:?}", latex);
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
                        fmt.write_str("<em>")?;
                        html(fmt, latex)?;
                        return fmt.write_str("</em>");
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
                        }
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
            println!("could not find anything in {:?}", latex);
            return fmt.write_str(latex);
        }
    }
}

fn finish_standalone_macro(latex: &str) -> &str {
    if latex.len() == 0 {
        ""
    } else if latex.chars().next().unwrap() == ' ' {
        println!("foo bar {:?}", latex);
        &latex[1..]
    } else {
        println!("it was {:?}", latex);
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

pub fn html_string(latex: &str) -> String {
    let mut s = String::with_capacity(latex.len());
    html(&mut s, latex).unwrap();
    s
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
    assert_eq!("hello good <em>world</em>", &html_string(r"hello {good \it world}"));
}
#[test]
fn inline_math() {
    assert_eq!(r"hello good $\cos^2x$ math", &html_string(r"hello good $\cos^2x$ math"));
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
