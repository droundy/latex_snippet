use super::*;

#[test]
fn test_physics_macros() {
    assert_eq!(
        r"
\langle {foo}|
bar
|{baz}\rangle 

{d\hspace{-0.28em}\bar{}} X

\left(\frac{T}{p}\right)_{V}

\left( % \myderiv
\frac{T}{p}\right)_{V}

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
    assert_eq!(
        r#"
hello
\paragraph*{Solution}{\it 
the solution is here
}


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

    assert_eq!(
        r#"
hello
\paragraph*{Solution}{\it 
the solution is here
}

"#,
        &include_solutions(
            r"
hello
\begin{solution}
the solution is here
\end{solution}"
        )
    );

    assert_eq!(
        r#"
\paragraph*{Solution}{\it 
foo
}

"#,
        &include_solutions(
            r"
\begin{solution}
foo
\end{solution}"
        )
    );

    assert_eq!(
        r#"<h5>Solution</h5>foo
"#,
        &html_string(
            r"\paragraph*{Solution}
foo
"
        )
    );

    assert_eq!(
        r#"
<h5>Solution</h5><i>
foo
</i>

"#,
        &html_string(&include_solutions(
            r"
\begin{solution}
foo
\end{solution}"
        ))
    );
}

#[test]
fn quotes() {
    assert_eq!(r#"foo “bar” baz"#, &html_string(r"foo ``bar'' baz"));
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
    assert_eq!(r"hello<i>$world</i>", &html_string(r"hello\it\$world"));
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
    assert_eq!(
        r" foo \ref{foo} bar",
        &html_string(
            r" foo \ref{foo} bar"
        )
    );
}
