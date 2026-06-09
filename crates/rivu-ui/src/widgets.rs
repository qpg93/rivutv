use ratatui::widgets::ListState;

pub struct StatefulList<T> {
    pub items: Vec<T>,
    pub state: ListState,
}

impl<T> StatefulList<T> {
    pub fn new(items: Vec<T>) -> Self {
        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(0));
        }
        Self { items, state }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() { return; }
        let i = self.state.selected().map(|i| {
            if i >= self.items.len() - 1 { 0 } else { i + 1 }
        }).unwrap_or(0);
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() { return; }
        let i = self.state.selected().map(|i| {
            if i == 0 { self.items.len() - 1 } else { i - 1 }
        }).unwrap_or(0);
        self.state.select(Some(i));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stateful_list_new_with_items() {
        let list = StatefulList::new(vec![1, 2, 3]);
        assert_eq!(list.items.len(), 3);
        assert_eq!(list.state.selected(), Some(0));
    }

    #[test]
    fn test_stateful_list_new_empty() {
        let list: StatefulList<i32> = StatefulList::new(vec![]);
        assert!(list.items.is_empty());
        assert_eq!(list.state.selected(), None);
    }

    #[test]
    fn test_stateful_list_next_wraps_around() {
        let mut list = StatefulList::new(vec![1, 2]);
        assert_eq!(list.state.selected(), Some(0));
        list.next();
        assert_eq!(list.state.selected(), Some(1));
        list.next();
        assert_eq!(list.state.selected(), Some(0));
    }

    #[test]
    fn test_stateful_list_previous_wraps_around() {
        let mut list = StatefulList::new(vec![1, 2]);
        list.previous();
        assert_eq!(list.state.selected(), Some(1));
        list.previous();
        assert_eq!(list.state.selected(), Some(0));
    }

    #[test]
    fn test_stateful_list_next_on_empty_does_not_panic() {
        let mut list: StatefulList<i32> = StatefulList::new(vec![]);
        list.next();
        assert_eq!(list.state.selected(), None);
    }

    #[test]
    fn test_stateful_list_previous_on_empty_does_not_panic() {
        let mut list: StatefulList<i32> = StatefulList::new(vec![]);
        list.previous();
        assert_eq!(list.state.selected(), None);
    }

    #[test]
    fn test_stateful_list_single_item_stays_selected() {
        let mut list = StatefulList::new(vec![42]);
        assert_eq!(list.state.selected(), Some(0));
        list.next();
        assert_eq!(list.state.selected(), Some(0));
        list.previous();
        assert_eq!(list.state.selected(), Some(0));
    }
}
