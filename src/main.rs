#![feature(extend_one)]
#![feature(try_trait_v2)]

use std::env;
use std::iter;

trait IterExt {
  type Item;
  fn split_with<B, F, R>(self, f: F) -> (B, R)
  where
    Self: Sized,
    B: Default + Extend<Self::Item>,
    R: iter::IntoIterator<Item = Self::Item> + iter::FromIterator<Self::Item>,
    F: FnMut(&Self::Item) -> bool;
}
impl<T: Iterator> IterExt for T {
  type Item = <Self as Iterator>::Item;

  fn split_with<B, F, R>(mut self, mut f: F) -> (B, R)
  where
    Self: Sized,
    B: Default + Extend<Self::Item>,
    F: FnMut(&Self::Item) -> bool,
    R: iter::IntoIterator<Item = Self::Item> + iter::FromIterator<Self::Item>,
  {
    let mut left: B = Default::default();
    let mut mid: Option<Self::Item> = None;

    while let Some(x) = self.next() {
      if f(&x) {
        left.extend_one(x);
      } else {
        mid = Some(x);
        break;
      }
    }

    let right: R = if let Some(mid) = mid {
      iter::once(mid).chain(self).collect()
    } else {
      self.collect()
    };

    (left, right)
  }
}

use regex;
use std::io::{self, Write};
use std::process::{self, Command};

fn main() {
  let mut args = env::args().skip(1);
  // let (opts, pargs): (Vec<String>, Vec<String>) = args.split_with(|x| x.starts_with("-"));
  let re = regex::Regex::new(r"-n(-?\d+)?").unwrap();
  let mut n = 0;
  let mut pargs = vec![];
  while let Some(arg) = args.next() {
    if arg.starts_with("-") {
      if let Some(pcaps) = re.captures(arg.as_str()) {
        let ns = if let Some(ns) = pcaps.get(1) {
          Some(ns.as_str().to_owned())
        } else {
          args.next().map(|s| s)
        };
        if let Some(Ok(nr)) = ns.map(|ns| ns.parse::<i32>()) {
          n = nr;
        }
      }
    } else {
      pargs.push(arg);
      pargs.extend(args);
      break;
    }
  }
  let pargs = pargs;
  if pargs.len() == 0 {
    println!("{}", n);
  } else {
    let level = {
      if n >= 13 {
        "LOW"
      } else if n >= 6 {
        "BELOWNORMAL"
      } else if n >= -5 {
        "NORMAL"
      } else if n >= -11 {
        "ABOVENORMAL"
      } else if n >= -18 {
        "HIGH"
      } else {
        "REALTIME"
      }
    };

    let output = Command::new(
      // r"C:\Program Files (x86)\Microsoft Configuration Manager\AdminConsole\bin\i386\CmRcViewer.exe",
      "cmd",
    )
    // .arg("/q")
    .arg("/c")
    .args(&["start", format!("/{}", level).as_str()])
    .arg("/b") // prevent new window
    .args(pargs)
    .output()
    .expect("Failed to run command");

    let _ = io::stdout().write_all(&output.stdout);
    let _ = io::stderr().write_all(&output.stderr);
    if let Some(ec) = output.status.code() {
      process::exit(ec);
    } else {
      process::exit(1);
    }
  }
}
