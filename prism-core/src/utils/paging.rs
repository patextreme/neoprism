#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub current_page: u32,
    pub page_size: u32,
    pub total_items: u32,
}

impl<T> Paginated<T> {
    pub fn total_pages(&self) -> u32 {
        self.total_items.div_ceil(self.page_size)
    }
}
