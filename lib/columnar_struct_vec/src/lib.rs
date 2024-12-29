extern crate proc_macro;

use quote::format_ident;
use quote::quote;
use syn::spanned::Spanned;
use syn::Field;
use syn::Ident;
use syn::Lit;
use syn::Visibility;
use syn::{parse_macro_input, Fields};

#[derive(PartialEq)]
enum DefaultType {
    None,
    FromMethod,
    FromLit(Lit),
}

struct FieldAttr {
    field: Field,
    default: DefaultType,
    error: Option<syn::Error>,
}

fn get_attrs(fields: &[&Field]) -> Vec<FieldAttr> {
    let mut attrs: Vec<FieldAttr> = vec![];
    fields.iter().for_each(|field| {
        let mut field_default = DefaultType::None;
        let mut field_error: Option<syn::Error> = None;

        for attr in &field.attrs {
            if attr.path().is_ident("struct_builder") {
                let parse_nested_result = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("default") {
                        if let Ok(value) = meta.value() {
                            // this parses the `=`
                            let s = value.parse()?;
                            field_default = DefaultType::FromLit(s);
                        } else {
                            field_default = DefaultType::FromMethod;
                        }
                        return Ok(());
                    }
                    let meta_path_str = meta.path.get_ident().unwrap().to_string();
                    return Err(syn::Error::new(
                        meta.path.span(),
                        format!("Invalid attribute {}", meta_path_str),
                    ));
                });
                parse_nested_result.clone().unwrap();
                if let Err(err) = parse_nested_result {
                    field_error = Some(err);
                    continue;
                }
            }
        }

        attrs.push(FieldAttr {
            field: (*field).clone(),
            default: field_default,
            error: field_error,
        });
    });
    return attrs;
}

fn build_row_builder_struct(
    struct_ident: &Ident,
    struct_visibility: &Visibility,
    field_attrs: &[FieldAttr],
) -> (proc_macro2::TokenStream, Ident) {
    let row_builder_struct_ident = format_ident!("{}RowBuilder", struct_ident);

    let row_builder_struct_definition = quote! {
        #struct_visibility struct #row_builder_struct_ident {
            // Prevent construction.
            _private: (),
            current_push_starting_length: usize,
            vec_struct: #struct_ident,
        }
    };

    let vec_struct_pushers: Vec<proc_macro2::TokenStream> = field_attrs
        .iter()
        .map(|field_attr| {
            let ident = &field_attr.field.ident;
            let field_type = &field_attr.field.ty;
            return quote! {
                pub fn #ident(&mut self, x:#field_type) -> () {
                    self.vec_struct.#ident.push(x);
                }
            };
        })
        .collect();

    let push_finalizer_per_field: Vec<_> = field_attrs.iter()
    .map(|field_attr| {
        let field_ident = &field_attr.field.ident;
        if let Some(err) = field_attr.error.clone() {
            return err.into_compile_error();
        }

        let ty = &field_attr.field.ty;

        return match &field_attr.default {
            DefaultType::FromLit(default_literal) => {
                // If it's a string, parse it. Otherwise, just use the literal.
                if let Lit::Str(_) = default_literal {
                    return quote! {
                        self.vec_struct.#field_ident.push(#default_literal.parse().unwrap());
                    }
                }
                return quote! {
                    self.vec_struct.#field_ident.push(#default_literal);
                }
            },
            DefaultType::None => quote! {
                return Err(anyhow::anyhow!("Missing '{}' for {}", stringify!(#field_ident), stringify!(#struct_ident)));
            },
            DefaultType::FromMethod => quote! {
                self.vec_struct.#field_ident.push(#ty::default());
            },
        };
    })
    .collect();

    let field_idents = field_attrs.iter().map(|field_attr| &field_attr.field.ident);

    let row_builder_struct_impl = quote! {
        impl #row_builder_struct_ident {
            #(#vec_struct_pushers)*

            pub fn finalize_push(mut self) -> anyhow::Result<#struct_ident> {
                let len = self.current_push_starting_length;
                #(
                    if (self.vec_struct.#field_idents.len() != len + 1) {
                        #push_finalizer_per_field
                    }
                )*
                return Ok(self.vec_struct);
            }
        }
    };

    return (
        quote! {
            #row_builder_struct_definition
            #row_builder_struct_impl
        },
        row_builder_struct_ident,
    );
}

fn build_vec_struct(
    struct_ident: &Ident,
    struct_visibility: &Visibility,
    field_attrs: &[FieldAttr],
    row_builder_struct_ident: &Ident,
) -> proc_macro2::TokenStream {
    let first_field_ident = &field_attrs[0].field.ident;
    let field_visibilities = field_attrs
        .iter()
        .map(|fa| &fa.field.vis)
        .collect::<Vec<_>>();
    let field_idents = field_attrs
        .iter()
        .map(|fa| fa.field.ident.as_ref().unwrap())
        .collect::<Vec<_>>();
    let field_types = field_attrs
        .iter()
        .map(|fa| &fa.field.ty)
        .collect::<Vec<_>>();

    let struct_definition = quote! {
        #struct_visibility struct #struct_ident {
            // Prevent direct construction.
            _private: (),
            #(
                #field_visibilities #field_idents: Vec<#field_types>
            ),*
        }
    };

    let vec_struct_impl = quote! {
        impl <'__a> Default for #struct_ident  {
            fn default() -> #struct_ident {
                return #struct_ident {
                    _private: (),
                    #(
                        #field_idents: vec![],
                    )*
                };
            }
        }
        impl <'__a> #struct_ident  {
            pub fn new(#(#field_idents: Vec<#field_types>),*) -> #struct_ident {
                return #struct_ident {
                    _private: (),
                    #(
                        #field_idents,
                    )*
                };
            }

            pub fn len(&self) -> usize {
                return self.#first_field_ident.len();
            }

            pub fn start_push(self) -> #row_builder_struct_ident {
                return #row_builder_struct_ident {
                    _private: (),
                    current_push_starting_length: self.len(),
                    vec_struct: self,
                }
            }
        }
    };

    return quote! {
        #struct_definition
        #vec_struct_impl
    };
}

#[proc_macro_attribute]
pub fn columnar_struct_vec(
    _args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as syn::ItemStruct);

    let fields = match input.fields {
        Fields::Named(fields_named) => fields_named.named,
        x => {
            return syn::Error::new(x.span(), "Only named fields are supported")
                .into_compile_error()
                .into()
        }
    };

    let field_attrs = get_attrs(&fields.iter().collect::<Vec<_>>());

    if input.generics.params.len() > 0 {
        return syn::Error::new(
            input.generics.span(),
            "We don't support structs with lifetimes or generics.",
        )
        .to_compile_error()
        .into();
    }

    let (row_builder_struct, row_builder_struct_ident) =
        build_row_builder_struct(&input.ident, &input.vis, &field_attrs);
    let vec_struct = build_vec_struct(
        &input.ident,
        &input.vis,
        &field_attrs,
        &row_builder_struct_ident,
    );

    return quote! {
        #row_builder_struct
        #vec_struct
    }
    .into();
}
