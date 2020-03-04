use crate::{
    canvas::{Drawable, Frame, Layer},
    triangle,
};

use iced_native::Size;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::Arc;

/// A simple cache that stores generated geometry to avoid recomputation.
///
/// A [`Cache`] will not redraw its geometry unless the dimensions of its layer
/// change or it is explicitly cleared.
///
/// [`Layer`]: ../trait.Layer.html
/// [`Cached`]: struct.Cached.html
#[derive(Debug)]
pub struct Cache<T: Drawable> {
    input: PhantomData<T>,
    state: RefCell<State>,
}

enum State {
    Empty,
    Filled {
        mesh: Arc<triangle::Mesh2D>,
        bounds: Size,
    },
}

impl<T> Cache<T>
where
    T: Drawable + std::fmt::Debug,
{
    /// Creates a new empty [`Cache`].
    ///
    /// [`Cache`]: struct.Cache.html
    pub fn new() -> Self {
        Cache {
            input: PhantomData,
            state: RefCell::new(State::Empty),
        }
    }

    /// Clears the cache, forcing a redraw the next time it is used.
    ///
    /// [`Cached`]: struct.Cached.html
    pub fn clear(&mut self) {
        *self.state.borrow_mut() = State::Empty;
    }

    /// Binds the [`Cache`] with some data, producing a [`Layer`] that can be
    /// added to a [`Canvas`].
    ///
    /// [`Cache`]: struct.Cache.html
    /// [`Layer`]: ../trait.Layer.html
    /// [`Canvas`]: ../../struct.Canvas.html
    pub fn with<'a>(&'a self, input: &'a T) -> impl Layer + 'a {
        Bind {
            cache: self,
            input: input,
        }
    }
}

#[derive(Debug)]
struct Bind<'a, T: Drawable> {
    cache: &'a Cache<T>,
    input: &'a T,
}

impl<'a, T> Layer for Bind<'a, T>
where
    T: Drawable + std::fmt::Debug,
{
    fn draw(&self, current_bounds: Size) -> Arc<triangle::Mesh2D> {
        use std::ops::Deref;

        if let State::Filled { mesh, bounds } =
            self.cache.state.borrow().deref()
        {
            if *bounds == current_bounds {
                return mesh.clone();
            }
        }

        let mut frame = Frame::new(current_bounds.width, current_bounds.height);
        self.input.draw(&mut frame);

        let mesh = Arc::new(frame.into_mesh());

        *self.cache.state.borrow_mut() = State::Filled {
            mesh: mesh.clone(),
            bounds: current_bounds,
        };

        mesh
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Empty => write!(f, "Empty"),
            State::Filled { mesh, bounds } => f
                .debug_struct("Filled")
                .field("vertices", &mesh.vertices.len())
                .field("indices", &mesh.indices.len())
                .field("bounds", bounds)
                .finish(),
        }
    }
}
