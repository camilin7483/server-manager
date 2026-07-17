use super::ansi::{AnsiColor, Cell, NamedColor};

const MAX_SCROLLBACK: usize = 10_000;
const DEFAULT_COLS: usize = 80;
const DEFAULT_ROWS: usize = 24;

pub struct TerminalGrid {
    pub cols: u16,
    pub rows: u16,
    scrollback: Vec<Vec<Cell>>,   // lines in scrollback
    screen: Vec<Vec<Cell>>,       // visible lines
    current_attrs: Cell,
    cursor_row: u16,
    cursor_col: u16,
    cursor_visible: bool,
    saved_cursor: (u16, u16),
    // Parsing state
    in_escape: bool,
    escape_buffer: Vec<u8>,
    csi_params: Vec<u16>,
    csi_private: bool,
    current_char: char,
}

impl TerminalGrid {
    pub fn new(cols: u16, rows: u16) -> Self {
        let screen = vec![vec![Cell::default(); cols as usize]; rows as usize];
        Self {
            cols,
            rows,
            scrollback: Vec::new(),
            screen,
            current_attrs: Cell::default(),
            cursor_row: 0,
            cursor_col: 0,
            cursor_visible: true,
            saved_cursor: (0, 0),
            in_escape: false,
            escape_buffer: Vec::new(),
            csi_params: Vec::new(),
            csi_private: false,
            current_char: '\0',
        }
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
        self.screen = vec![vec![Cell::default(); cols as usize]; rows as usize];
        if self.cursor_row >= rows { self.cursor_row = rows - 1; }
        if self.cursor_col >= cols { self.cursor_col = cols - 1; }
    }

    pub fn write(&mut self, data: &[u8]) {
        let text = String::from_utf8_lossy(data);
        for ch in text.chars() {
            self.process_char(ch as u8 as char);
        }
    }

    pub fn writeln(&mut self, data: &str) {
        self.write(data.as_bytes());
        self.newline();
    }

    fn process_char(&mut self, ch: char) {
        if self.in_escape {
            self.escape_buffer.push(ch as u8);
            if self.try_parse_escape() {
                self.in_escape = false;
                self.escape_buffer.clear();
            }
            return;
        }

        match ch as u8 {
            0x1b => { // ESC
                self.in_escape = true;
                self.escape_buffer.clear();
                self.csi_params.clear();
                self.csi_private = false;
            }
            b'\x07' => {} // BEL - bell, ignore
            b'\x08' => { self.cursor_back(1); } // BS
            b'\t' => { // TAB
                let spaces = 8 - (self.cursor_col % 8);
                for _ in 0..spaces { self.put_char(' '); }
            }
            b'\n' => { self.newline(); }
            b'\r' => { self.cursor_col = 0; }
            b'\x0c' => { self.clear_screen(); } // FF - form feed
            _ => {
                if ch.is_ascii_graphic() || ch == ' ' {
                    self.put_char(ch);
                }
            }
        }
    }

    fn put_char(&mut self, ch: char) {
        if self.cursor_col >= self.cols {
            self.cursor_col = 0;
            self.cursor_row += 1;
        }
        if self.cursor_row >= self.rows {
            self.scroll_up(1);
            self.cursor_row = self.rows - 1;
        }
        let row = self.cursor_row as usize;
        let col = self.cursor_col as usize;
        if row < self.screen.len() && col < self.screen[row].len() {
            let mut cell = self.current_attrs;
            cell.ch = ch;
            self.screen[row][col] = cell;
        }
        self.cursor_col += 1;
    }

    fn newline(&mut self) {
        self.cursor_row += 1;
        self.cursor_col = 0;
        if self.cursor_row >= self.rows {
            self.scroll_up(1);
            self.cursor_row = self.rows - 1;
        }
    }

    fn scroll_up(&mut self, n: u16) {
        for _ in 0..n {
            let top_line = self.screen.remove(0);
            self.scrollback.push(top_line);
            if self.scrollback.len() > MAX_SCROLLBACK {
                self.scrollback.remove(0);
            }
            self.screen.push(vec![Cell::default(); self.cols as usize]);
        }
    }

    fn clear_screen(&mut self) {
        for row in &mut self.screen {
            for cell in row {
                *cell = Cell::default();
            }
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    fn clear_line(&mut self, mode: u8) {
        let row = self.cursor_row as usize;
        if row >= self.screen.len() { return; }
        let col = self.cursor_col as usize;

        match mode {
            0 => { // clear to end
                for c in col..self.screen[row].len() {
                    self.screen[row][c] = Cell::default();
                }
            }
            1 => { // clear to beginning
                for c in 0..=col {
                    self.screen[row][c] = Cell::default();
                }
            }
            2 => { // clear whole line
                for c in 0..self.screen[row].len() {
                    self.screen[row][c] = Cell::default();
                }
            }
            _ => {}
        }
    }

    fn cursor_back(&mut self, n: u16) {
        self.cursor_col = self.cursor_col.saturating_sub(n);
    }

    fn cursor_forward(&mut self, n: u16) {
        self.cursor_col = (self.cursor_col + n).min(self.cols - 1);
    }

    fn cursor_up(&mut self, n: u16) {
        self.cursor_row = self.cursor_row.saturating_sub(n);
    }

    fn cursor_down(&mut self, n: u16) {
        self.cursor_row = (self.cursor_row + n).min(self.rows - 1);
    }

    fn try_parse_escape(&mut self) -> bool {
        let s = &self.escape_buffer;
        if s.len() < 2 { return false; }

        if s[1] == b'[' {
            // CSI sequence - check if complete (ends with 0x40-0x7E)
            let last = *s.last().unwrap();
            if last < 0x40 || last > 0x7E {
                return s.len() > 16; // safety escape
            }
            self.handle_csi(last as char);
            return true;
        }

        if s[1] == b'c' { // RIS - reset to initial state
            self.clear_screen();
            self.current_attrs = Cell::default();
            return true;
        }

        if s.len() >= 3 && s[1] == b'7' { // DECSC - save cursor
            self.saved_cursor = (self.cursor_row, self.cursor_col);
            return true;
        }
        if s.len() >= 3 && s[1] == b'8' { // DECRC - restore cursor
            (self.cursor_row, self.cursor_col) = self.saved_cursor;
            return true;
        }

        s.len() > 6 // safety timeout
    }

    fn handle_csi(&mut self, final_char: char) {
        let params = &self.csi_params;

        match final_char {
            'A' => self.cursor_up(params.first().copied().unwrap_or(1).max(1)),
            'B' => self.cursor_down(params.first().copied().unwrap_or(1).max(1)),
            'C' => self.cursor_forward(params.first().copied().unwrap_or(1).max(1)),
            'D' => self.cursor_back(params.first().copied().unwrap_or(1).max(1)),
            'E' => { // CNL - cursor next line
                self.cursor_row = (self.cursor_row + 1).min(self.rows - 1);
                self.cursor_col = 0;
            }
            'H' | 'f' => { // CUP - cursor position
                let row = params.first().copied().unwrap_or(1).saturating_sub(1);
                let col = params.get(1).copied().unwrap_or(1).saturating_sub(1);
                self.cursor_row = row.min(self.rows - 1);
                self.cursor_col = col.min(self.cols - 1);
            }
            'J' => { // ED - erase display
                let mode = params.first().copied().unwrap_or(0) as u8;
                match mode {
                    0 => { // erase below
                        let row = self.cursor_row as usize;
                        let col = self.cursor_col as usize;
                        if row < self.screen.len() {
                            for c in col..self.screen[row].len() {
                                self.screen[row][c] = Cell::default();
                            }
                        }
                        for r in (row + 1)..self.screen.len() {
                            for c in 0..self.screen[r].len() {
                                self.screen[r][c] = Cell::default();
                            }
                        }
                    }
                    1 => { // erase above
                        for r in 0..self.cursor_row as usize {
                            for c in 0..self.screen[r].len() {
                                self.screen[r][c] = Cell::default();
                            }
                        }
                        let row = self.cursor_row as usize;
                        let col = self.cursor_col as usize;
                        if row < self.screen.len() {
                            for c in 0..=col {
                                self.screen[row][c] = Cell::default();
                            }
                        }
                    }
                    2 | 3 => self.clear_screen(),
                    _ => {}
                }
            }
            'K' => { // EL - erase in line
                let mode = params.first().copied().unwrap_or(0) as u8;
                self.clear_line(mode);
            }
            'm' => { // SGR - set graphics rendition
                let params_clone = params.clone();
                self.handle_sgr(&params_clone);
            }
            's' => { // save cursor
                self.saved_cursor = (self.cursor_row, self.cursor_col);
            }
            'u' => { // restore cursor
                (self.cursor_row, self.cursor_col) = self.saved_cursor;
            }
            'h' | 'l' => {} // set/reset modes - ignore for now
            _ => {}
        }
    }

    fn handle_sgr(&mut self, params: &[u16]) {
        if params.is_empty() {
            self.current_attrs = Cell::default();
            return;
        }

        let mut i = 0;
        while i < params.len() {
            match params[i] {
                0 => self.current_attrs = Cell::default(),
                1 => self.current_attrs.bold = true,
                2 => self.current_attrs.bold = false, // dim
                3 => self.current_attrs.italic = true,
                4 => self.current_attrs.underline = true,
                7 => self.current_attrs.inverse = true,
                9 => self.current_attrs.strikethrough = true,
                22 => self.current_attrs.bold = false,
                23 => self.current_attrs.italic = false,
                24 => self.current_attrs.underline = false,
                27 => self.current_attrs.inverse = false,
                29 => self.current_attrs.strikethrough = false,
                30..=37 => {
                    let color = NamedColor::from_idx((params[i] - 30) as u8);
                    self.current_attrs.fg = AnsiColor::Named(color);
                }
                38 => {
                    if i + 3 < params.len() && params[i + 1] == 5 {
                        self.current_attrs.fg = AnsiColor::Indexed(params[i + 2] as u8);
                        i += 3;
                    } else if i + 4 < params.len() && params[i + 1] == 2 {
                        self.current_attrs.fg = AnsiColor::Rgb(
                            params[i + 2] as u8, params[i + 3] as u8, params[i + 4] as u8,
                        );
                        i += 4;
                    }
                }
                39 => self.current_attrs.fg = AnsiColor::Default,
                40..=47 => {
                    let color = NamedColor::from_idx((params[i] - 40) as u8);
                    self.current_attrs.bg = AnsiColor::Named(color);
                }
                48 => {
                    if i + 3 < params.len() && params[i + 1] == 5 {
                        self.current_attrs.bg = AnsiColor::Indexed(params[i + 2] as u8);
                        i += 3;
                    }
                }
                49 => self.current_attrs.bg = AnsiColor::Default,
                90..=97 => { // bright foreground
                    let color = NamedColor::from_idx((params[i] - 90 + 8) as u8);
                    self.current_attrs.fg = AnsiColor::Named(color);
                }
                100..=107 => { // bright background
                    let color = NamedColor::from_idx((params[i] - 100 + 8) as u8);
                    self.current_attrs.bg = AnsiColor::Named(color);
                }
                _ => {}
            }
            i += 1;
        }
    }

    // Rendering helpers
    pub fn all_lines(&self) -> Vec<&[Cell]> {
        self.scrollback.iter().map(|v| v.as_slice()).collect()
    }

    pub fn screen_lines(&self) -> &[Vec<Cell>] {
        &self.screen
    }

    pub fn scrollback_len(&self) -> usize {
        self.scrollback.len()
    }

    pub fn cursor_position(&self) -> (u16, u16) {
        (self.cursor_row, self.cursor_col)
    }
}
