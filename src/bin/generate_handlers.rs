extern crate syn;
extern crate quote;

use syn::{File as SynFile, Item, TraitItem, FnArg};

use quote::ToTokens;

use std::fs::File;
use std::env::args;
use std::io::Read;

fn main() {
  let path = args().nth(1).unwrap();
  let mut file = File::open(path).unwrap();
  let mut content = String::new();
  file.read_to_string(&mut content).unwrap();
  let file: SynFile = syn::parse_str(&content).unwrap();
  for item in file.items {
    let t = match item {
      Item::Trait(t) => t,
      _ => continue
    };
    if t.ident.to_string() != "EventHandler" {
      continue;
    }
    'item: for item in t.items {
      let m = match item {
        TraitItem::Method(m) => m,
        _ => continue
      };
      if !m.attrs.is_empty() {
        // I'm sorry
        for attr in m.attrs {
          if attr.into_token_stream().to_string() == r#"# [ cfg ( not ( feature = "cache" ) ) ]"# {
            continue 'item;
          }
        }
      }
      let method_ident = m.sig.ident;
      let method_args = m.sig.inputs;
      print!("handler!({}, ", method_ident);
      let num_args = method_args.len();
      for (i, arg) in method_args.into_iter().enumerate() {
        match arg {
          FnArg::Typed(captured) => print!("param{}: {}", i, captured.ty.into_token_stream()),
          _ => continue,
        }
        if i != num_args - 1 {
          print!(", ");
        }
      }
      println!(");");
    }
  }
}
