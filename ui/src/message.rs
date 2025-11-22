use crate::pages::Page;

#[derive(Debug)]
pub enum Message {
    Tick,
    NavigateTo(Page),
}
