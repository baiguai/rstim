use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Tree,
    Normal,
    Insert,
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: Uuid,
    pub name: String,
    pub content: Option<String>,
    pub children: Vec<Uuid>,
    pub parent: Option<Uuid>,
}

impl TreeNode {
    pub fn new(name: String, content: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            content,
            children: Vec::new(),
            parent: None,
        }
    }

    pub fn is_folder(&self) -> bool {
        self.content.is_none()
    }
}

#[derive(Debug)]
pub struct App {
    pub mode: Mode,
    pub nodes: HashMap<Uuid, TreeNode>,
    pub selected_node: Option<Uuid>,
    pub tree_focused: bool,
}

impl App {
    pub fn new() -> Self {
        let app = Self {
            mode: Mode::Tree,
            nodes: HashMap::new(),
            selected_node: None,
            tree_focused: true,
        };
        app
    }

    pub fn add_sibling_node(&mut self) {
        let new_node = TreeNode::new("New Node".to_string(), None);
        let new_id = new_node.id;
        
        if let Some(selected_id) = self.selected_node {
            if let Some(selected_node) = self.nodes.get(&selected_id) {
                if let Some(parent_id) = selected_node.parent {
                    if let Some(parent) = self.nodes.get_mut(&parent_id) {
                        parent.children.push(new_id);
                    }
                    let mut node = new_node;
                    node.parent = Some(parent_id);
                    self.nodes.insert(new_id, node);
                } else {
                    self.nodes.insert(new_id, new_node);
                }
            } else {
                self.nodes.insert(new_id, new_node);
            }
        } else {
            self.nodes.insert(new_id, new_node);
        }
        
        self.selected_node = Some(new_id);
    }

    pub fn add_child_node(&mut self) {
        let new_node = TreeNode::new("New Node".to_string(), None);
        let new_id = new_node.id;
        
        if let Some(selected_id) = self.selected_node {
            if let Some(selected_node) = self.nodes.get_mut(&selected_id) {
                selected_node.children.push(new_id);
                let mut node = new_node;
                node.parent = Some(selected_id);
                self.nodes.insert(new_id, node);
            }
        } else {
            self.nodes.insert(new_id, new_node);
        }
        
        self.selected_node = Some(new_id);
    }

    fn render_tree(&self, area: Rect, f: &mut Frame) {
        let items: Vec<ListItem> = self.nodes
            .values()
            .map(|node| {
                let icon = if node.is_folder() { "📁 " } else { "📄 " };
                let style = if Some(node.id) == self.selected_node {
                    Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(Span::styled(
                    format!("{}{}", icon, node.name),
                    style,
                )))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)))
            .style(Style::default().bg(Color::Black).fg(Color::White));

        f.render_widget(list, area);
    }

    fn render_editor(&self, area: Rect, f: &mut Frame) {
        let content = if let Some(selected_id) = self.selected_node {
            if let Some(node) = self.nodes.get(&selected_id) {
                node.content.as_deref().unwrap_or("")
            } else {
                ""
            }
        } else {
            ""
        };

        let paragraph = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)))
            .style(Style::default().bg(Color::Black).fg(Color::White));

        f.render_widget(paragraph, area);
    }

    fn render_status(&self, area: Rect, f: &mut Frame) {
        let mode_text = match self.mode {
            Mode::Tree => "TREE",
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
        };

        let status_line = Line::from(vec![
            Span::styled(
                format!(" {} ", mode_text),
                Style::default().bg(Color::Blue).fg(Color::Black).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                format!("| Nodes: {}", self.nodes.len()),
                Style::default().fg(Color::Gray),
            ),
        ]);

        let paragraph = Paragraph::new(status_line)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)))
            .style(Style::default().bg(Color::Black).fg(Color::White));

        f.render_widget(paragraph, area);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(size);

            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(70),
                ])
                .split(chunks[0]);

            app.render_tree(main_chunks[0], f);
            app.render_editor(main_chunks[1], f);
            app.render_status(chunks[1], f);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.mode {
                    Mode::Tree => {
                        match key.code {
                            KeyCode::Char('a') => app.add_sibling_node(),
                            KeyCode::Char('A') => app.add_child_node(),
                            KeyCode::Char('q') => break,
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new();
        assert_eq!(app.mode, Mode::Tree);
        assert!(app.nodes.is_empty());
        assert!(app.selected_node.is_none());
    }

    #[test]
    fn test_add_sibling_node_empty() {
        let mut app = App::new();
        app.add_sibling_node();
        assert_eq!(app.nodes.len(), 1);
        assert!(app.selected_node.is_some());
    }

    #[test]
    fn test_add_child_node_empty() {
        let mut app = App::new();
        app.add_child_node();
        assert_eq!(app.nodes.len(), 1);
        assert!(app.selected_node.is_some());
    }

    #[test]
    fn test_node_is_folder() {
        let folder = TreeNode::new("Folder".to_string(), None);
        assert!(folder.is_folder());

        let note = TreeNode::new("Note".to_string(), Some("content".to_string()));
        assert!(!note.is_folder());
    }
}