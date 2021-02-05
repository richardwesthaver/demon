use cursive::align::HAlign;
use cursive::direction::Orientation;
use cursive::traits::*;
use cursive::views::{Dialog, DummyView, LinearLayout, ResizedView};
use cursive_table_view::{TableView, TableViewItem};

use std::cmp::Ordering;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum BasicColumn {
    Name,
    Count,
    Rate,
}


