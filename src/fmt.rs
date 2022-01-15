
use crate::chain::Chain;
use crate::error::ErrorImpl;
use crate::ptr::Ref;
use core::fmt::{self, Debug, Write};

impl ErrorImpl {
    pub(crate) unsafe fn display(this: Ref<Self>, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Self::error(this))?;

        if f.alternate() {
            for cause in Self::chain(this).skip(1) {
                write!(f, ": {}", cause)?;
            }
        }

        Ok(())
    }

    pub(crate) unsafe fn debug(this: Ref<Self>, f: &mut fmt::Formatter) -> fmt::Result {
        let error = Self::error(this);

        if f.alternate() {
            return Debug::fmt(error, f);
        }

        write!(f, "{}", error)?;
