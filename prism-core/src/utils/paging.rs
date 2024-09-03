#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub current_page: u64,
    pub total_pages: u64,
    pub total_items: u64,
}
