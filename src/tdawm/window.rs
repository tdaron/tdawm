pub type WindowId = u64;

#[derive(Debug, Clone, Copy, Default)]
pub struct Window {
    pub id: WindowId,
    pub fixed_position: Option<Position>,
    pub fixed_size: Option<Size>,
    pub window_type: WindowType,
}
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub x: u32,
    pub y: u32,
}

impl From<u64> for Window {
    fn from(value: u64) -> Self {
        Self {
            id: value,
            ..Default::default()
        }
    }
}
impl std::cmp::Ord for Window {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}
impl std::cmp::PartialEq for Window {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}
impl std::cmp::PartialOrd for Window {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl std::cmp::Eq for Window {}
#[derive(Debug, Clone, Copy, Default)]
pub enum WindowType {
    #[default]
    Normal,
    Dock,
}
