use auto_args::AutoArgs;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, AutoArgs)]
enum Format {
    /// HTML format
    HTML,
    /// PDF format
    PDF,
    /// latex source
    Latex,
}

#[derive(Debug, AutoArgs)]
struct Args {
    /// choose format (html, tex, or pdf)
    _format: Format,
    /// show solutions
    solution: bool,
    /// check for unsupported macros
    check: bool,
    /// primary key for database
    pk: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::from_args();

    let mut latex = String::new();
    if let Some(pk) = args.pk {
        let output = std::process::Command::new("mysql")
            .args(&["-u", "osubash", "-ss", "-N", "-H", "-e",
                    &format!("select problem_latex from osu_production.admin_app_problem where id = {}",
                    pk)])
            .output()
            .expect("failed to execute process");
        latex = String::from_utf8_lossy(&output.stdout[28..output.stdout.len()-18])
            .to_string();
    } else {
        use std::io::Read;
        std::io::stdin().read_to_string(&mut latex)?;
    }
    let mut latex: &str = &latex;
    let mut refined = String::with_capacity(latex.len());
    if args.check {
        let environments = regex::Regex::new(r"\\begin\{([^\}]+)\}").unwrap();
        let mut environments: std::collections::HashSet<String> =
            environments.find_iter(latex)
            .map(|m| m.as_str().to_string())
            .collect();
        // The following are known good environments
        let good_environments: &[&'static str] =
            &["solution", "enumerate", "equation", "equation*", "align", "align*"];
        for &e in good_environments {
            environments.remove(e);
        }
        // Unsupported environments.  I'm not actually aware of anything
        // that we cannot handle or that we will not want to permit.
        let bad_environments: &[&'static str] = &["buggy"];
        for &e in bad_environments {
            if environments.contains(e) {
                refined.push_str(&format!(r#"\error{{bad environment: {}}}\\
"#, e));
                environments.remove(e);
            }
        }

        let macros = regex::Regex::new(r"\\([^0-9_/|><\\$\-+\s\(\)\[\]{}]+)").unwrap();
        let mut macros: std::collections::HashSet<String> =
            macros.find_iter(latex)
            .map(|m| m.as_str().to_string())
            .collect();
        // The following is a whitelist of definitely non-problematic
        // macros.  I'm not sure when if ever we want to enforce only
        // macros on this whitelist.  For now I'm figuring to warn on
        // anything outside the list.  Ideally we'd have a list of macros
        // that pandoc understands and use that, but we also would need a
        // list of things MathJax understands, since pandoc can effectively
        // pass along any math symbols without understanding them, so long
        // as MathJax *does* understand them.
        let good_macros =
            &["begin", "end", "includegraphics", "columnwidth",
              "noindent", "textwidth", "item", "psi", "Psi", "textit",
              "\"o", "\"u", "&", "%", "left", "right", "frac", "pm", ";",
              ",", "text", "it", "em",
            ];
        for &m in good_macros {
            macros.remove(m);
        }
        // Unsupported macros.
        let bad_macros =
            &[ "section", "section*", // could mess up problem set layout
                "newcommand", "renewcommand", "newenvironment", // have namespacing issues
                "usepackage", // big can of worms
                "def", // namespacing issues?
                "cases", // old cases that doesn't work with amsmath
            ];
        for &m in bad_macros {
            if macros.contains(m) {
                refined.push_str(&format!(r#"\error{{bad macro: {}}}\\
"#, &m[1..]));
                macros.remove(m);
            }
        }
        for e in environments {
            refined.push_str(&format!(r#"\warning{{possibly bad environment: {}}}\\
"#, e));
        }
        for m in macros {
            refined.push_str(&format!(r#"\warning{{possibly bad macro: {}}}\\
"#, &m[1..]));
        }
        refined.push_str(&latex);
        latex = &refined;
    }
    let mut refined = String::with_capacity(latex.len());
    if args.solution {
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
    } else {
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
    }
    let latex = refined;

    match args._format {
        Format::HTML => {
            latex_snippet::html(&mut std::io::stdout(), &latex)?;
        }
        Format::PDF => {
            println!("have not yet implemented PDF output");
            std::process::exit(1);
        }
        Format::Latex => {
            use std::io::Write;
            std::io::stdout().write_all(latex.as_bytes())?;
        }
    }
    Ok(())
}
