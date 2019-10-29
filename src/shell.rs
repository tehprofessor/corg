use std::collections::HashMap;
use std::fmt::{Arguments, Write as FmtWrite};
use std::io::{self, ErrorKind, Write};

use pulldown_cmark::Event::*;
use pulldown_cmark::{Alignment, CowStr, Event, LinkType, Tag};
use std::fmt;

enum TableState {
    Head,
    Body,
}

trait CorgTaggable {
    fn start_tag(&self) -> String;
    fn write_tag(&self, text: String) -> String;
    fn end_tag(&self) -> String;
    fn to_string(&self) -> String;
}

#[derive(Debug, PartialEq, Clone)]
struct CorgHeader {
    level: i32,
    close_before_start: bool,
    text: Option<String>,
}

impl CorgHeader {
    fn new(level: i32, previous_header: Option<CorgHeader>) -> CorgHeader {
        // If the new heading is level 2, and we have a previous heading,
        // we need to close the previous function
        let close_before_start = match (level, previous_header) {
            (2, Some(CorgHeader { level, .. })) if level == 2 => true,
            (2, Some(_)) => false,
            _ => false,
        };

        CorgHeader {
            level,
            close_before_start,
            text: None,
        }
    }

    /// Returns a string which can be used as a zsh function name
    fn function_slug(&self) -> String {
        let text = &self.text;

        match text {
            Some(slug) => slug.to_lowercase().replace(" ", "-"),
            _ => String::from(""),
        }
    }

    fn update_text(&mut self, text: Option<String>) {
        self.text = text
    }
}

impl CorgTaggable for CorgHeader {
    /// Returns text to be used for the start of a header. This function will also close
    /// any previous header that opens a shell function prepending the close to the output.
    fn start_tag(&self) -> String {
        let mut output = String::from("");

        // Check if we need to close the previous function
        if self.close_before_start {
            // Push the closing of the previous function
            let closing = String::from("}\n# - end function\n");
            output.push_str(closing.as_str());
        }

        // Now push the start of the heading
        match self.level {
            1 => output.push_str("corg_announce \"Running Document: "),
            2 => output.push_str("\n# - begin function:\n"),
            _ => output.push_str("\n# - start section:\n"),
        }

        String::from(output)
    }

    /// Returns the body of a header depending on its level:
    ///
    ///     - level-1 headings it closes the announce statement.
    ///     - level-2 headings it opens a shell function with the heading name.
    ///     - all other headings it returns an empty string.
    ///
    fn write_tag(&self, text: String) -> String {
        let header = CorgHeader {
            level: self.level,
            close_before_start: self.close_before_start,
            text: Some(text.clone()),
        };

        match header.level {
            1 => format!("{}\"\n\n", text.clone()),
            2 => format!("function {} {{\n", header.function_slug()),
            _ => String::from(""),
        }
    }

    fn end_tag(&self) -> String {
        match self.level {
            2 => String::from(""),
            _ => String::from(""),
        }
    }

    fn to_string(&self) -> String {
        format!(
            "CorgHeader {{ level: {}, close_before_start: {} }}",
            self.level, self.close_before_start
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
struct CorgCode;

impl CorgTaggable for CorgCode {
    fn start_tag(&self) -> String {
        String::from("")
    }

    fn write_tag(&self, text: String) -> String {
        let mut code = String::new();
        let mut lines: Vec<&str> = text.split("\n").collect();
        lines.reverse();
        while let Some(line) = lines.pop() {
            // If the line isn't empty we want to pad it.
            if line != "" {
                code.push_str("\t");
            }

            // If the line contains a heredoc we want to make sure it is padded too
            if line.contains("<< \"EOF\"") {
                println!("FOUND EOF w/ QUOTES");
                let padded_line = line.replace("<< \"EOF\"", "<< \"\tEOF\"");
                code.push_str(padded_line.as_str());
            } else if line.contains("<< 'EOF'") {
                println!("FOUND EOF w/ SINGLE QUOTES");
                let padded_line = line.replace("<< 'EOF'", "<< '\tEOF'");
                code.push_str(padded_line.as_str());
            } else {
                // Normal line no coddling needed
                code.push_str(line);
            }

            if lines.len() > 0 {
                code.push_str("\n");
            }
        }

        String::from(code)
    }

    fn end_tag(&self) -> String {
        String::from("")
    }

    fn to_string(&self) -> String {
        String::from("CorgCode {}")
    }
}

#[derive(Debug, PartialEq, Clone)]
struct CorgCodeBlock {
    lang: String,
}

impl CorgTaggable for CorgCodeBlock {
    fn start_tag(&self) -> String {
        String::from("# - begin code:\n")
    }

    /// Delegates to Corg Code
    fn write_tag(&self, text: String) -> String {
        let corg_code = CorgCode {};
        corg_code.write_tag(text)
    }

    fn end_tag(&self) -> String {
        String::from("")
    }

    fn to_string(&self) -> String {
        format!("CorgCodeBlock {{ lang: {} }}", self.lang)
    }
}

#[derive(Debug, PartialEq, Clone)]
struct CorgParagraph;

impl CorgTaggable for CorgParagraph {
    fn start_tag(&self) -> String {
        String::from("\n# - paragraph:\ncorg_debug \"")
    }

    fn write_tag(&self, text: String) -> String {
        format!("{}", text)
    }

    fn end_tag(&self) -> String {
        String::from("\"\n\n")
    }

    fn to_string(&self) -> String {
        String::from("CorgParagraph")
    }
}

impl std::fmt::Debug for Box<dyn CorgTaggable> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let farts = self.to_string();
        write!(f, "{}", farts)
    }
}

impl PartialEq for Box<dyn CorgTaggable> {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

/// This is probably a terrible design pattern but hey... I have no idea
/// what i'm doing.
#[derive(Debug, PartialEq)]
struct CorgStateManager {
    current_function_name: Option<String>,
    current_heading_level: Option<i32>,
    inside_codeblock: bool,
    current_indentation: String,
    function_names: Vec<String>,
    corg_tag: Option<Box<dyn CorgTaggable>>,
    header: Option<CorgHeader>,
    headers: Vec<CorgHeader>,
}

impl CorgStateManager {
    /// Returns a new empty instance of CorgStateManager
    fn new() -> CorgStateManager {
        CorgStateManager {
            current_function_name: None,
            current_heading_level: None,
            inside_codeblock: false,
            current_indentation: "".to_string(),
            function_names: vec![],
            corg_tag: None,
            header: None,
            headers: vec![],
        }
    }

    fn push_function_name(&mut self, function_name: String) {
        if let Some(mut header) = self.header.clone() {
            header.update_text(Some(function_name));
            self.current_function_name = Some(header.function_slug());
            self.function_names.push(header.function_slug());
            self.header = Some(header);
        }
    }

    fn update_tag(&mut self, new_tag: Box<dyn CorgTaggable>) {
        self.corg_tag = Some(new_tag);
    }

    fn update_header(&mut self, level: i32) {
        // Clone the previous header, this helps setup decisions on how
        // to close the sections they contain.
        let previous_header = self.header.clone();
        // Create a new CorgHeader instance using the current level and
        // previous header.
        let corg_header = CorgHeader::new(level, previous_header);
        // Clear any previous function names, we have to set it later.
        self.current_function_name = None;
        // Set the current header.
        self.header = Some(corg_header.clone());
        // Update the current tag.
        self.update_tag(Box::new(corg_header));
    }

    fn needs_to_push_function_name(&self) -> bool {
        match (&self.header, &self.current_function_name) {
            (Some(CorgHeader{level, ..}), None) if *level == 2 => true,
            _ => false
        }
    }
}

/// This wrapper exists because we can't have both a blanket implementation
/// for all types implementing `Write` and types of the for `&mut W` where
/// `W: StrWrite`. Since we need the latter a lot, we choose to wrap
/// `Write` types.
struct WriteWrapper<W>(W);

/// Trait that allows writing string slices. This is basically an extension
/// of `std::io::Write` in order to include `String`.
pub(crate) trait StrWrite {
    fn write_str(&mut self, s: &str) -> io::Result<()>;

    fn write_fmt(&mut self, args: Arguments) -> io::Result<()>;
}

impl<W> StrWrite for WriteWrapper<W>
where
    W: Write,
{
    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.0.write_all(s.as_bytes())
    }

    #[inline]
    fn write_fmt(&mut self, args: Arguments) -> io::Result<()> {
        self.0.write_fmt(args)
    }
}

impl<'w> StrWrite for String {
    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.push_str(s);
        Ok(())
    }

    #[inline]
    fn write_fmt(&mut self, args: Arguments) -> io::Result<()> {
        // FIXME: translate fmt error to io error?
        FmtWrite::write_fmt(self, args).map_err(|_| ErrorKind::Other.into())
    }
}

impl<W> StrWrite for &'_ mut W
where
    W: StrWrite,
{
    #[inline]
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        (**self).write_str(s)
    }

    #[inline]
    fn write_fmt(&mut self, args: Arguments) -> io::Result<()> {
        (**self).write_fmt(args)
    }
}

struct ShellWriter<'a, I, W> {
    /// Iterator supplying events.
    iter: I,

    /// Writer to write to.
    writer: W,

    /// Whether or not the last write wrote a newline.
    end_newline: bool,

    /// Hacky solution to writing shell files until I understand
    /// what the fuck this thing truly does.
    corg_state: CorgStateManager,

    table_state: TableState,
    table_alignments: Vec<Alignment>,
    table_cell_index: usize,
    numbers: HashMap<CowStr<'a>, usize>,
}

impl<'a, I, W> ShellWriter<'a, I, W>
where
    I: Iterator<Item = Event<'a>>,
    W: StrWrite,
{
    fn new(iter: I, writer: W) -> Self {
        let corg_state = CorgStateManager::new();

        Self {
            iter,
            writer,
            end_newline: true,
            corg_state: corg_state,
            table_state: TableState::Head,
            table_alignments: vec![],
            table_cell_index: 0,
            numbers: HashMap::new(),
        }
    }

    /// Writes a new line.
    fn write_newline(&mut self) -> io::Result<()> {
        self.end_newline = true;
        self.writer.write_str("\n")
    }

    /// Writes a buffer, and tracks whether or not a newline was written.
    #[inline]
    fn write(&mut self, s: &str) -> io::Result<()> {
        self.writer.write_str(s)?;

        if !s.is_empty() {
            self.end_newline = s.ends_with('\n');
        }
        Ok(())
    }

    pub fn run(mut self) -> io::Result<()> {
        while let Some(event) = self.iter.next() {
            match event {
                Start(tag) => {
                    self.start_tag(tag)?;
                }
                End(tag) => {
                    self.end_tag(tag)?;
                }
                Text(text) => {
                    let corg_state = &self.corg_state;
                    let corg_tag = &corg_state.corg_tag;
                    let output = match corg_tag {
                        Some(tag) => tag.write_tag(text.to_string()),
                        _ => text.to_string(),
                    };

                    if corg_state.needs_to_push_function_name() {
                        self.corg_state.push_function_name(text.to_string());
                    };

                    self.write(output.as_str())?;
                }
                Code(text) => {
                    self.write("\n# IS THIS CALLED\n")?;
                    self.write(&text)?;
                    self.write("")?;
                }
                Html(html) | InlineHtml(html) => {
                    self.write(&html)?;
                }
                SoftBreak => {
                    self.write_newline()?;
                }
                HardBreak => self.write("\n\n")?,
                FootnoteReference(name) => {
                    let len = self.numbers.len() + 1;
                    self.write("# -- note:\n# ")?;
                    self.write(&name)?;
                    let number = *self.numbers.entry(name).or_insert(len);
                    write!(&mut self.writer, " #{}", number)?;
                }
                TaskListMarker(true) => {
                    self.write("")?;
                }
                TaskListMarker(false) => {
                    self.write("")?;
                }
            }
        }
        // Close last function body
        self.write("\n}\n")?;
        // Grab all the function names we've created as a string.
        let function_names = self.corg_state.function_names.join("\n");
        let mut run_script_block = String::new();
        // Label the section in the output.
        run_script_block.push_str("\n# - run doc: \n");
        // Insert the function names in the block
        run_script_block.push_str(&function_names);
        // Write the function names to execute 'em
        self.write(&run_script_block)?;
        // Done
        Ok(())
    }

    /// Writes the start of an HTML tag.
    fn start_tag(&mut self, tag: Tag<'a>) -> io::Result<()> {
        match tag {
            Tag::Paragraph => {
                // Update our current corg tag.
                self.corg_state.update_tag(Box::new(CorgParagraph {}));
                // Now grab the output string.
                let maybe_paragraph = &self.corg_state.corg_tag;

                let output = match maybe_paragraph {
                    Some(corg_tag) => corg_tag.start_tag(),
                    _ => String::from(""),
                };
                // Don't write a newline
                self.end_newline = false;
                // Write the paragraph start to disk
                self.write(output.as_str())
            }
            Tag::Rule => {
                let rule_str = "# **************************************************************************** #";
                if self.end_newline {
                    self.write(&rule_str)
                } else {
                    self.write(&rule_str)?;
                    self.write("\n")
                }
            }
            Tag::Header(level) => {
                // Update the header
                self.corg_state.update_header(level);
                // Now get the output for the header.
                let maybe_header = &self.corg_state.header;
                let output = match maybe_header {
                    Some(header) => header.start_tag(),
                    _ => String::from(""),
                };
                // Make sure a newline is appended to the output
                self.end_newline = true;
                // Write it to disk
                self.write(output.as_str())
            }
            Tag::Table(alignments) => {
                self.table_alignments = alignments;
                self.write("")
            }
            Tag::TableHead => {
                self.table_state = TableState::Head;
                self.table_cell_index = 0;
                self.write("")
            }
            Tag::TableRow => {
                self.table_cell_index = 0;
                self.write("")
            }
            Tag::TableCell => {
                match self.table_state {
                    TableState::Head => {
                        self.write("")?;
                        // self.write("<th")?;
                    }
                    TableState::Body => {
                        self.write("")?;
                        // self.write("<td")?;
                    }
                }
                match self.table_alignments.get(self.table_cell_index) {
                    // Some(&Alignment::Left) => self.write(" align=\"left\">"),
                    // Some(&Alignment::Center) => self.write(" align=\"center\">"),
                    // Some(&Alignment::Right) => self.write(" align=\"right\">"),
                    _ => self.write(""),
                }
            }
            Tag::BlockQuote => {
                if self.end_newline {
                    self.write("# block quote")?;
                    self.write("corg_info \n")
                } else {
                    self.write("# block quote")?;
                    self.write("\ncorg_info \n")
                }
            }
            Tag::CodeBlock(info) => {
                if !self.end_newline {
                    self.write_newline()?;
                }

                // Extract the language
                let lang = info.split(' ').next().unwrap();
                let code_lang = lang.to_string();
                // Update the current tag
                self.corg_state
                    .update_tag(Box::new(CorgCodeBlock { lang: code_lang }));
                // Grab the new tag
                let maybe_code_block = &self.corg_state.corg_tag;
                let output = match maybe_code_block {
                    Some(code_block) => code_block.start_tag(),
                    _ => String::from(""),
                };

                // Write the code to disk
                self.write(output.as_str())
            }
            Tag::List(Some(1)) => {
                if self.end_newline {
                    self.write("# List \n")
                } else {
                    self.write("\n# List\n")
                }
            }
            Tag::List(Some(_start)) => self.write("#"),
            Tag::List(None) => {
                if self.end_newline {
                    self.write("# List (None)\n")
                } else {
                    self.write("\n# List (None)\n")
                }
            }
            Tag::Item => {
                if self.end_newline {
                    self.write("# -")
                } else {
                    self.write("\n# -")
                }
            }
            Tag::Emphasis => self.write(""),
            Tag::Strong => self.write(""),
            Tag::Strikethrough => self.write(""),
            Tag::Link(LinkType::Email, _dest, _title) => self.write(""),
            Tag::Link(_link_type, _dest, _title) => self.write(""),
            Tag::Image(_link_type, _dest, _title) => self.write(""),
            Tag::FootnoteDefinition(name) => {
                if self.end_newline {
                    self.write("# ")?;
                } else {
                    self.write("\n# ")?;
                }
                self.write(&name)?;
                self.write(" - ")?;
                let len = self.numbers.len() + 1;
                let number = *self.numbers.entry(name).or_insert(len);
                write!(&mut self.writer, "{}", number)?;
                self.write("\n")
            }
            Tag::HtmlBlock => Ok(()),
        }
    }

    fn end_tag(&mut self, tag: Tag) -> io::Result<()> {
        match tag {
            Tag::Paragraph => {
                let output = match &self.corg_state.corg_tag {
                    Some(paragraph) => paragraph.end_tag(),
                    _ => String::from(""),
                };

                self.write(output.as_str())?;
            }
            Tag::Rule => (),
            Tag::Header(_level) => {
                let output = match &self.corg_state.corg_tag {
                    Some(header) => header.end_tag(),
                    _ => String::from(""),
                };

                self.write(output.as_str())?;
            }
            Tag::Table(_) => {
                self.write("")?;
            }
            Tag::TableHead => {
                self.write("")?;
                self.table_state = TableState::Body;
            }
            Tag::TableRow => {
                self.write("")?;
            }
            Tag::TableCell => {
                match self.table_state {
                    TableState::Head => {
                        self.write("")?;
                    }
                    TableState::Body => {
                        self.write("")?;
                    }
                }

                self.table_cell_index += 1;
            }
            Tag::BlockQuote => {
                self.write("")?;
            }
            Tag::CodeBlock(_code_lang) => {
                let output = match &self.corg_state.corg_tag {
                    Some(code_block) => code_block.end_tag(),
                    _ => String::from(""),
                };

                self.write(output.as_str())?;
            }
            Tag::List(Some(_)) => {
                self.write("")?;
            }
            Tag::List(None) => {
                self.write("")?;
            }
            Tag::Item => {
                self.write("")?;
            }
            Tag::Emphasis => {
                self.write("")?;
            }
            Tag::Strong => {
                self.write("")?;
            }
            Tag::Strikethrough => {
                self.write("")?;
            }
            Tag::Link(_, _, _) => {
                self.write("")?;
            }
            Tag::Image(_, _, _) => (), // shouldn't happen, handled in start
            Tag::FootnoteDefinition(_) => {
                self.write("")?;
            }
            Tag::HtmlBlock => {}
        }
        Ok(())
    }
}

/// Iterate over an `Iterator` of `Event`s, generate HTML for each `Event`, and
/// push it to a `String`.
///
/// # Examples
///
/// ```
/// use pulldown_cmark::{html, Parser};
///
/// let markdown_str = r#"
/// hello
/// =====
///
/// * alpha
/// * beta
/// "#;
/// let parser = Parser::new(markdown_str);
///
/// let mut html_buf = String::new();
/// html::push_html(&mut html_buf, parser);
///
/// assert_eq!(html_buf, r#"<h1>hello</h1>
/// <ul>
/// <li>alpha</li>
/// <li>beta</li>
/// </ul>
/// "#);
/// ```

/// CORG-NOTE:
/// push_shell is probably useful if you need to build up a buffer from multiple
/// markdown files.
///
pub fn push_shell<'a, I>(s: &mut String, iter: I)
where
    I: Iterator<Item = Event<'a>>,
{
    ShellWriter::new(iter, s).run().unwrap();
}
/// :CORG-NOTE

/// Iterate over an `Iterator` of `Event`s, generate HTML for each `Event`, and
/// write it out to a writable stream.
///
/// **Note**: using this function with an unbuffered writer like a file or socket
/// will result in poor performance. Wrap these in a
/// [`BufWriter`](https://doc.rust-lang.org/std/io/struct.BufWriter.html) to
/// prevent unnecessary slowdowns.
///
/// # Examples
///
/// ```
/// use pulldown_cmark::{html, Parser};
/// use std::io::Cursor;
///
/// let markdown_str = r#"
/// hello
/// =====
///
/// * alpha
/// * beta
/// "#;
/// let mut bytes = Vec::new();
/// let parser = Parser::new(markdown_str);
///
/// html::write_html(Cursor::new(&mut bytes), parser);
///
/// assert_eq!(&String::from_utf8_lossy(&bytes)[..], r#"<h1>hello</h1>
/// <ul>
/// <li>alpha</li>
/// <li>beta</li>
/// </ul>
/// "#);
/// ```
pub fn write_html<'a, I, W>(writer: W, iter: I) -> io::Result<()>
where
    I: Iterator<Item = Event<'a>>,
    W: Write,
{
    ShellWriter::new(iter, WriteWrapper(writer)).run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corg_new() {
        let expected = CorgStateManager {
            current_function_name: None,
            current_heading_level: None,
            inside_codeblock: false,
            current_indentation: "".to_string(),
            function_names: vec![],
            corg_tag: None,
            header: None,
            headers: vec![]
        };

        let actual = CorgStateManager::new();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_corg_header_function_slug() {

        let header_text = String::from("Holla Cheese Burgers");
        let corg_header = CorgHeader { level: 2, close_before_start: false, text: Some(header_text) };

        let actual = corg_header.function_slug();

        assert_eq!(actual, "holla-cheese-burgers".to_string());
    }

    #[test]
    fn test_corg_needs_to_push_function_name() {
        let mut ksm = CorgStateManager::new();

        ksm.header = Some(CorgHeader { level: 2, close_before_start: false, text: None });
        ksm.current_heading_level = Some(2);

        assert_eq!(
            ksm.needs_to_push_function_name(),
            true
        )
    }

    #[test]
    fn test_corg_push_function_name() {
        let mut ksm = CorgStateManager::new();
        ksm.header = Some(CorgHeader{ level: 2, close_before_start: false, text: None});

        ksm.push_function_name("shit".to_string());

        assert_eq!(ksm.current_function_name, Some("shit".to_string()))
    }
}
