use std::fmt::{self, Debug, Display, Formatter};

pub struct Error {
  msg: String,
}

impl Error {
  pub fn new<M: Into<String>>(msg: M) -> Self {
    Error { msg: msg.into() }
  }

  pub fn wrap<E: std::error::Error + Send + Sync + 'static>(err: E) -> Self {
    Error { msg: err.to_string() }
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.msg)
  }
}

impl Debug for Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "routerify::Error: {}", self.msg)
  }
}

impl std::error::Error for Error {
  fn description(&self) -> &str {
    self.msg.as_str()
  }
}

pub(crate) trait ErrorExt {
  fn wrap(self) -> Error;
  fn context<C: Display + Send + Sync + 'static>(self, ctx: C) -> Error;
}

impl<E: std::error::Error + Send + Sync + 'static> ErrorExt for E {
  fn wrap(self) -> Error {
    Error { msg: self.to_string() }
  }

  fn context<C: Display + Send + Sync + 'static>(self, ctx: C) -> Error {
    let msg = format!("{}: {}", self.to_string(), ctx);
    Error { msg }
  }
}

pub(crate) trait ResultExt<T> {
  fn wrap(self) -> Result<T, Error>;
  fn context<C: Display + Send + Sync + 'static>(self, ctx: C) -> Result<T, Error>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> ResultExt<T> for Result<T, E> {
  fn wrap(self) -> Result<T, Error> {
    self.map_err(|e| e.wrap())
  }

  fn context<C: Display + Send + Sync + 'static>(self, ctx: C) -> Result<T, Error> {
    match self {
      Ok(val) => Ok(val),
      Err(err) => Err(err.context(ctx)),
    }
  }
}
