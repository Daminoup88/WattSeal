use iced::{
    Alignment, Element, Event, Length, Rectangle, Size, Vector,
    advanced::{
        Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer,
        widget::{Operation, Tree},
    },
    widget::Row,
};

use crate::styles::style_constants::{SPACING_MEDIUM, SPACING_XLARGE};

/// Keeps the title left-aligned and the controls right-aligned, including after wrapping.
pub fn chart_header<'a, Message: 'a, Theme: 'a, Renderer: renderer::Renderer + 'a>(
    title: impl Into<Element<'a, Message, Theme, Renderer>>,
    controls: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer> {
    Element::new(ChartHeader {
        content: Row::new()
            .width(Length::Fill)
            .spacing(SPACING_XLARGE)
            .align_y(Alignment::Center)
            .push(title)
            .push(controls)
            .wrap()
            .vertical_spacing(SPACING_MEDIUM)
            .into(),
    })
}

struct ChartHeader<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
}

impl<Message, Theme, Renderer: renderer::Renderer> Widget<Message, Theme, Renderer>
    for ChartHeader<'_, Message, Theme, Renderer>
{
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let node = self.content.as_widget_mut().layout(tree, renderer, limits);
        let mut children = node.children().to_vec();
        // Wrapping decides which line contains the controls. Move that entire
        // subtree to the right edge so drawing, hit testing, and overlays agree.
        if let Some(controls) = children.get_mut(1) {
            controls.move_to_mut((node.size().width - controls.size().width, controls.bounds().y));
        }
        layout::Node::with_children(node.size(), children)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        self.content.as_widget_mut().operate(tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content
            .as_widget_mut()
            .update(tree, event, layout, cursor, renderer, clipboard, shell, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

#[cfg(test)]
mod tests {
    use iced::{alignment::Horizontal, widget::Space};

    use super::*;

    #[test]
    fn titles_and_controls_stay_separate_and_aligned_across_widths() {
        // Fixed extents represent measured title, time selector, metric selector,
        // and help button. Exercise the real widget layout without a GPU.
        for width in [180.0, 240.0, 320.0, 420.0, 800.0] {
            let controls: Element<'_, (), (), ()> = Row::new()
                .spacing(SPACING_MEDIUM)
                .push(Space::new().width(130).height(28))
                .push(Space::new().width(110).height(28))
                .push(Space::new().width(28).height(28))
                .wrap()
                .vertical_spacing(SPACING_MEDIUM)
                .align_x(Horizontal::Right)
                .into();
            let mut header = chart_header(Space::new().width(90).height(20), controls);
            let mut tree = Tree::new(&header);
            let node = header.as_widget_mut().layout(
                &mut tree,
                &(),
                &layout::Limits::new(Size::ZERO, Size::new(width, 1000.0)),
            );
            let title = node.children()[0].bounds();
            let controls = node.children()[1].bounds();
            assert_eq!(title.x, 0.0);
            assert_eq!(title.width, 90.0);
            assert_eq!(controls.x + controls.width, width);
            assert!(title.intersection(&controls).is_none(), "overlap at {width}");
            if width >= 398.0 {
                assert_eq!(node.size().height, 28.0, "wide headers should stay on one line");
            } else {
                assert!(controls.y >= title.y + title.height + SPACING_MEDIUM);
            }
            for control in node.children()[1].children() {
                let bounds = control.bounds();
                assert!(bounds.x >= 0.0 && bounds.x + bounds.width <= controls.width);
            }
        }
    }
}
