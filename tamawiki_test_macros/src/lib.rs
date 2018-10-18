#[macro_use] extern crate quote;
extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse_macro_input, LitStr, Ident};
use std::fs;
use std::iter::FromIterator;


#[proc_macro]
pub fn make_transform_tests(tokens: TokenStream) -> TokenStream {
    let dir = parse_macro_input!(tokens as LitStr).value();
    let mut output = vec![];
    
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_stem().unwrap().to_str().unwrap();
        let n = Ident::new(name, Span::call_site());
            
        let expanded = quote! {
            #[test]
            fn #n() {
                assert_eq!(1000, 2000);
            }
        };
        output.push(TokenStream::from(expanded));
    }

    TokenStream::from_iter(output.into_iter())
}
