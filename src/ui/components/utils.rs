use iced::widget::{
    self,
    text::{Rich, Span},
};

pub trait IntoLinks {
    type Item;
    fn into_links<'a, F, Link, Message>(
        self,
        f: F,
        on_link_click: impl Fn(Link) -> Message + 'a,
    ) -> Rich<'a, Link, Message>
    where
        // A function which produces a text and a link from the iterable
        F: Fn(Self::Item) -> (String, Link),
        Link: Clone + 'static;
}

impl<I> IntoLinks for I
where
    I: IntoIterator,
{
    type Item = I::Item;

    /// Constructs [`text::rich::Rich`] text links from any iterable as long as the
    /// transformation function `f` can turn it into a (String, Link) pair.
    fn into_links<'a, F, Link, Message>(
        self,
        f: F,
        on_link_click: impl Fn(Link) -> Message + 'a,
    ) -> Rich<'a, Link, Message>
    where
        Link: Clone + 'static,
        F: Fn(Self::Item) -> (String, Link),
    {
        let mut iter = self.into_iter().peekable();

        let text: Vec<Span<'_, Link>> = std::iter::from_fn(move || {
            let item = iter.next()?;
            let (name, link) = f(item);
            let span = widget::span(name).link(link);

            if iter.peek().is_none() {
                Some(vec![span])
            } else {
                Some(vec![span, widget::span(", ")])
            }
        })
        .flatten()
        .collect();

        widget::rich_text(text).on_link_click(on_link_click)
    }
}
