use base::Span;

/// Describes some kind of problem or occurrence in the code. Contains one or
/// more remarks with descriptions and separate code spans.
///
/// This type doesn't provide a `Display` impl, since all spans reference an
/// external filemap which needs to be provided. Use `print` methods of the
/// `diag` module instead.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Report {
    /// Kind of the report (usually the same as the first remark kind)
    pub kind: ReportKind,
    /// Span of the main code snippet
    pub span: Option<Span>,
    /// List of remarks describing the report
    pub remarks: Vec<Remark>,
}

impl Report {
    /// Creates a error report with one message and one span
    pub fn simple_error<S: Into<String>>(msg: S, span: Span) -> Report {
        Report {
            kind: ReportKind::Error,
            span: Some(span),
            remarks: vec![Remark::error(msg, Snippet::Orig(span))],
        }
    }

    /// Creates a error report with one message, but without span
    pub fn simple_spanless_error<S: Into<String>>(msg: S) -> Report {
        Report {
            kind: ReportKind::Error,
            span: None,
            remarks: vec![Remark::error(msg, Snippet::None)],
        }
    }

    /// Creates a warning report with one message and one span
    pub fn simple_warning<S: Into<String>>(msg: S, span: Span) -> Report {
        Report {
            kind: ReportKind::Warning,
            span: Some(span),
            remarks: vec![Remark::warning(msg, Snippet::Orig(span))],
        }
    }

    /// Adds a note without a span/code snippet to the existing Report
    pub fn with_note<S: Into<String>>(self, msg: S) -> Report {
        self.with_remark(Remark::note(msg, Snippet::None))
    }

    /// Adds a note with a span/code snippet to the existing Report
    pub fn with_span_note<S: Into<String>>(self, msg: S, span: Span)
        -> Report
    {
        self.with_remark(Remark::note(msg, Snippet::Orig(span)))
    }

    /// Adds a remark to the returned Report
    pub fn with_remark(mut self, rem: Remark) -> Report {
        self.remarks.push(rem);
        self
    }
}

/// A report can either be an `Error` or a `Warning`. Still pretty similar to
/// `RemarkType` -- may be merged with it in the future.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReportKind {
    /// Something went very wrong and will stop further processing
    Error,
    /// Something important should be fixed, but doesn't stop processing
    Warning,
}

/// Part of a Report that describes the occurrence with an optional code
/// snippet.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Remark {
    pub kind: RemarkKind,
    /// Remark description
    pub desc: String,
    pub snippet: Snippet,
}

impl Remark {
    /// Creates a new remark with the given parameters
    pub fn new<S: Into<String>>(kind: RemarkKind, desc: S, snippet: Snippet)
        -> Self
    {
        Remark {
            kind: kind,
            desc: desc.into(),
            snippet: snippet,
        }
    }

    /// Creates a new remark of kind `Error` with the given parameters
    pub fn error<S: Into<String>>(desc: S, snippet: Snippet) -> Self {
        Self::new(RemarkKind::Error, desc, snippet)
    }

    /// Creates a new remark of kind `Warning` with the given parameters
    pub fn warning<S: Into<String>>(desc: S, snippet: Snippet) -> Self {
        Self::new(RemarkKind::Warning, desc, snippet)
    }

    /// Creates a new remark of kind `Note` with the given parameters
    pub fn note<S: Into<String>>(desc: S, snippet: Snippet) -> Self {
        Self::new(RemarkKind::Note, desc, snippet)
    }
}

/// Kinds of remarks
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RemarkKind {
    /// Something went very wrong and will stop further processing
    Error,
    /// Something important should be fixed, but doesn't stop processing
    Warning,
    /// Additional information about an error or a warning
    Note,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Snippet {
    /// No snippet
    None,
    /// Show the original code with this highlighted span
    Orig(Span),
    /// Show original code, but replace a part of it with something new and
    /// highlight the new part. Hint: also able to only insert.
    Replace {
        span: Span,
        with: String,
    }
}

impl Snippet {
    /// Returns the span if it exists
    pub fn span(&self) -> Option<Span> {
        match *self {
            Snippet::None => None,
            Snippet::Orig(span) => Some(span),
            Snippet::Replace { span, ..} => Some(span),
        }
    }
}
