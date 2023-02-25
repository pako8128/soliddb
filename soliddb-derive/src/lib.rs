use darling::{
    ast::{Data, Fields},
    FromDeriveInput, FromField, FromVariant,
};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

#[derive(FromDeriveInput)]
#[darling(attributes(solid), supports(enum_any, struct_named))]
struct ItemOpts {
    ident: syn::Ident,
    data: Data<VariantOpts, FieldOpts>,
    table: u32,
}

#[derive(FromVariant)]
#[darling(attributes(solid))]
struct VariantOpts {
    fields: Fields<FieldOpts>,
}

#[derive(Clone, FromField)]
#[darling(attributes(solid))]
struct FieldOpts {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    #[darling(default)]
    unique: bool,
    #[darling(default)]
    indexed: bool,
}

#[derive(FromDeriveInput)]
#[darling(attributes(solid), supports(any))]
struct SingleOpts {
    ident: syn::Ident,
    single: u32,
}

#[proc_macro_derive(Single, attributes(solid))]
pub fn derive_single(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let SingleOpts { ident, single } = match SingleOpts::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let output = quote! {
        impl ::soliddb::Single for #ident {
            const SINGLE: u32 = #single;
        }
    };
    output.into()
}

#[proc_macro_derive(Table, attributes(solid))]
pub fn derive_item(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let ItemOpts { ident, table, data } = match ItemOpts::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    if table == 0 {
        panic!("table 0 is reserved");
    }

    match data {
        Data::Struct(fields) => gen_struct(ident, table, fields),
        Data::Enum(variants) => gen_enum(ident, table, variants),
    }
}

fn gen_struct(ident: syn::Ident, table: u32, fields: Fields<FieldOpts>) -> TokenStream {
    let unique_fields = find_unique_fields(&fields);
    let unique_field_names: Vec<_> = unique_fields
        .iter()
        .cloned()
        .map(|field| field.ident.unwrap())
        .collect();

    let indexed_fields = find_indexed_fields(&fields);
    let indexed_field_names: Vec<_> = indexed_fields
        .iter()
        .cloned()
        .map(|field| field.ident.unwrap())
        .collect();

    if unique_fields.len() > 126 {
        panic!("only 126 unique indices per table are allowed");
    }
    if indexed_fields.len() > 126 {
        panic!("only 126 non unique indices per table are allowed");
    }

    let unique_keys: Vec<_> = (1u8..).take(unique_fields.len()).collect();
    let indexed_keys: Vec<_> = (128u8..).take(indexed_fields.len()).collect();

    let unique_getters =
        unique_keys
            .iter()
            .copied()
            .zip(unique_fields.iter())
            .map(|(index, field)| {
                unique_getter_method(index, field.ident.as_ref().unwrap(), &field.ty)
            });

    let indexed_getters =
        indexed_keys
            .iter()
            .copied()
            .zip(indexed_fields.iter())
            .map(|(index, field)| {
                indexed_getter_method(index, field.ident.as_ref().unwrap(), &field.ty)
            });

    let unique_value_func = if unique_keys.is_empty() {
        quote! {}
    } else {
        quote! {
            fn unique_value(&self, index: u8) -> Vec<u8> {
                match index {
                    #(#unique_keys => ::soliddb::IndexValue::as_bytes(&self.#unique_field_names),)*
                    _ => unreachable!("no unique value for index {}", index),
                }
            }
        }
    };

    let non_unique_value_func = if indexed_fields.is_empty() {
        quote! {}
    } else {
        quote! {
            fn non_unique_value(&self, index: u8) -> Vec<u8> {
                match index {
                    #(#indexed_keys => ::soliddb::IndexValue::as_bytes(&self.#indexed_field_names),)*
                    _ => unreachable!("no non unique value for index {}", index),
                }
            }
        }
    };

    let output = quote! {
        impl ::soliddb::Table for #ident {
            const TABLE: u32 = #table;
            const UNIQUE_INDICES: &'static [u8] = &[#(#unique_keys),*];
            const NON_UNIQUE_INDICES: &'static [u8] = &[#(#indexed_keys),*];

            #unique_value_func
            #non_unique_value_func
        }

        impl #ident {
            #(#unique_getters)*
            #(#indexed_getters)*
        }
    };
    output.into()
}

fn gen_enum(ident: syn::Ident, table: u32, variants: Vec<VariantOpts>) -> TokenStream {
    for variant in variants {
        if !find_unique_fields(&variant.fields).is_empty() {
            panic!("unique fields are not allowed for enums");
        }

        if !find_indexed_fields(&variant.fields).is_empty() {
            panic!("indexed fields are not allowed for enums");
        }
    }

    let output = quote! {
        impl ::soliddb::Table for #ident {
            const TABLE: u32 = #table;
        }
    };
    output.into()
}

fn find_unique_fields(fields: &Fields<FieldOpts>) -> Vec<FieldOpts> {
    fields
        .iter()
        .cloned()
        .filter(|field| field.unique)
        .collect()
}

fn find_indexed_fields(fields: &Fields<FieldOpts>) -> Vec<FieldOpts> {
    fields
        .iter()
        .cloned()
        .filter(|field| field.indexed)
        .collect()
}

fn unique_getter_method(index: u8, field: &syn::Ident, ty: &syn::Type) -> proc_macro2::TokenStream {
    let method = format_ident!("get_by_{field}");

    quote! {
        pub fn #method(db: &::soliddb::DB, value: &#ty) -> ::soliddb::Result<::soliddb::WithId<Self>> {
            let value = <#ty as ::soliddb::IndexValue>::as_bytes(value);
            Self::get_by_unique_index(db, #index, &value)
        }
    }
}

fn indexed_getter_method(
    index: u8,
    field: &syn::Ident,
    ty: &syn::Type,
) -> proc_macro2::TokenStream {
    let method = format_ident!("get_by_{field}");

    quote! {
        pub fn #method(db: &::soliddb::DB, value: &#ty) -> ::soliddb::Result<Vec<::soliddb::WithId<Self>>> {
            let value = <#ty as ::soliddb::IndexValue>::as_bytes(value);
            Self::get_by_non_unique_index(db, #index, &value)
        }
    }
}
