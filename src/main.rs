use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};
use std::{
    error::Error,
    io::{self, Stdout},
    ops::AddAssign,
};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

#[derive(Clone)]
struct Node {
    name: String,
    children: Vec<Node>,
}

impl Node {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            children: Vec::new(),
        }
    }
}

enum AppMode {
    Tree,
}

struct App {
    tree: Vec<Node>,
    selected: Option<Vec<usize>>,
    mode: AppMode,
    prev_key_code: Option<KeyCode>,
}

impl App {
    fn new() -> Self {
        Self {
            tree: Vec::new(),
            selected: None,
            mode: AppMode::Tree,
            prev_key_code: None,
        }
    }

    fn add_child(&mut self) {
        let new_node = Node::new("new node");
        if let Some(selected_path) = self.selected.clone() {
            if let Some(selected_node) = self.get_node_mut(&selected_path) {
                selected_node.children.push(new_node);
                let mut new_path = selected_path;
                new_path.push(selected_node.children.len() - 1);
                self.selected = Some(new_path);
            }
        } else {
            self.tree.push(new_node);
            self.selected = Some(vec![self.tree.len() - 1]);
        }
    }

    fn get_node_mut(&mut self, path: &[usize]) -> Option<&mut Node> {
        let mut current_nodes = &mut self.tree;
        for (i, &index) in path.iter().enumerate() {
            if i == path.len() - 1 {
                return current_nodes.get_mut(index);
            }
            match current_nodes.get_mut(index) {
                Some(node) => current_nodes = &mut node.children,
                None => return None,
            }
        }
        None
    }

    fn get_parent_children_mut(&mut self, path: &[usize]) -> Option<&mut Vec<Node>> {
        if path.is_empty() {
            return Some(&mut self.tree);
        }
        let mut current_level = &mut self.tree;
        for &index in path {
            if let Some(node) = current_level.get_mut(index) {
                current_level = &mut node.children;
            } else {
                return None;
            }
        }
        Some(current_level)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new();
    run(&mut terminal, &mut app)?;
    restore_terminal(&mut terminal)?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal, Box<dyn Error>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Terminal) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(terminal.show_cursor()?)
}

fn run(terminal: &mut Terminal, app: &mut App) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| ui(f, app))?;
        if let Event::Key(key) = event::read()? {
            match app.mode {
                AppMode::Tree => {
                    let prev_key = app.prev_key_code.take(); // Take ownership and leave None
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('A') => {
                            app.add_child();
                        }
                        KeyCode::Char('j') => {
                            if let Some(selected) = app.selected.clone() {
                                if selected.is_empty() {
                                    continue;
                                }
                                let parent_path = &selected[..selected.len() - 1];
                                if let Some(parent_nodes) = app.get_parent_children_mut(parent_path)
                                {
                                    let current_index = *selected.last().unwrap();
                                    if current_index < parent_nodes.len() - 1 {
                                        app.selected
                                            .as_mut()
                                            .unwrap()
                                            .last_mut()
                                            .unwrap()
                                            .add_assign(1);
                                    }
                                }
                            }
                        }
                        KeyCode::Char('k') => {
                            if let Some(selected) = &mut app.selected {
                                if !selected.is_empty() {
                                    let current_index = *selected.last().unwrap();
                                    if current_index > 0 {
                                        *selected.last_mut().unwrap() -= 1;
                                    }
                                }
                            }
                        }
                        KeyCode::Char('g') => {
                            if let Some(KeyCode::Char('g')) = prev_key {
                                if !app.tree.is_empty() {
                                    app.selected = Some(vec![0]);
                                }
                            } else {
                                app.prev_key_code = Some(key.code);
                            }
                        }
                        KeyCode::Char('G') => {
                            let mut path = Vec::new();
                            let mut current_nodes = &app.tree;
                            if !current_nodes.is_empty() {
                                let mut last_index = current_nodes.len() - 1;
                                path.push(last_index);
                                current_nodes = &current_nodes[last_index].children;
                                while !current_nodes.is_empty() {
                                    last_index = current_nodes.len() - 1;
                                    path.push(last_index);
                                    current_nodes = &current_nodes[last_index].children;
                                }
                            }

                            if !path.is_empty() {
                                app.selected = Some(path);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui(frame: &mut Frame, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(frame.size());

    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(main_layout[0]);

    let mut lines = Vec::new();
    build_tree_lines(&app.tree, &app.selected, vec![], &mut lines);
    let tree_widget = Paragraph::new(lines).block(Block::default().title("Tree"));

    frame.render_widget(tree_widget, inner_layout[0]);
    frame.render_widget(
        Block::default(),
        inner_layout[1],
    );

    let mode_text = match app.mode {
        AppMode::Tree => "Tree Mode",
    };

    frame.render_widget(
        Paragraph::new(mode_text).block(Block::default()),
        main_layout[1],
    );
}

fn build_tree_lines(nodes: &[Node], selected: &Option<Vec<usize>>, path: Vec<usize>, lines: &mut Vec<Line>) {
    for (i, node) in nodes.iter().enumerate() {
        let mut current_path = path.clone();
        current_path.push(i);

        let is_selected = if let Some(sel) = selected {
            sel == &current_path
        } else {
            false
        };

        let style = if is_selected {
            Style::default().fg(Color::Black).bg(Color::White)
        } else {
            Style::default()
        };

        let prefix = " ".repeat(path.len() * 2);
        lines.push(Line::from(Span::styled(format!("{}{}", prefix, node.name), style)));

        if !node.children.is_empty() {
            build_tree_lines(&node.children, selected, current_path, lines);
        }
    }
}
