
//! # Cell Module
//!
//! This module defines the core data structures for representing individual cells
//! in a cloud matrix data structure. It includes cell values, formatting options,
//! and alignment configurations.
//!
//! ## Overview
//!
//! The cell module provides the fundamental building blocks for tabular data representation:
//! - [`CellValue`]: A flexible type for storing various data types in cells
//! - [`CellFormat`]: Comprehensive formatting options for cell appearance
//! - [`HorizontalAlignment`] and [`VerticalAlignment`]: Text alignment enums
//!
//! ## Examples
//!
//! ```rust
//! use cmx_core::model::data::cell::{CellValue, CellFormat, HorizontalAlignment, VerticalAlignment};
//!
//! // Create a cell value from different data types
//! let text_value: CellValue = serde_json::json!("Hello World");
//! let number_value: CellValue = serde_json::json!(42);
//! let boolean_value: CellValue = serde_json::json!(true);
//!
//! // Create a cell format with custom styling
//! let format = CellFormat {
//!     font_name: "Arial".to_string(),
//!     font_size: 12.0,
//!     font_color: "#000000".to_string(),
//!     background_color: "#FFFFFF".to_string(),
//!     bold: false,
//!     italic: true,
//!     underline: false,
//!     horizontal_alignment: HorizontalAlignment::Center,
//!     vertical_alignment: VerticalAlignment::Middle,
//! };
//! ```

/// Represents the value that can be stored in a cell.
///
/// This type alias uses `serde_json::Value` to provide maximum flexibility
/// in storing different data types within cells. It supports:
/// - Strings (`"text"`)
/// - Numbers (`42`, `3.14`)
/// - Booleans (`true`, `false`)
/// - Arrays (`[1, 2, 3]`)
/// - Objects (`{"key": "value"}`)
/// - Null values (`null`)
///
/// # Examples
///
/// ```rust
/// use cmx_core::model::data::cell::CellValue;
/// use serde_json::json;
///
/// let string_cell: CellValue = json!("Hello");
/// let number_cell: CellValue = json!(123);
/// let array_cell: CellValue = json!([1, 2, 3]);
/// ```
pub type CellValue = serde_json::Value;


/// Defines the visual formatting options for a cell.
///
/// `CellFormat` encapsulates all the styling properties that can be applied
/// to customize the appearance of cell content. This includes typography,
/// colors, text styling, and alignment settings.
///
/// # Examples
///
/// ```rust
/// use cmx_core::model::data::cell::{CellFormat, HorizontalAlignment, VerticalAlignment};
///
/// let header_format = CellFormat {
///     font_name: "Arial".to_string(),
///     font_size: 14.0,
///     font_color: "#000000".to_string(),
///     background_color: "#E6E6E6".to_string(),
///     bold: true,
///     italic: false,
///     underline: false,
///     horizontal_alignment: HorizontalAlignment::Center,
///     vertical_alignment: VerticalAlignment::Middle,
/// };
/// ```
#[allow(dead_code)]
pub struct CellFormat {
    /// The name of the font family to use for the cell text.
    ///
    /// Common examples include "Arial", "Times New Roman", "Helvetica", etc.
    /// The availability of fonts depends on the rendering environment.
    pub font_name: String,

    /// The size of the font in points.
    ///
    /// Typical values range from 8.0 to 72.0, with 12.0 being a common default.
    /// Values should be positive floats.
    pub font_size: f32,

    /// The color of the text in the cell.
    ///
    /// Accepts hexadecimal color codes (e.g., "#FF0000" for red) or
    /// standard color names. The format should be compatible with
    /// the target rendering system.
    pub font_color: String,

    /// The background color of the cell.
    ///
    /// Accepts hexadecimal color codes (e.g., "#FFFFFF" for white) or
    /// standard color names. Transparent backgrounds can be achieved
    /// with appropriate color values depending on the rendering context.
    pub background_color: String,

    /// Whether the text should be rendered in bold weight.
    ///
    /// When `true`, the text will appear thicker and more prominent.
    /// When `false`, normal font weight is used.
    pub bold: bool,

    /// Whether the text should be rendered in italic style.
    ///
    /// When `true`, the text will appear slanted. When `false`,
    /// normal text orientation is used.
    pub italic: bool,

    /// Whether the text should be underlined.
    ///
    /// When `true`, a line will appear beneath the text.
    /// When `false`, no underline is applied.
    pub underline: bool,

    /// The horizontal alignment of text within the cell.
    ///
    /// Determines how text is positioned horizontally (left, center, or right).
    pub horizontal_alignment: HorizontalAlignment,

    /// The vertical alignment of text within the cell.
    ///
    /// Determines how text is positioned vertically (top, middle, or bottom).
    pub vertical_alignment: VerticalAlignment,
}

/// Specifies the horizontal text alignment within a cell.
///
/// This enum defines the three standard horizontal alignment options
/// available for positioning text within a cell's boundaries.
#[allow(dead_code)]
pub enum HorizontalAlignment {
    /// Aligns text to the left edge of the cell.
    ///
    /// This is typically the default alignment for most text content.
    Left,

    /// Centers text horizontally within the cell.
    ///
    /// Useful for headers, titles, and numeric data that should
    /// appear balanced in the cell.
    Center,

    /// Aligns text to the right edge of the cell.
    ///
    /// Commonly used for numeric data and right-aligned content.
    Right,
}

/// Specifies the vertical text alignment within a cell.
///
/// This enum defines the three standard vertical alignment options
/// available for positioning text within a cell's boundaries.
#[allow(dead_code)]
pub enum VerticalAlignment {
    /// Aligns text to the top of the cell.
    ///
    /// Text will appear at the topmost position within the cell's height.
    Top,

    /// Centers text vertically within the cell.
    ///
    /// Text will be positioned in the middle of the cell's height.
    /// This is often the default vertical alignment.
    Middle,

    /// Aligns text to the bottom of the cell.
    ///
    /// Text will appear at the bottommost position within the cell's height.
    Bottom,
}
