//! A module dealing with accumulation hints.

/// A hint about the accumulation process.
///
/// It is merely a hint, so some field may not be reliable!
/// See the documentation of each field to see its guarantee.
pub struct AccumHint {
    finished: bool,
}

/// A builder for [`AccumHint`].
pub struct Builder {
    finished: bool,
}

impl AccumHint {
    /// Creates a builder to create this struct.
    #[inline]
    pub fn builder() -> Builder {
        Builder { finished: false }
    }

    /// Has the accumulation process finished?
    ///
    /// # Guarantee
    ///
    /// If this returns `true`, it is guaranteed that the accumulation process
    /// has finished.
    ///
    /// If this returns `false`, nothing is guaranteed;
    /// not even that the accumulation process has not finished.
    #[inline]
    pub fn finished(&self) -> bool {
        self.finished
    }
}

impl Builder {
    /// Sets the `finished` field.
    #[inline]
    pub fn finished(mut self, finished: bool) -> Self {
        self.finished = finished;
        self
    }

    /// Create an [`AccumHint`].
    #[inline]
    pub fn build(self) -> AccumHint {
        AccumHint {
            finished: self.finished,
        }
    }
}
