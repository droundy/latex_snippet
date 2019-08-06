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
