use crate::pages::Page;
use crate::themes::AppTheme;

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    NavigateTo(Page),
    ChangeTheme(AppTheme),
}
