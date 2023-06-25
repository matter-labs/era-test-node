use std::{path::Path, pin::Pin};

use futures::Future;
use std::fs::File;

pub(crate) trait IntoBoxedFuture: Sized + Send + 'static {
    fn into_boxed_future(self) -> Pin<Box<dyn Future<Output = Self> + Send>> {
        Box::pin(async { self })
    }
}

impl<T, U> IntoBoxedFuture for Result<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
{
}

pub fn read_file_to_json_value(path: impl AsRef<Path>) -> serde_json::Value {
    //let path: Path = path.into();
    serde_json::from_reader(
        File::open(&path)
            .unwrap_or_else(|e| panic!("Failed to open file {:?}: {}", path.as_ref(), e)),
    )
    .unwrap_or_else(|e| panic!("Failed to parse file {:?}: {}", path.as_ref(), e))
}
