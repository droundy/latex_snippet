use super::*;
use expect_test::expect;

#[test]
fn test_physics_macros() {
    assert_eq!(
        r"
\left\langle {foo}\right|
bar
\left|{baz}\right\rangle 

{d\hspace{-0.28em}\bar{}} X

\left(\frac{\partial {T}}{\partial {p}}\right)_{V}

\left(\frac{\partial {T}}{\partial {p}}\right)_{V}

",
        &physics_macros(
            r"
\bra{foo}
bar
\ket{baz}

\dbar X

\thermoderivative{T}{p}{V}

\myderiv{T}{p}{V}

"
        )
    );

    assert_eq!(
        r"  \left\langle {1}\middle|{0}\right\rangle   ",
        &physics_macros(r"  \bra{1}\ket{0}  ")
    );
}

#[test]
fn test_with_image_directory() {
    assert_eq!(
        r#"some stuff before <img src="images/foo"/> and after"#,
        &with_image_directory(
            r"some stuff before \includegraphics{foo} and after",
            "images/"
        )
    );
}

#[test]
fn test_section() {
    assert_eq!(
        "xx
<section><h2>foo</h2>
bar
</section><section><h2>baz</h2>
baz
</section>",
        &html_string(
            r"xx
\section{foo}
bar
\section{baz}
baz
"
        )
    );
}

#[test]
fn test_subsection() {
    assert_eq!(
        "xx
<section><h3>foo</h3>
bar
</section><section><h3>baz</h3>
baz
</section>",
        &html_string(
            r"xx
\subsection{foo}
bar
\subsection{baz}
baz
"
        )
    );

    assert_eq!(
        "xx
<section><h3>foo</h3>
bar
<section><h4>baz</h4>
baz
</section></section>",
        &html_string(
            r"xx
\subsection{foo}
bar
\subsubsection{baz}
baz
"
        )
    );

    assert_eq!(
        "xx
<section><h2>foo</h2>
bar
<section><h4>baz</h4>
baz
</section></section>",
        &html_string(
            r"xx
\section{foo}
bar
\subsubsection{baz}
baz
"
        )
    );
}

#[test]
fn test_subsubsection() {
    assert_eq!(
        "xx
<section><h4>foo</h4>
bar
</section><section><h4>baz</h4>
baz
</section>",
        &html_string(
            r"xx
\subsubsection{foo}
bar
\subsubsection{baz}
baz
"
        )
    );
    assert_eq!(
        "xx
<section><h4>foo</h4>
bar
</section><section><h3>baz</h3>
baz
</section>",
        &html_string(
            r"xx
\subsubsection{foo}
bar
\subsection{baz}
baz
"
        )
    );
}

#[test]
fn includesolutions() {
    println!("starting first test");
    assert_eq!(
        r#"
hello
\begin{quotation}\paragraph*{Solution}
the solution is here
\end{quotation}
foo"#,
        &include_solutions(
            r"
hello
\begin{solution}
the solution is here
\end{solution}
foo"
        )
    );
    println!("starting first test");

    assert_eq!(
        r#"
hello
\begin{quotation}\paragraph*{Solution}
the solution is here
\end{quotation}"#,
        &include_solutions(
            r"
hello
\begin{solution}
the solution is here
\end{solution}"
        )
    );
    println!("starting first test");

    assert_eq!(
        r#"
\begin{quotation}\paragraph*{Solution}
foo
\end{quotation}"#,
        &include_solutions(
            r"
\begin{solution}
foo
\end{solution}"
        )
    );
    println!("starting penultimate test");

    assert_eq!(
        r#"<section><h4>Solution</h4>
foobar
</section>"#,
        &html_string(
            r"\subsubsection*{Solution}
foobar
"
        )
    );
    println!("starting true penultimate test");

    assert_eq!(
        r#"<blockquote class="solution">
foo
</blockquote>"#,
        &html_string(
            r"\begin{solution}
foo
\end{solution}"
        )
    );
    println!("starting first test");

    assert_eq!(
        r#"<blockquote class="solution"><p>foo
</p></blockquote>"#,
        &html_string(
            r"\begin{solution}

foo
\end{solution}"
        )
    );
}

#[test]
fn subsection_in_solution() {
    assert_eq!(
        r#"<section><h2>Outer section</h2>
Before solution
<section><h3>Hello world</h3><blockquote class="solution">
In hello world
</blockquote>
After solution</section></section>"#,
        &html_string(
            r"\section{Outer section}
Before solution
\begin{solution}
\subsection{Hello world}
In hello world
\end{solution}
After solution"
        )
    );
}

#[test]
fn test_pull_sections_out() {
    assert_eq!(
        r"\section{Outer section}
Before solution
\begin{solution}
\end{solution}\subsection{Hello world}\begin{solution}
In hello world
\end{solution}
After solution",
        &pull_sections_out(
            r"\section{Outer section}
Before solution
\begin{solution}
\subsection{Hello world}
In hello world
\end{solution}
After solution"
        )
    );
}

#[test]
fn quotes() {
    assert_eq!(r#"foo “bar” baz"#, &html_string(r"foo ``bar'' baz"));
    assert_eq!(r#"\(a''\)"#, &html_string(r"$a''$"));
    assert_eq!(
        r#"\begin{align}a''\end{align}"#,
        &html_string(r"\begin{align}a''\end{align}")
    );
}

#[test]
fn curly_braces() {
    assert_eq!(r#"  \( \} \)  \{"#, &html_string(r" { $ \} $ } \{"));
}

#[test]
fn includegraphics() {
    assert_eq!(
        r#"hello<img src="filename"/>foo"#,
        &html_string(r"hello\includegraphics[width=\columnwidth]{filename}foo")
    );
}

#[test]
fn includegraphics_width() {
    assert_eq!(
        r#"hello<img style="width:30em" src="filename"/>"#,
        &html_string(r"hello\includegraphics[width=30em]{filename}")
    );
}

#[test]
fn figure() {
    assert_eq!(
        "hello<figure>foo</figure>",
        &html_string(r"hello\begin{figure}foo\end{figure}")
    );

    assert_eq!(
        "hello<figure>foo</figure>",
        &html_string(r"hello\begin{figure}[ht]foo\end{figure}")
    );

    assert_eq!(
        r#"hello<figure class="center">foo</figure>"#,
        &html_string(r"hello\begin{figure}\centering foo\end{figure}")
    );

    assert_eq!(
        r#"hello<figure> <div class="center">foo</div></figure>"#,
        &html_string(r"hello\begin{figure} \centering foo\end{figure}")
    );
}

#[test]
fn wrapfigure() {
    assert_eq!(
        r#"hello<figure class="wrapfigure" style="width:10em">foo</figure>"#,
        &html_string(r"hello\begin{wrapfigure}{r}{10em}foo\end{wrapfigure}")
    );
}

#[test]
fn figure_with_caption() {
    assert_eq!(
        "hello<figure>foo<figcaption>hello</figcaption></figure>",
        &html_string(r"hello\begin{figure}foo\caption{hello}\end{figure}")
    );
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
fn underline_hello() {
    assert_eq!("<u>hello</u>", &html_string(r"\underline{hello}"));
}
#[test]
fn textit_hello() {
    assert_eq!("<i>hello</i>", &html_string(r"\textit{hello}"));
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
        r"hello good \(\cos^2x\) math",
        &html_string(r"hello good $\cos^2x$ math")
    );
    assert_eq!(
        r"hello good \[\cos^2x\] math",
        &html_string(r"hello good $$\cos^2x$$ math")
    );
    assert_eq!(
        r"hello good \(\cos^2x\) math",
        &html_string(r"hello good \(\cos^2x\) math")
    );
    assert_eq!(
        r"hello good \[\cos^2x\] math",
        &html_string(r"hello good \[\cos^2x\] math")
    );
}
#[test]
fn escape_space() {
    assert_eq!(r"hello<i> world</i>", &html_string(r"hello\it\ world"));
}
#[test]
fn escape_pound() {
    assert_eq!(r"hello<i>#world</i>", &html_string(r"hello\it\#world"));
}
#[test]
fn escape_underscore() {
    assert_eq!(r"hello<i>_world</i>", &html_string(r"hello\it\_world"));
}
#[test]
fn escape_ampersand() {
    assert_eq!(
        r"hello<i>&amp; world</i>",
        &html_string(r"hello\it\& world")
    );
}
#[test]
fn escape_dollar() {
    expect![[r#"hello<i><span>$</span>world</i>"#]].assert_eq(&html_string(r"hello\it\$world"));
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
fn windows_newlines() {
    assert_eq!(
        "<p>Hello world\r\n\r\n</p><p>New paragraph</p>",
        &html_string("Hello world\r\n\r\nNew paragraph")
    );
    assert_eq!(
        "<p>Hello world\r\n  \r\n</p><p>New paragraph</p>",
        &html_string("Hello world\r\n  \r\nNew paragraph")
    );
    assert_eq!(
        "<p>Hello world\n\n</p><p>New paragraph</p>",
        &html_string("Hello world\n\nNew paragraph")
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
fn omit_stuff() {
    assert_eq!(
        r"xxx",
        &omit_handout(r"xx\begin{handout} good stuff \end{handout}x")
    );
    assert_eq!(
        r" good stuff ",
        &only_handout(r"xx\begin{handout} good stuff \end{handout}x")
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
fn definition() {
    let expected = expect![[r#"

        <dl><span class="error">
        buggy
        </span><dt>tasty</dt><dd> Apples
        </dd><dt>nice</dt><dd> Oranges
        </dd><dt>good for you</dt><dd> Vegetables
        <ol><li>Carrots
        </li><li>Potatotes
        </li></ol>
        </dd><dd>Pears
        </dd></dl>
        some more stuff
    "#]];
    expected.assert_eq(&html_string(
        r"
\begin{description}
buggy
\item[tasty] Apples
\item[nice] Oranges
\item[good for you] Vegetables
\begin{enumerate}
\item Carrots
\item Potatotes
\end{enumerate}
\item Pears
\end{description}
some more stuff
",
    ));

    let expected = expect![[r#"
        <dl><dt>tasty</dt><dd> Apples
        </dd><dd>Oranges
        </dd></dl>
        More valid text."#]];
    expected.assert_eq(&html_string(
        r"\begin{description}
\item[tasty] Apples
\item Oranges
\end{description}
More valid text.",
    ));
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
</li></ol><span class="error">MISSING \end{enumerate}</span>Pears
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
</li></ul><span class="error">MISSING \end{itemize}</span>Pears
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

#[test]
fn test_strip_comments() {
    assert_eq!(
        r"
this  is\% 
",
        &strip_comments(
            r"
this % comment
is\%% comment
% whole line comment

"
        )
    );
}

#[test]
fn tabular() {
    assert_eq!(
        r"
<table><tr><td>
foo </td><td> bar </td><td> baz </td></tr><tr><td>
extra </td><td> good
</td></tr></table>
",
        &html_string(
            r"
\begin{tabular}{ccc}
foo & bar & baz \\
extra & good
\end{tabular}
"
        )
    );
}

#[test]
fn test_ref() {
    assert_eq!(r" foo \ref{foo} bar", &html_string(r" foo \ref{foo} bar"));
}

#[test]
fn test_itemize_broken() {
    assert_eq!(
            "\n<ul></ul><span class=\"error\">MISSING \\end{itemize}</span>\n<ul></ul><span class=\"error\">MISSING \\end{itemize}</span>\n",
        &html_string(
            r"
\begin{itemize}
\begin{itemize}
")
    );
}

#[test]
fn paragraph_end_after_environment() {
    assert_eq!(
        "<div class=\"center\">\ncontents\n</div>\n\n",
        &html_string(
            r"\begin{center}
contents
\end{center}

"
        )
    );
}
