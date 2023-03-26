\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{resume}[2016/01/04 My custom resume class]
\LoadClass{article}

\RequirePackage{titlesec}
\RequirePackage{scrextend}

\titleformat{\section}          % Customize the /section command
    {
        \Large
        \scshape
        \raggedright
    }                           % Make the \section headers large (\Large)
                                % small capitals (\scshape) and left aligned (\raggedright)
    {}{0em}                     % Can be used to give a prefix to all sections, like 'Section ...'
    {}                          % Can be used to insert code before the heading
    [\titlerule]                % Inserts a horizontal line after the heading

\titleformat{\subsection}
    {
        \large
        \raggedright
    }
    {}{0em}
    {}

\titleformat{\subsubsection}
    {
        \raggedright
    }
    {}{0em}
    {}

\newcommand{\datedsection}[2]{%
    \section[#1]{#1 \hfill #2}%
}

\newcommand{\datedsubsection}[2]{%
    \subsection[#1]{#1 \hfill #2}%
}

\newcommand{\name}[1]{
    \centerline{\Huge{#1}}
}

\newcommand{\contact}[6]{
    \centerline{#1 {\large\textperiodcentered} #2 {\large\textperiodcentered} #3 {\large}}
    
    \centerline{{\Large} #4 \ {\Large} #5 \ {\Large} #6}
}

\newcommand{\expitems}[3]{
    \begin{itemize}
    \setlength\itemsep{0em}
    \rmfamily
    \item #1
    \item #2
    \item #3
    \end{itemize}
}

\newcommand{\locationsubsection}[2]{
    \subsection[#1]{#1 \hfill \textit{#2}}   % organization and location
}

\newcommand{\expblock}[3]{
    \begin{addmargin}[1em]{1em}
    \subsubsection[#1]{\textbf{#1} \hfill #2}% postion and date
    %\expitems{#3}{#4}{#5}
    \begin{itemize} 
    \setlength\itemsep{0em}
    \rmfamily
    #3
    \end{itemize}
    \end{addmargin}
}
