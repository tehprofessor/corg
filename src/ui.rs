use std::io;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle};
use tui::widgets::{Block, Borders, Paragraph, Row, Table, Text, Widget};
use tui::{Frame, Terminal};


pub struct Title<'a> {
    pub text: Text<'a>,
    pub label: &'a str,
    pub style: Style,
    pub borders: Borders,
}

impl<'a> Title<'a> {
    pub fn new(text: &'a str, label: &'a str) -> Title<'a> {
        let text = Text::styled(text, Style::default().fg(Color::Cyan));

        Self::with_text(text, label)
    }

    pub fn with_text(text: Text<'a>, label: &'a str) -> Title<'a> {
        Self {
            text,
            label,
            style: Style::default().fg(Color::Magenta).modifier(Modifier::BOLD),
            borders: Borders::ALL
        }
    }
}

pub struct Host<'a> {
    pub name: &'a str,
    pub fqdn: &'a str,
    pub scheme: &'a str,
}

pub struct Document<'a> {
    pub file_name: &'a str,
    pub modified_at: &'a str,
    pub sha: &'a str,
}

impl<'a> Document<'a> {
    pub fn new(file_name: &'a str, modified_at: &'a str, sha: &'a str) -> Document<'a> {
        Self {
            file_name,
            modified_at,
            sha,
        }
    }
}

pub struct ListState<I> {
    pub items: Vec<I>,
    pub selected: usize,
}

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub documents: Vec<Document<'a>>,
    pub servers: Vec<Host<'a>>,
    pub commands: Vec<String>,
    pub input: String,
}

impl<'a> App<'a> {
    pub fn demo() -> App<'a> {
        let servers = vec![
            Host {
                name: "faye",
                fqdn: "faye.futuregadgetlab.dev",
                scheme: "key",
            },
            Host {
                name: "microwave",
                fqdn: "microwave.futuregadgetlab.dev",
                scheme: "key",
            },
            Host {
                name: "janet",
                fqdn: "janet.futuregadgetlab.dev",
                scheme: "key",
            },
            Host {
                name: "pdns",
                fqdn: "pdns.futuregadgetlab.dev",
                scheme: "key",
            },
        ];

        let documents = vec![
            Document::new("faye.md", "2019-08-28T19:04:07.450466Z", "abc"),
            Document::new("janet.md", "2019-08-16T12:02:07.321466Z", "abc"),
            Document::new("microwave.md", "2019-08-14T17:33:24.101466Z", "abc"),
            Document::new("nginx.md", "2019-08-14T17:33:24.101466Z", "abc"),
        ];

        Self::new("file.md", documents, servers)
    }

    pub fn new(title: &'a str, documents: Vec<Document<'a>>, servers: Vec<Host<'a>>) -> App<'a> {
        Self {
            title,
            should_quit: false,
            documents,
            servers,
            commands: vec![],
            input: String::new(),
        }
    }

    pub fn on_key(&mut self, key: char) {
        match key {
            'q' => self.should_quit = true,
            _ => (),
        }
    }
}

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &App) -> Result<(), io::Error> {
    terminal.draw(|mut f| {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(15), Constraint::Percentage(85)].as_ref())
            .direction(Direction::Vertical)
            .split(f.size());

        let title = Title::new(&app.input, "Document");

        draw_title(&mut f, title, chunks[0]);
        draw_file_browser(&mut f, &app, chunks[1]);
    })
}

fn draw_title<B: Backend>(f: &mut Frame<B>, title: Title, area: Rect) {
    Paragraph::new([title.text].iter())
        .block(
            Block::default()
                .title(title.label)
                .title_style(title.style)
                .borders(title.borders),
        )
        .wrap(true)
        .render(f, area)
}

pub fn draw_file_browser<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let header = ["Name", "Modified", "SHA"];
    let rows = app.documents.iter().map(|s| {
        let style = Style::default().fg(Color::LightGreen);
        Row::StyledData(vec![s.file_name, s.modified_at, s.sha].into_iter(), style)
    });

    Table::new(header.into_iter(), rows)
        .block(Block::default().title("Documents").borders(Borders::ALL))
        .header_style(Style::default().fg(Color::Yellow))
        .widths(&[15, 15, 10])
        .render(f, area);

    //    let text = [
    //        Text::raw("This is a paragraph with several lines. You can change style your text the way you want.\n\nFox example: "),
    //        Text::styled("under", Style::default().fg(Color::Red)),
    //        Text::raw(" "),
    //        Text::styled("the", Style::default().fg(Color::Green)),
    //        Text::raw(" "),
    //        Text::styled("rainbow", Style::default().fg(Color::Blue)),
    //        Text::raw(".\nOh and if you didn't "),
    //        Text::styled("notice", Style::default().modifier(Modifier::ITALIC)),
    //        Text::raw(" you can "),
    //        Text::styled("automatically", Style::default().modifier(Modifier::BOLD)),
    //        Text::raw(" "),
    //        Text::styled("wrap", Style::default().modifier(Modifier::REVERSED)),
    //        Text::raw(" your "),
    //        Text::styled("text", Style::default().modifier(Modifier::UNDERLINED)),
    //        Text::raw(".\nOne more thing is that it should display unicode characters: 10€")
    //    ];
}
