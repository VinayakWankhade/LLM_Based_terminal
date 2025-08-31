use crate::ansi::{AnsiParser, AnsiCommand, CharAttributes, CursorPosition};
use crate::pty::{PtyManager, TerminalSize, TerminalOutput};
use crate::shell_hooks::ShellHooksManager;
use crate::search::{SearchIndexManager, ScrollMatch, ContextLine};
use crate::ai::AiContext;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalChar {
    pub character: char,
    pub attributes: CharAttributes,
}

impl Default for TerminalChar {
    fn default() -> Self {
        TerminalChar {
            character: ' ',
            attributes: CharAttributes::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TerminalGrid {
    pub rows: Vec<Vec<TerminalChar>>,
    pub cols: usize,
    pub cursor: CursorPosition,
    pub saved_cursor: Option<CursorPosition>,
}

impl TerminalGrid {
    pub fn new(cols: usize, rows: usize) -> Self {
        let grid_rows = (0..rows)
            .map(|_| vec![TerminalChar::default(); cols])
            .collect();

        TerminalGrid {
            rows: grid_rows,
            cols,
            cursor: CursorPosition { row: 0, col: 0 },
            saved_cursor: None,
        }
    }

    pub fn resize(&mut self, new_cols: usize, new_rows: usize) {
        // Resize existing rows
        for row in &mut self.rows {
            row.resize(new_cols, TerminalChar::default());
        }

        // Add or remove rows
        if new_rows > self.rows.len() {
            for _ in self.rows.len()..new_rows {
                self.rows.push(vec![TerminalChar::default(); new_cols]);
            }
        } else {
            self.rows.truncate(new_rows);
        }

        self.cols = new_cols;

        // Ensure cursor is within bounds
        self.cursor.row = self.cursor.row.min(new_rows as u16 - 1);
        self.cursor.col = self.cursor.col.min(new_cols as u16 - 1);
    }

    pub fn write_char(&mut self, ch: char, attributes: &CharAttributes) {
        if self.cursor.row as usize >= self.rows.len() {
            return;
        }

        let row = &mut self.rows[self.cursor.row as usize];
        if (self.cursor.col as usize) < row.len() {
            row[self.cursor.col as usize] = TerminalChar {
                character: ch,
                attributes: attributes.clone(),
            };
            self.cursor.col += 1;

            // Wrap to next line if needed
            if self.cursor.col as usize >= self.cols {
                self.cursor.col = 0;
                if (self.cursor.row as usize) < self.rows.len() - 1 {
                    self.cursor.row += 1;
                } else {
                    // Scroll up
                    self.scroll_up(1);
                }
            }
        }
    }

    pub fn move_cursor(&mut self, row: u16, col: u16) {
        self.cursor.row = row.min(self.rows.len() as u16 - 1);
        self.cursor.col = col.min(self.cols as u16 - 1);
    }

    pub fn move_cursor_relative(&mut self, delta_row: i16, delta_col: i16) {
        let new_row = (self.cursor.row as i16 + delta_row)
            .max(0)
            .min(self.rows.len() as i16 - 1) as u16;
        let new_col = (self.cursor.col as i16 + delta_col)
            .max(0)
            .min(self.cols as i16 - 1) as u16;
        
        self.cursor.row = new_row;
        self.cursor.col = new_col;
    }

    pub fn clear_screen(&mut self) {
        for row in &mut self.rows {
            for cell in row {
                *cell = TerminalChar::default();
            }
        }
        self.cursor = CursorPosition { row: 0, col: 0 };
    }

    pub fn clear_line(&mut self) {
        if let Some(row) = self.rows.get_mut(self.cursor.row as usize) {
            for cell in row {
                *cell = TerminalChar::default();
            }
        }
    }

    pub fn scroll_up(&mut self, lines: usize) {
        if lines >= self.rows.len() {
            self.clear_screen();
            return;
        }

        // Remove lines from the top
        for _ in 0..lines {
            self.rows.remove(0);
            // Add empty line at the bottom
            self.rows.push(vec![TerminalChar::default(); self.cols]);
        }
    }
}

#[derive(Debug)]
pub struct Terminal {
    pub id: String,
    pub grid: TerminalGrid,
    pub parser: AnsiParser,
    pub size: TerminalSize,
}

impl Terminal {
    pub fn new(id: String, size: TerminalSize) -> Self {
        let grid = TerminalGrid::new(size.cols as usize, size.rows as usize);
        let parser = AnsiParser::new();

        Terminal {
            id,
            grid,
            parser,
            size,
        }
    }

    pub fn process_output(&mut self, data: &str) {
        let commands = self.parser.parse(data);
        
        for command in commands {
            self.execute_command(command);
        }
    }

    fn execute_command(&mut self, command: AnsiCommand) {
        match command {
            AnsiCommand::PrintText(text) => {
                for ch in text.chars() {
                    self.grid.write_char(ch, self.parser.current_attributes());
                }
            }
            AnsiCommand::CursorUp(n) => {
                self.grid.move_cursor_relative(-(n as i16), 0);
            }
            AnsiCommand::CursorDown(n) => {
                self.grid.move_cursor_relative(n as i16, 0);
            }
            AnsiCommand::CursorLeft(n) => {
                self.grid.move_cursor_relative(0, -(n as i16));
            }
            AnsiCommand::CursorRight(n) => {
                self.grid.move_cursor_relative(0, n as i16);
            }
            AnsiCommand::CursorPosition(row, col) => {
                self.grid.move_cursor(row.saturating_sub(1), col.saturating_sub(1));
            }
            AnsiCommand::CursorHome => {
                self.grid.move_cursor(0, 0);
            }
            AnsiCommand::ClearScreen => {
                self.grid.clear_screen();
            }
            AnsiCommand::ClearLine => {
                self.grid.clear_line();
            }
            AnsiCommand::ClearToEndOfLine => {
                // TODO: Implement partial line clearing
                self.grid.clear_line();
            }
            AnsiCommand::ClearToBeginningOfLine => {
                // TODO: Implement partial line clearing
                self.grid.clear_line();
            }
            AnsiCommand::ScrollUp(n) => {
                self.grid.scroll_up(n as usize);
            }
            AnsiCommand::ScrollDown(_n) => {
                // TODO: Implement scroll down
            }
            AnsiCommand::SetGraphicsMode(params) => {
                self.parser.apply_graphics_mode(&params);
            }
            AnsiCommand::Bell => {
                // TODO: Handle bell (audio/visual notification)
                log::info!("Terminal bell");
            }
            AnsiCommand::Unknown(seq) => {
                log::warn!("Unknown escape sequence: {}", seq);
            },
            _ => {
                // Handle other ANSI commands
            }
        }
    }

    pub fn resize(&mut self, new_size: TerminalSize) {
        self.size = new_size.clone();
        self.grid.resize(new_size.cols as usize, new_size.rows as usize);
    }
}

pub struct TerminalManager {
    terminals: Arc<Mutex<HashMap<String, Terminal>>>,
    pty_manager: Arc<Mutex<PtyManager>>,
    shell_hooks: Arc<Mutex<ShellHooksManager>>,
    search_index: Arc<Mutex<SearchIndexManager>>,
}

impl TerminalManager {
    pub fn gather_context(&self, terminal_id: &str) -> Option<AiContext> {
        let (working_dir, prompt) = if let Some(p) = self.get_current_prompt(terminal_id) {
            (Some(p.working_dir.clone()), Some(p.prompt_text.clone()))
        } else { (None, None) };
        let recent_commands = self
            .get_command_history(terminal_id, Some(20))
            .unwrap_or_default()
            .into_iter()
            .map(|c| c.text)
            .collect::<Vec<_>>();
        let tail_output = self
            .search_index
            .lock()
            .unwrap()
            .tail(terminal_id, 200)
            .unwrap_or_default();
        Some(AiContext { working_dir, prompt, recent_commands, tail_output })
    }
    pub fn new() -> (Self, mpsc::UnboundedReceiver<TerminalOutput>) {
        let (pty_manager, output_receiver) = PtyManager::new();
        
        let manager = TerminalManager {
            terminals: Arc::new(Mutex::new(HashMap::new())),
            pty_manager: Arc::new(Mutex::new(pty_manager)),
            shell_hooks: Arc::new(Mutex::new(ShellHooksManager::new())),
            search_index: Arc::new(Mutex::new(SearchIndexManager::new())),
        };

        (manager, output_receiver)
    }

    pub fn create_terminal(
        &self,
        size: TerminalSize,
        shell: Option<String>,
        working_dir: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let shell_path = shell.clone().unwrap_or_else(|| {
            if cfg!(windows) {
                std::env::var("SHELL").unwrap_or_else(|_| "powershell.exe".to_string())
            } else {
                std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
            }
        });

        let work_dir = working_dir.clone().unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });

        let session_id = self.pty_manager
            .lock()
            .unwrap()
            .create_session(size.clone(), shell, working_dir)?;

        // Initialize shell hooks for this session
        self.shell_hooks
            .lock()
            .unwrap()
            .create_session_hooks(session_id.clone(), &shell_path, work_dir);
        // Initialize search index
        self.search_index.lock().unwrap().create_session(session_id.clone());

        let terminal = Terminal::new(session_id.clone(), size);
        self.terminals
            .lock()
            .unwrap()
            .insert(session_id.clone(), terminal);

        Ok(session_id)
    }

    pub fn write_to_terminal(
        &self,
        terminal_id: &str,
        data: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.pty_manager
            .lock()
            .unwrap()
            .write_to_session(terminal_id, data)
    }

    pub fn resize_terminal(
        &self,
        terminal_id: &str,
        size: TerminalSize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(terminal) = self.terminals.lock().unwrap().get_mut(terminal_id) {
            terminal.resize(size.clone());
        }

        self.pty_manager
            .lock()
            .unwrap()
            .resize_session(terminal_id, size)
    }

    pub fn close_terminal(&self, terminal_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.terminals.lock().unwrap().remove(terminal_id);
        self.shell_hooks.lock().unwrap().remove_session(terminal_id);
        self.search_index.lock().unwrap().remove_session(terminal_id);
        self.pty_manager
            .lock()
            .unwrap()
            .close_session(terminal_id)
    }

    pub fn process_output(&self, output: TerminalOutput) {
        // Process output with shell hooks for command tracking
        self.shell_hooks
            .lock()
            .unwrap()
            .process_output(&output.session_id, &output.data);

        // Append to search index
        self.search_index
            .lock()
            .unwrap()
            .append_output(&output.session_id, &output.data);

        // Process output for terminal display
        if let Some(terminal) = self.terminals
            .lock()
            .unwrap()
            .get_mut(&output.session_id)
        {
            terminal.process_output(&output.data);
        }
    }

    pub fn get_terminal_state(&self, terminal_id: &str) -> Option<TerminalGrid> {
        self.terminals
            .lock()
            .unwrap()
            .get(terminal_id)
            .map(|terminal| terminal.grid.clone())
    }

    // Shell hooks integration methods
    pub fn get_command_history(&self, terminal_id: &str, limit: Option<usize>) -> Option<Vec<crate::shell_hooks::Command>> {
        self.shell_hooks
            .lock()
            .unwrap()
            .get_command_history(terminal_id, limit)
    }

    pub fn get_command_suggestions(&self, terminal_id: &str, partial_command: &str) -> Option<Vec<crate::shell_hooks::CommandSuggestion>> {
        self.shell_hooks
            .lock()
            .unwrap()
            .get_command_suggestions(terminal_id, partial_command)
    }

    pub fn handle_tab_completion(&self, terminal_id: &str, current_line: &str, cursor_pos: usize) -> Option<Vec<String>> {
        self.shell_hooks
            .lock()
            .unwrap()
            .handle_tab_completion(terminal_id, current_line, cursor_pos)
    }

    pub fn is_at_prompt(&self, terminal_id: &str) -> bool {
        self.shell_hooks
            .lock()
            .unwrap()
            .is_at_prompt(terminal_id)
    }

    pub fn get_current_prompt(&self, terminal_id: &str) -> Option<crate::shell_hooks::PromptInfo> {
        self.shell_hooks
            .lock()
            .unwrap()
            .get_current_prompt(terminal_id)
            .cloned()
    }

    pub fn search_history(&self, terminal_id: &str, query: &str) -> Option<Vec<crate::shell_hooks::Command>> {
        self.shell_hooks
            .lock()
            .unwrap()
            .search_history(terminal_id, query)
    }

    pub fn search_scrollback(&self, terminal_id: &str, query: &str, case_sensitive: bool, use_regex: bool, limit: usize) -> Option<Vec<ScrollMatch>> {
        self.search_index
            .lock()
            .unwrap()
            .search(terminal_id, query, case_sensitive, use_regex, limit)
    }

    pub fn get_scrollback_context(&self, terminal_id: &str, line_index: usize, before: usize, after: usize) -> Option<Vec<ContextLine>> {
        self.search_index
            .lock()
            .unwrap()
            .context(terminal_id, line_index, before, after)
    }
}
