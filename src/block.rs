use std::cell::Cell;
use std::cmp::Ordering;
use std::io::{self, stdout, Error, Result as IoResult, Write};

use crate::interactive::Line as InteractiveLine;

/// Block is a struct that represents a block of interactive elements or indexed lines in the terminal. It consists of a vector of [`InteractiveElement`] instances.
pub struct Block {
    pub interactive_lines: Vec<InteractiveLine>,
    pub cursor_position: Position,
}

#[allow(unused)]
pub struct Position {
    row: Cell<u8>,
    col: Cell<u8>,
}

impl Block {
    /// Creates a new `Block` instance. The `interactive_lines` argument is a vector of `InteractiveElement` instances.
    /// # Example
    /// ```rust
    /// use interm::{interactive::Line as InteractiveLine, Block};
    ///
    /// let mut elements: Vec<InteractiveLine> = Vec::with_capacity(10);
    /// for idx in 0..10 {
    ///     elements.push(InteractiveLine::new(format!("Download {idx}").as_str()));
    /// }
    /// let block = Block::new(elements).unwrap();
    /// ```
    /// # Panics
    /// If the `interactive_lines` vector is larger than 255 elements, a panic will occur.
    ///
    /// # Notes
    /// The `interactive_lines` vector is consumed by the `Block` instance.
    ///
    /// # See also
    /// [`Block::update_element()`](struct.Block.html#method.update_element)
    /// [`Block::goto_idx()`](struct.Block.html#method.goto_idx)
    /// [`Block::goto_element()`](struct.Block.html#method.goto_element)
    /// [`Block::clear_line()`](struct.Block.html#method.clear_line)
    /// [`Block::clear_lines()`](struct.Block.html#method.clear_lines)
    /// [`Block::hide_cursor()`](struct.Block.html#method.hide_cursor)
    /// [`Block::show_cursor()`](struct.Block.html#method.show_cursor)
    pub fn new(interactive_lines: Vec<InteractiveLine>) -> Result<Self, Error> {
        assert!(
            interactive_lines.len() <= 255,
            "interactive_lines vector is larger than 255 elements"
        );
        assert!(
            !interactive_lines.is_empty(),
            "interactive_lines vector is empty"
        );

        let mut lines_vec: Vec<InteractiveLine> = Vec::with_capacity(interactive_lines.len());
        let cursor_position = Position {
            row: Cell::new(interactive_lines.len() as u8),
            col: Cell::new(0),
        };

        for (idx, line) in interactive_lines.into_iter().enumerate() {
            let mut line = line;
            line.update_relative_row(idx as u8);
            lines_vec.push(line);
        }

        let cur = Self {
            cursor_position,
            interactive_lines: lines_vec,
        };

        cur.prepare_all()?;

        Ok(cur)
    }

    /// Updates the content of `elem` which is an `InteractiveElement` instance. Optionally, you can clear the line before updating.
    /// # Example
    /// ```rust
    /// use interm::{interactive::Line as InteractiveLine, Block};
    ///
    /// let mut elements: Vec<InteractiveLine> = ...;
    /// let mut block = Block::new(elements).unwrap();
    ///
    /// let elem = &block.interactive_lines[0];
    /// block.update_element(elem, "Download 0: Complete", true);
    ///
    /// ```
    ///
    /// # Errors
    /// If the `elem` argument is not found in the `interactive_lines` vector, an error will be returned.
    ///
    /// # See also
    /// [`Block::new()`](struct.Block.html#method.new)
    /// [`Block::goto_idx()`](struct.Block.html#method.goto_idx)
    /// [`Block::goto_element()`](struct.Block.html#method.goto_element)
    /// [`Block::clear_line()`](struct.Block.html#method.clear_line)
    /// [`Block::clear_lines()`](struct.Block.html#method.clear_lines)
    /// [`Block::hide_cursor()`](struct.Block.html#method.hide_cursor)
    /// [`Block::show_cursor()`](struct.Block.html#method.show_cursor)
    ///
    pub fn update_element(
        &mut self,
        elem: &InteractiveLine,
        content: &str,
        clear: bool,
    ) -> Result<(), Error> {
        let relative_row = elem.relative_row.get();
        if let Some(elem) = self.interactive_lines.get_mut(relative_row as usize) {
            elem.content = content.to_string();
        } else {
            return Err(Error::new(
                io::ErrorKind::Other,
                format!("element {elem:?} not found"),
            ));
        }

        self.goto_element(elem)?;
        if clear {
            self.clear_line()?;
        }

        self.write_inline(content)?;
        Ok(())
    }

    /// Pass the index to an `InteractiveElement` and the cursor will go to it.
    /// # Example
    /// ```rust
    /// use interm::{interactive::Line as InteractiveLine, Block};
    ///
    /// let mut elements: Vec<InteractiveLine> = ...;
    /// let mut block = Block::new(elements).unwrap();
    ///
    /// block.goto_idx(0)?;
    /// ```
    /// # Errors
    /// If the index is not found in the `interactive_lines` vector, an error will be returned.
    /// # See also
    /// [`Block::new()`](struct.Block.html#method.new)
    /// [`Block::update_element()`](struct.Block.html#method.update_element)
    /// [`Block::goto_element()`](struct.Block.html#method.goto_element)
    /// [`Block::clear_line()`](struct.Block.html#method.clear_line)
    /// [`Block::clear_lines()`](struct.Block.html#method.clear_lines)
    /// [`Block::hide_cursor()`](struct.Block.html#method.hide_cursor)
    /// [`Block::show_cursor()`](struct.Block.html#method.show_cursor)
    ///
    /// # Notes
    /// The index is relative to the `interactive_lines` vector, not the terminal.
    /// For example, if you have 10 elements in the `interactive_lines` vector, the index will be from 0 to 9.
    /// If you want to go to the 5th element in the terminal, you need to pass 4 as the index.
    pub fn goto_idx(&self, idx: usize) -> Result<(), Error> {
        if let Some(el) = self.interactive_lines.get(idx) {
            self.go_to(el)?;
            self.cursor_position.row.set(el.relative_row.get());
        } else {
            return Err(Error::new(
                io::ErrorKind::Other,
                format!("index {idx} not found"),
            ));
        }
        Ok(())
    }

    /// Pass and `InteractiveElement` instance and the cursor will go to it.
    /// # Example
    /// ```rust
    /// use interm::{interactive::Line as InteractiveLine, Block};
    ///     
    /// let mut elements: Vec<InteractiveLine> = ...;
    /// let mut block = Block::new(elements).unwrap();
    ///
    /// let elem = &block.interactive_lines[0];
    /// block.goto_element(elem)?;
    /// ```
    /// # Errors
    /// If the `elem` argument is not found in the `interactive_lines` vector, an error will be returned.
    /// # See also
    /// [`Block::new()`](struct.Block.html#method.new)
    /// [`Block::update_element()`](struct.Block.html#method.update_element)
    /// [`Block::goto_idx()`](struct.Block.html#method.goto_idx)
    /// [`Block::clear_line()`](struct.Block.html#method.clear_line)
    /// [`Block::clear_lines()`](struct.Block.html#method.clear_lines)
    /// [`Block::hide_cursor()`](struct.Block.html#method.hide_cursor)
    /// [`Block::show_cursor()`](struct.Block.html#method.show_cursor)
    ///
    /// # Notes
    /// The index is relative to the `interactive_lines` vector, not the terminal.
    /// For example, if you have 10 elements in the `interactive_lines` vector, the index will be from 0 to 9.
    /// If you want to go to the 5th element in the terminal, you need to pass 4 as the index.
    pub fn goto_element(&self, el: &InteractiveLine) -> Result<(), Error> {
        let relative_row = el.relative_row.get();
        if let Some(el) = self.interactive_lines.get(relative_row as usize) {
            self.go_to(el)?;
            self.cursor_position.row.set(relative_row);
        } else {
            return Err(Error::new(
                io::ErrorKind::Other,
                format!("index {idx} not found", idx = relative_row),
            ));
        }
        self.write_inline("\r")?;
        Ok(())
    }

    /// Clears the row in which the cursor is currently in.
    /// # Example
    /// ```rust
    /// use interm::{interactive::Line as InteractiveLine, Block};
    ///
    /// let mut elements: Vec<InteractiveLine> = ...;
    /// let mut block = Block::new(elements).unwrap();
    ///
    /// block.clear_line()?;
    /// ```
    /// # See also
    /// [`Block::new()`](struct.Block.html#method.new)
    /// [`Block::update_element()`](struct.Block.html#method.update_element)
    /// [`Block::goto_idx()`](struct.Block.html#method.goto_idx)
    /// [`Block::goto_element()`](struct.Block.html#method.goto_element)
    /// [`Block::clear_lines()`](struct.Block.html#method.clear_lines)
    /// [`Block::hide_cursor()`](struct.Block.html#method.hide_cursor)
    /// [`Block::show_cursor()`](struct.Block.html#method.show_cursor)
    pub fn clear_line(&self) -> IoResult<()> {
        self.write_inline("\x1b[2K\r")?;
        Ok(())
    }

    /// Clears all interactive element lines.
    /// # Example
    /// ```rust
    /// use interm::{interactive::Line as InteractiveLine, Block};
    ///
    /// let mut elements: Vec<InteractiveLine> = ...;
    /// let mut block = Block::new(elements).unwrap();
    ///
    /// block.clear_lines()?;
    /// ```
    /// # See also
    /// [`Block::new()`](struct.Block.html#method.new)
    /// [`Block::update_element()`](struct.Block.html#method.update_element)
    /// [`Block::goto_idx()`](struct.Block.html#method.goto_idx)
    /// [`Block::goto_element()`](struct.Block.html#method.goto_element)
    /// [`Block::clear_line()`](struct.Block.html#method.clear_line)
    /// [`Block::hide_cursor()`](struct.Block.html#method.hide_cursor)
    /// [`Block::show_cursor()`](struct.Block.html#method.show_cursor)
    pub fn clear_lines(&mut self) -> IoResult<()> {
        let last_line = self.interactive_lines.len() - 1;
        self.goto_idx(last_line)?;
        for _ in 0..last_line {
            self.clear_line()?;
            self.move_up(1)?;
        }
        Ok(())
    }

    /// Hides cursor.
    /// # Example
    /// ```rust
    /// use interm::{interactive::Line as InteractiveLine, Block};
    ///
    /// let mut elements: Vec<InteractiveLine> = ...;
    /// let mut block = Block::new(elements).unwrap();
    ///     
    /// block.hide_cursor()?;
    /// ```
    /// # See also
    /// [`Block::new()`](struct.Block.html#method.new)
    /// [`Block::update_element()`](struct.Block.html#method.update_element)
    /// [`Block::goto_idx()`](struct.Block.html#method.goto_idx)
    /// [`Block::goto_element()`](struct.Block.html#method.goto_element)
    /// [`Block::clear_line()`](struct.Block.html#method.clear_line)
    /// [`Block::clear_lines()`](struct.Block.html#method.clear_lines)
    /// [`Block::show_cursor()`](struct.Block.html#method.show_cursor)
    pub fn hide_cursor(&self) -> IoResult<()> {
        self.write_inline("\x1b[?25l")?;
        Ok(())
    }

    /// Shows cursor.
    /// # Example
    /// ```rust
    /// use interm::{interactive::Line as InteractiveLine, Block};
    ///
    /// let mut elements: Vec<InteractiveLine> = ...;
    /// let mut block = Block::new(elements).unwrap();
    ///
    /// block.show_cursor()?;
    /// ```
    /// # See also
    /// [`Block::new()`](struct.Block.html#method.new)
    /// [`Block::update_element()`](struct.Block.html#method.update_element)
    /// [`Block::goto_idx()`](struct.Block.html#method.goto_idx)
    /// [`Block::goto_element()`](struct.Block.html#method.goto_element)
    /// [`Block::clear_line()`](struct.Block.html#method.clear_line)
    /// [`Block::clear_lines()`](struct.Block.html#method.clear_lines)
    /// [`Block::hide_cursor()`](struct.Block.html#method.hide_cursor)
    ///
    /// # Notes
    /// This method is automatically called when the `Block` instance is dropped.
    /// If you want to show the cursor before the `Block` instance is dropped, you can call this method.
    pub fn show_cursor(&self) -> IoResult<()> {
        self.write_inline("\x1b[?25h")?;
        Ok(())
    }

    fn write_inline(&self, str: &str) -> IoResult<()> {
        {
            let mut out = stdout().lock();
            let prepared_str = format!("\r{str}\r", str = str);
            out.write_all(prepared_str.as_bytes())?;
            out.flush()?;
        }
        Ok(())
    }

    fn go_to(&self, el: &InteractiveLine) -> IoResult<()> {
        let relative_row = el.relative_row.get();
        match self.cursor_position.row.get().cmp(&relative_row) {
            Ordering::Greater => {
                let move_by = self.cursor_position.row.get().abs_diff(relative_row);
                self.move_up(move_by)?;
            }
            Ordering::Less => {
                let move_by = relative_row.abs_diff(self.cursor_position.row.get());
                self.move_down(move_by)?;
            }
            _ => {}
        };
        Ok(())
    }

    fn move_up(&self, n: u8) -> IoResult<()> {
        let up_seq = format!("\x1b[{n}F");
        self.write_inline(&up_seq)?;
        Ok(())
    }

    fn move_down(&self, n: u8) -> IoResult<()> {
        let down_seq = format!("\x1b[{n}E");
        self.write_inline(&down_seq)?;
        Ok(())
    }

    fn prepare_all(&self) -> IoResult<()> {
        let prepared_space = "\n".repeat(self.interactive_lines.len());
        self.write_inline(&prepared_space)?;
        Ok(())
    }
}

// Ensures that the cursor will be shown when the `Block` instance is dropped even if the user forgot to call `Block::show_cursor()` or if the program panics.
impl Drop for Block {
    fn drop(&mut self) {
        self.show_cursor().unwrap();
    }
}
