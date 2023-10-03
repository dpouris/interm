use std::cell::Cell;

#[derive(Default, Debug, Clone)]
/// A line that can be updated in the terminal. It is used in the [`Block`] struct.
pub struct InteractiveLine {
    pub content: String,
    pub relative_row: Cell<u8>,
}

impl InteractiveLine {
    /// Creates a new [`InteractiveLine`] with the given content.
    ///
    /// # Arguments
    ///
    /// * `content` - The content of the line.
    ///
    /// # Example
    ///
    /// ```
    /// use interm::interactive::Line;
    ///
    /// let line = Line::new("Hello, world!");
    ///
    /// assert_eq!(line.content, "Hello, world!");
    /// ```
    ///
    /// [`Block`]: struct.Block.html
    /// [`InteractiveLine`]: struct.InteractiveLine.html
    /// [`InteractiveLine::content`]: struct.InteractiveLine.html#structfield.content
    /// [`InteractiveLine::relative_row`]: struct.InteractiveLine.html#structfield.relative_row
    /// [`InteractiveLine::new`]: struct.InteractiveLine.html#method.new
    /// [`InteractiveLine::update_content`]: struct.InteractiveLine.html#method.update_content
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_owned(),
            ..Default::default()
        }
    }

    /// Updates the content of the line.
    /// # Arguments
    /// * `content` - The new content of the line.
    /// # Example
    /// ```
    /// use interm::interactive::Line;
    /// let mut line = Line::new("Hello, world!");
    /// line.update_content("Hello, world! 2");
    /// assert_eq!(line.content, "Hello, world! 2");
    /// ```
    /// [`Block`]: struct.Block.html
    /// [`InteractiveLine`]: struct.InteractiveLine.html
    /// [`InteractiveLine::content`]: struct.InteractiveLine.html#structfield.content
    /// [`InteractiveLine::relative_row`]: struct.InteractiveLine.html#structfield.relative_row
    /// [`InteractiveLine::new`]: struct.InteractiveLine.html#method.new
    /// [`InteractiveLine::update_content`]: struct.InteractiveLine.html#method.update_content
    ///
    /// # Notes
    /// This method is used in the [`Block::update_element`] method.
    pub fn update_content(&mut self, content: &str) {
        self.content = content.to_owned();
    }

    pub(crate) fn update_relative_row(&mut self, row: u8) {
        *self.relative_row.get_mut() = row;
    }
}
