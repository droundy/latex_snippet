use auto_args::AutoArgs;

use std::io::Write;

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
            .args(&[
                "-u",
                "osubash",
                "-ss",
                "-N",
                "-H",
                "-e",
                &format!(
                    "select problem_latex from osu_production.admin_app_problem where id = {}",
                    pk
                ),
            ])
            .output()
            .expect("failed to execute process");
        latex = String::from_utf8_lossy(&output.stdout[28..output.stdout.len() - 18]).to_string();
    } else {
        use std::io::Read;
        std::io::stdin().read_to_string(&mut latex)?;
    }
    if args.check {
        latex = latex_snippet::check_latex(&latex);
    }
    latex = if args.solution {
        latex_snippet::include_solutions(&latex)
    } else {
        latex_snippet::omit_solutions(&latex)
    };

    match args._format {
        Format::HTML => {
            latex_snippet::html(&mut std::io::stdout(), &latex)?;
        }
        Format::PDF => {
            let dir = tempfile::tempdir()?;
            std::env::set_current_dir(&dir)?;
            let mut contents = String::new();
            use std::fmt::Write;
            contents.write_str(
                r"\documentclass{article}

\usepackage{amsmath}
\usepackage{fullpage}
\usepackage{color}
\newcommand\error[1]{\textcolor{red}{\it #1}}
\newcommand\warning[1]{\textcolor{blue}{\it #1}}

\begin{document}
\title{A single problem}
\maketitle
",
            )?;
            contents.write_str(&latex)?;
            contents.write_str(
                r"
\end{document}
",
            )?;
            std::fs::write("problem.tex", &contents)?;
            std::process::Command::new("pdflatex")
                .args(&["problem.tex"])
                .stderr(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .status()?;
            std::process::Command::new("pdflatex")
                .args(&["problem.tex"])
                .stderr(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .status()?;
            let mut file = std::fs::File::open("problem.pdf")?;
            std::io::copy(&mut file, &mut std::io::stdout())?;
        }
        Format::Latex => {
            std::io::stdout().write_all(latex.as_bytes())?;
        }
    }
    Ok(())
}
