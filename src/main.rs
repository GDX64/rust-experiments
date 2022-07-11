use carboxyl::{lift, Signal, Sink, Stream};

pub fn main() {}

struct Scale2DRange {
    x: (f64, f64),
    y: (f64, f64),
}

trait Scale2D {
    fn x(&self, val: f64) -> f64;
    fn y(&self, val: f64) -> f64;
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

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use carboxyl::{Signal, Sink};

    use crate::{make, Drawable, Scale2D, Scale2DRange};

    #[derive(Clone)]
    struct TestScale2D {}

    impl Scale2D for TestScale2D {
        fn x(&self, val: f64) -> f64 {
            val.round()
        }

        fn y(&self, val: f64) -> f64 {
            val.round()
        }
    }

    #[derive(Clone)]
    struct TestDrawable {
        values: Arc<Vec<f64>>,
    }

    impl Drawable for TestDrawable {
        fn min_max(&self) -> crate::Scale2DRange {
            let max = self.values.iter().map(|&x| x).reduce(|a, b| a.max(b));
            let min = self.values.iter().map(|&x| x).reduce(|a, b| a.min(b));
            let y = (max.unwrap_or_default(), min.unwrap_or_default());
            crate::Scale2DRange { x: (0.0, 0.0), y }
        }

        fn range(&self, r: (usize, usize)) -> Self {
            let v = self.values[r.0..r.1].to_vec();
            TestDrawable {
                values: Arc::new(v),
            }
        }

        fn scale(&self, scale: &dyn Scale2D) -> Self {
            let v = self.values.iter().map(|&value| scale.y(value)).collect();
            TestDrawable {
                values: Arc::new(v),
            }
        }
    }

    #[test]
    fn test_make() {
        let d_sink = Sink::new();
        let drawables = d_sink.stream().hold(TestDrawable {
            values: Arc::new(vec![1.2, 2.3, 3.1]),
        });
        let range_sink = Sink::new();
        let range = range_sink.stream().hold((0, 2));
        let make_scale = Signal::new(|_: Scale2DRange| TestScale2D {});
        let s = make(drawables, range, make_scale);
        assert_eq!(vec![1.0, 2.0], *s.sample().values);
        range_sink.send((1, 3));
        assert_eq!(vec![2.0, 3.0], *s.sample().values);
    }
}
