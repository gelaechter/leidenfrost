use iced::{
    Element,
    advanced::{Widget, renderer, widget::tree::Tag},
};

/// Creates a new [`Tagged`] widget
pub fn tagged<'a, Message, Theme, Renderer>(
    inner: impl Into<Element<'a, Message, Theme, Renderer>>,
    tag: Tag,
) -> Tagged<'a, Message, Theme, Renderer> {
    Tagged {
        inner: inner.into(),
        tag,
    }
}

/// A special widget that causes the inner widget to be evaluated
/// with a specific tag.
/// 
/// This essentially makes this usable as a diffing-boundary that explicitly
/// tells iced that two widgets are different and should not reconciled between.
pub struct Tagged<'a, Message, Theme, Renderer> {
    inner: Element<'a, Message, Theme, Renderer>,
    tag: Tag,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Tagged<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> iced::Size<iced::Length> {
        self.inner.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.inner.as_widget_mut().layout(tree, renderer, limits)
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.inner
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }


    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        self.tag
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        self.inner.as_widget().state()
    }

    fn diff(&mut self, tree: &mut iced::advanced::widget::Tree) {
        self.inner.as_widget_mut().diff(tree);
    }

    fn operate(
        &mut self,
        _tree: &mut iced::advanced::widget::Tree,
        _layout: iced::advanced::Layout<'_>,
        _renderer: &Renderer,
        _operation: &mut dyn iced::advanced::widget::Operation,
    ) {
    }

    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &Renderer,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) {
        self.inner
            .as_widget_mut()
            .update(tree, event, layout, cursor, renderer, shell, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.inner
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'b>,
        renderer: &Renderer,
        viewport: &iced::Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        self.inner
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
    
    fn is_void(&self) -> bool {
        false
    }
}

impl<'a, Message: 'a, Theme: 'a, Renderer> From<Tagged<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + 'a,
{
    fn from(value: Tagged<'a, Message, Theme, Renderer>) -> Self {
        Self::new(value)
    }
}
