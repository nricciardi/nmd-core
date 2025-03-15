pub const SPACE_TAB_EQUIVALENCE: &str = r"   ";


pub const CHAPTER_TAGS_PATTERN: &str = r"(?:\r?\n@(.*))*";
pub const CHAPTER_STYLE_PATTERN: &str = r"(\r?\n\{(?s:(.*))\})?";
pub const IDENTIFIER_PATTERN: &str = r"#([\w-]+)";
pub const NEW_LINE_PATTERN: &str = r"(?:\n|\r\n)";
pub const MULTI_LINES_CONTENT_PATTERN: &str = r"([\s\S]*?)";
pub const MULTI_LINES_CONTENT_EXCLUDING_HEADINGS_PATTERN: &str = r"(?m)^([^#\n][\s\S]*?)";