use carboxyl::{lift, Signal, Sink, Stream};

pub fn main() {}

struct Scale2DRange {
    x: (f64, f64),
    y: (f64, f64),
}

trait Scale2D {
    fn x(&self, val: f64) -> usize;
    fn y(&self, val: f64) -> usize;
}

trait Drawable {
    fn min_max(&self) -> Scale2DRange;
    fn range(&self, r: (usize, usize)) -> Self;
    fn scale(&self, scale: &dyn Scale2D) -> Self;
}

fn make<S, F, D>(
    drawables: Signal<D>,
    range: Signal<(usize, usize)>,
    make_scale: Signal<F>,
) -> Signal<D>
where
    D: Drawable + Sync + Clone + Send + 'static,
    S: Scale2D + Sync + Clone + Send,
    F: (Fn(Scale2DRange) -> S) + Send + Sync + 'static + Clone,
{
    lift!(
        |s, r, scale_fn| {
            let ranged = s.range(r);
            let scale = scale_fn(ranged.min_max());
            ranged.scale(&scale)
        },
        &drawables,
        &range,
        &make_scale
    )
}
