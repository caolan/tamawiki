#[macro_use]
extern crate quote;
#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;
extern crate proc_macro2;
extern crate rustfmt;
extern crate serde;
extern crate serde_json;
extern crate syn;

use proc_macro2::Span;
use proc_macro2::{Delimiter, Group, Punct, Spacing, TokenStream};
use quote::{ToTokens, TokenStreamExt};
use rustfmt::Input;
use std::collections::HashMap;
use std::default::Default;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use syn::Ident;

const SHARED_APPLY_TESTS: &'static str = "./public/src/_static/js/tests/shared/apply.json";

const SHARED_TRANSFORM_TESTS: &'static str = "./public/src/_static/js/tests/shared/transform.json";

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("shared_tests.rs");
    let mut outfile = File::create(&out_path).unwrap();

    println!("cargo:rerun-if-changed={}", SHARED_APPLY_TESTS);
    println!("cargo:rerun-if-changed={}", SHARED_TRANSFORM_TESTS);

    outfile
        .write(b"// WARNING this file is auto-generated by build.rs. Do not edit directly!\n")
        .unwrap();

    let mut apply_file = File::open(SHARED_APPLY_TESTS).unwrap();
    let mut apply_content = String::new();
    apply_file.read_to_string(&mut apply_content).unwrap();

    let apply_configs: Vec<ApplyTestConfig> = serde_json::from_str(&apply_content).unwrap();

    for config in apply_configs.into_iter() {
        write_apply_test(&mut outfile, config);
    }

    let mut transform_file = File::open(SHARED_TRANSFORM_TESTS).unwrap();
    let mut transform_content = String::new();
    transform_file
        .read_to_string(&mut transform_content)
        .unwrap();

    let transform_configs: Vec<TransformTestConfig> =
        serde_json::from_str(&transform_content).unwrap();

    for config in transform_configs.into_iter() {
        write_transform_test(&mut outfile, config);
    }

    // Format the generated tests using rustfmt, otherwise the test
    // functions are each collapsed onto one line only, making
    // tracking down test errors more painful.
    let summary = rustfmt::run(Input::File(out_path), &Default::default());

    assert!(!summary.has_parsing_errors());
    assert!(!summary.has_operational_errors());
    // ignore formatting errors, since we don't mind if it's not formatted perfectly
}

// include the type definitions here so we can deserialize the JSON
// test configs using them
include!("./src/document/types.rs");

#[derive(Debug, Deserialize)]
struct ApplyTestConfig {
    name: String,
    initial: Document,
    expected: Document,
    events: Vec<Event>,
    error: Option<EditError>,
}

#[derive(Debug, Deserialize)]
struct TransformTestConfig {
    name: String,
    initial: Event,
    expected: Event,
    concurrent: Vec<Event>,
}

fn field<F>(tokens: &mut TokenStream, name: &'static str, mut f: F)
where
    F: FnMut(&mut TokenStream) -> (),
{
    tokens.append(Ident::new(name, Span::call_site()));
    tokens.append(Punct::new(':', Spacing::Joint));
    f(tokens);
    tokens.append(Punct::new(',', Spacing::Joint));
}

impl ToTokens for Document {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut fields = TokenStream::new();
        field(&mut fields, "content", |tokens| {
            tokens.append(Ident::new("String", Span::call_site()));
            tokens.append(Punct::new(':', Spacing::Joint));
            tokens.append(Punct::new(':', Spacing::Alone));
            tokens.append(Ident::new("from", Span::call_site()));
            let mut inner = TokenStream::new();
            self.content.to_tokens(&mut inner);
            tokens.append(Group::new(Delimiter::Parenthesis, inner));
        });
        field(&mut fields, "participants", |tokens| {
            self.participants.to_tokens(tokens)
        });
        tokens.append(Ident::new("Document", Span::call_site()));
        tokens.append(Group::new(Delimiter::Brace, fields));
    }
}

impl ToTokens for Participants {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Ident::new("Participants", Span::call_site()));
        tokens.append(Punct::new(':', Spacing::Joint));
        tokens.append(Punct::new(':', Spacing::Alone));

        let mut values = TokenStream::new();
        for (k, v) in &self.entries {
            let mut pair = TokenStream::new();
            k.to_tokens(&mut pair);
            pair.append(Punct::new(',', Spacing::Joint));
            v.to_tokens(&mut pair);
            values.append(Group::new(Delimiter::Parenthesis, pair));
            values.append(Punct::new(',', Spacing::Joint));
        }
        let mut wrapper = TokenStream::new();
        wrapper.append(Ident::new("vec", Span::call_site()));
        wrapper.append(Punct::new('!', Spacing::Joint));
        wrapper.append(Group::new(Delimiter::Bracket, values));

        tokens.append(Ident::new("from_iter", Span::call_site()));
        tokens.append(Group::new(Delimiter::Parenthesis, wrapper));
    }
}

impl ToTokens for DocumentParticipant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut fields = TokenStream::new();
        field(&mut fields, "cursor_pos", |tokens| {
            self.cursor_pos.to_tokens(tokens)
        });
        tokens.append(Ident::new("DocumentParticipant", Span::call_site()));
        tokens.append(Group::new(Delimiter::Brace, fields));
    }
}

impl ToTokens for Event {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Ident::new("Event", Span::call_site()));
        tokens.append(Punct::new(':', Spacing::Joint));
        tokens.append(Punct::new(':', Spacing::Alone));
        let mut inner = TokenStream::new();
        match *self {
            Event::Join(ref data) => {
                tokens.append(Ident::new("Join", Span::call_site()));
                data.to_tokens(&mut inner);
                tokens.append(Group::new(Delimiter::Parenthesis, inner));
            }
            Event::Leave(ref data) => {
                tokens.append(Ident::new("Leave", Span::call_site()));
                data.to_tokens(&mut inner);
                tokens.append(Group::new(Delimiter::Parenthesis, inner));
            }
            Event::Edit(ref data) => {
                tokens.append(Ident::new("Edit", Span::call_site()));
                data.to_tokens(&mut inner);
                tokens.append(Group::new(Delimiter::Parenthesis, inner));
            }
        }
    }
}

impl ToTokens for Join {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut fields = TokenStream::new();
        field(&mut fields, "id", |tokens| self.id.to_tokens(tokens));
        tokens.append(Ident::new("Join", Span::call_site()));
        tokens.append(Group::new(Delimiter::Brace, fields));
    }
}

impl ToTokens for Leave {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut fields = TokenStream::new();
        field(&mut fields, "id", |tokens| self.id.to_tokens(tokens));
        tokens.append(Ident::new("Leave", Span::call_site()));
        tokens.append(Group::new(Delimiter::Brace, fields));
    }
}

impl ToTokens for Edit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut fields = TokenStream::new();
        field(&mut fields, "author", |tokens| {
            self.author.to_tokens(tokens)
        });
        field(&mut fields, "operations", |tokens| {
            let mut inner = TokenStream::new();
            for op in &self.operations {
                op.to_tokens(&mut inner);
                inner.append(Punct::new(',', Spacing::Joint));
            }
            tokens.append(Ident::new("vec", Span::call_site()));
            tokens.append(Punct::new('!', Spacing::Joint));
            tokens.append(Group::new(Delimiter::Bracket, inner));
        });
        tokens.append(Ident::new("Edit", Span::call_site()));
        tokens.append(Group::new(Delimiter::Brace, fields));
    }
}

impl ToTokens for Operation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Ident::new("Operation", Span::call_site()));
        tokens.append(Punct::new(':', Spacing::Joint));
        tokens.append(Punct::new(':', Spacing::Alone));
        let mut inner = TokenStream::new();
        match *self {
            Operation::Insert(ref data) => {
                tokens.append(Ident::new("Insert", Span::call_site()));
                data.to_tokens(&mut inner);
                tokens.append(Group::new(Delimiter::Parenthesis, inner));
            }
            Operation::Delete(ref data) => {
                tokens.append(Ident::new("Delete", Span::call_site()));
                data.to_tokens(&mut inner);
                tokens.append(Group::new(Delimiter::Parenthesis, inner));
            }
        }
    }
}

impl ToTokens for Insert {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut fields = TokenStream::new();
        field(&mut fields, "pos", |tokens| self.pos.to_tokens(tokens));
        field(&mut fields, "content", |tokens| {
            tokens.append(Ident::new("String", Span::call_site()));
            tokens.append(Punct::new(':', Spacing::Joint));
            tokens.append(Punct::new(':', Spacing::Alone));
            tokens.append(Ident::new("from", Span::call_site()));
            let mut inner = TokenStream::new();
            self.content.to_tokens(&mut inner);
            tokens.append(Group::new(Delimiter::Parenthesis, inner));
        });
        tokens.append(Ident::new("Insert", Span::call_site()));
        tokens.append(Group::new(Delimiter::Brace, fields));
    }
}

impl ToTokens for Delete {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut fields = TokenStream::new();
        field(&mut fields, "start", |tokens| self.start.to_tokens(tokens));
        field(&mut fields, "end", |tokens| self.end.to_tokens(tokens));
        tokens.append(Ident::new("Delete", Span::call_site()));
        tokens.append(Group::new(Delimiter::Brace, fields));
    }
}

impl ToTokens for EditError {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Ident::new("EditError", Span::call_site()));
        tokens.append(Punct::new(':', Spacing::Joint));
        tokens.append(Punct::new(':', Spacing::Alone));
        tokens.append(Ident::new(
            match *self {
                EditError::OutsideDocument => "OutsideDocument",
                EditError::InvalidOperation => "InvalidOperation",
            },
            Span::call_site(),
        ));
    }
}

fn write_apply_test(mut output: &File, config: ApplyTestConfig) {
    match config {
        ApplyTestConfig {
            name,
            initial,
            events,
            error,
            expected,
        } => {
            let name = Ident::new(&name, Span::call_site());
            write!(
                output,
                "{}\n",
                if error.is_some() {
                    quote! {
                        #[test]
                        fn #name() {
                            let mut doc = #initial;
                            #(let last_result = doc.apply(&#events);)*
                            assert_eq!(last_result, Err(#error));
                            assert_eq!(doc, #expected);
                        }
                    }
                } else {
                    quote! {
                        #[test]
                        fn #name() {
                            let mut doc = #initial;
                            #(doc.apply(&#events).unwrap();)*
                            assert_eq!(doc, #expected);
                        }
                    }
                }
            ).unwrap();
        }
    }
}

fn write_transform_test(mut output: &File, config: TransformTestConfig) {
    match config {
        TransformTestConfig {
            name,
            initial,
            concurrent,
            expected,
        } => {
            let name = Ident::new(&name, Span::call_site());
            write!(
                output,
                "{}\n",
                quote! {
                    #[test]
                    fn #name() {
                        let mut event = #initial;
                        #(event.transform(&#concurrent);)*
                        assert_eq!(event, #expected);
                    }
                }
            ).unwrap();
        }
    }
}
