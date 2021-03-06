extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use quote::ToTokens;

#[proc_macro_derive(BotCommand)]
pub fn bot_command(input: TokenStream) -> TokenStream {
  // Parse the string representation
  let ast = syn::parse(input).unwrap();

  // Build the impl
  impl_bot_command(&ast)
}

fn impl_bot_command(ast: &syn::DeriveInput) -> TokenStream {
  let struct_data = match ast.data {
    syn::Data::Struct(ref s) => s,
    _ => panic!("cannot derive BotCommand on anything but a struct"),
  };
  let name = &ast.ident;
  let mut field_name = None;
  for field in struct_data.fields.iter() {
    let path = match field.ty {
      syn::Type::Path(ref p) => p,
      _ => continue,
    };
    if path.path.clone().into_token_stream().to_string() == "Arc < BotEnv >" {
      field_name = Some(field.ident.clone().expect("cannot derive for tuple structs"));
      break;
    }
  }
  match field_name {
    Some(ident) => quote! {
      impl crate::commands::BotCommand for #name {
        #[cfg_attr(feature = "cargo-clippy", allow(redundant_field_names))]
        fn new(env: Arc<crate::bot::BotEnv>) -> Self {
          #name { #ident: env }
        }
      }
    }.into(),
    None => quote! {
      impl crate::commands::BotCommand for #name {
        fn new(_: Arc<crate::bot::BotEnv>) -> Self {
          #name
        }
      }
    }.into(),
  }
}
