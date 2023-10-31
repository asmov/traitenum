use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Ident, Data, DataEnum, Error, parenthesized, LitStr, LitInt};

enum Orientation {
    Vertical,
    Horizontal
}

struct SheetEnumVariant {
    ident: Ident,
    headings_boxed_iterator: Ident,
    title: String,
    orientation: Orientation,
}

const ATTR_SHEET_IDENT: &'static str = "sheet";
const ATTR_SHEET_TITLE: &'static str = "title";
const ATTR_SHEET_HEADINGS: &'static str = "headings";
const ATTR_SHEET_HORIZONTAL: &'static str = "horizontal";
const ATTR_SHEET_VERTICAL: &'static str = "vertical";

#[proc_macro_derive(SheetEnum, attributes(sheet))]
pub fn sheet_enum_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let variants = match parse_sheet_variants(&input) {
        Ok(e) => e,
        Err(e) => return e
    };

    let orientation_matches_horizontal = variants.iter().filter_map(|v| {
        let name = &v.ident;
        match v.orientation {
            Orientation::Horizontal => Some(quote! {
                Self::#name => crate::spreadsheet::SheetOrientation::Horizontal,
            }),
            _ => None
        }
    });

    let orientation_matches_horizontal = quote! {
        #(#orientation_matches_horizontal)*
    };

    let title_matches = variants.iter().map(|v| {
        let name = &v.ident;
        let title = &v.title;
        quote! {
            Self::#name => #title,
        }
    });

    let title_matches = quote! {
        #(#title_matches)*
    };

    let headings_matches = variants.iter().map(|v| {
        let name = &v.ident;
        let headings_boxed_iterator_name = &v.headings_boxed_iterator;
        quote! {
            Self::#name => Box::new(#headings_boxed_iterator_name::new()),
        }
    });

    let headings_matches = quote! {
        #(#headings_matches)*
    };


    let output = quote! {
        impl #impl_generics crate::spreadsheet::SheetTrait for #name #ty_generics #where_clause {
            fn orientation(&self) -> crate::spreadsheet::SheetOrientation {
                match self {
                    #orientation_matches_horizontal
                    _ => crate::spreadsheet::SheetOrientation::Vertical
                }
            }
            
            fn title(&self) -> &'static str {
                match self {
                    #title_matches
                }
            }

            fn headings(&self) -> Box<dyn Iterator<Item = Box<dyn HeadingTrait<SheetEnum = Self>>>> {
                match self {
                    #headings_matches
                }
            }
        }
    };

    proc_macro::TokenStream::from(output)
}

fn get_enum(input: &DeriveInput) -> Result<&DataEnum, proc_macro::TokenStream> {
    match input.data {
        Data::Enum(ref data_enum) => Ok(data_enum),
        _ => Err(Error::new(input.span(),"fgcd-parse-macro: macro only supports enums").to_compile_error().into())
    }
}

fn parse_sheet_variants(input: &DeriveInput)
    -> Result<Vec<SheetEnumVariant>, proc_macro::TokenStream>
{
    let data_enum = match get_enum(&input) {
        Ok(d) => d,
        Err(e) => return Err(e)
    };

    let variants: Result<Vec<SheetEnumVariant>, Error> = data_enum.variants.iter()
        .map(|v| {
            let mut variant = SheetEnumVariant{
                ident: v.ident.clone(),
                headings_boxed_iterator: Ident::new(&format!("{}HeadingBoxedIter", v.ident.to_string()), input.span()),
                title: v.ident.to_string(),
                orientation: Orientation::Vertical };

            let attr = match v.attrs.iter().find(|a| a.path().is_ident(ATTR_SHEET_IDENT)) {
                Some(a) => a,
                None => return Ok(variant)
            };

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(ATTR_SHEET_TITLE) {
                    let content;
                    parenthesized!(content in meta.input);
                    let litstr: LitStr = content.parse()?;
                    variant.title = litstr.value();
                    Ok(())
                } else if meta.path.is_ident(ATTR_SHEET_HEADINGS) {
                    let content;
                    parenthesized!(content in meta.input);
                    let ident: Ident = content.parse()?;
                    variant.headings_boxed_iterator = Ident::new(&format!("{ident}BoxedIter"), input.span());
                    Ok(())
                }else if meta.path.is_ident(ATTR_SHEET_HORIZONTAL) {
                    variant.orientation = Orientation::Horizontal;
                    Ok(())
                } else if meta.path.is_ident(ATTR_SHEET_VERTICAL) {
                    variant.orientation = Orientation::Vertical;
                    Ok(())
                } else {
                    Err(meta.error(format!("SheetEnum: Unrecognized meta: {}", meta.path.get_ident().unwrap())))
                }
            })?;
           
            Ok(variant)
        })
        .collect();

    match variants {
        Ok(v) => Ok(v),
        Err(e) => Err(e.to_compile_error().into())
    }
}

const ATTR_HEADINGS_IDENT: &'static str = "headings";
const ATTR_HEADINGS_SHEET: &'static str = "sheet";
const ATTR_HEADING_IDENT: &'static str = "heading";
const ATTR_HEADING_TITLE: &'static str = "title";
const ATTR_HEADING_ROW: &'static str = "row";
const ATTR_HEADING_COLUMN: &'static str = "column";

struct HeadingEnumAttributes {
    sheet_enum: syn::Path,
    sheet_variant: syn::Path
}

struct HeadingEnumVariant {
    ident: Ident,
    title: String,
    row: u32,
    column: u32
}

#[proc_macro_derive(HeadingEnum, attributes(headings, heading))]
pub fn heading_enum_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let data_enum = match get_enum(&input) {
        Ok(e) => e,
        Err(e) => return e
    };

    let enum_attrs = match parse_heading_enum_attributes(&input) {
        Ok(a) => a,
        Err(e) => return e
    };

    let variants = match parse_heading_variants(data_enum) {
        Ok(e) => e,
        Err(e) => return e
    };

    let rowcol_matches = variants.iter().map(|v| {
        let name = &v.ident;
        let row = v.row;
        let column = v.column;
        quote! {
            Self::#name => crate::spreadsheet::RowCol(#row, #column),
        }
    });

    let rowcol_matches = quote! {
        #(#rowcol_matches)*
    };

    let title_matches = variants.iter().map(|v| {
        let name = &v.ident;
        let title = &v.title;
        quote! {
            Self::#name => #title,
        }
    });


    let title_matches = quote! {
        #(#title_matches)*
    };

    let sheet_enum_name = enum_attrs.sheet_enum; 
    let sheet_variant_name = enum_attrs.sheet_variant; 
    let iterator_name = Ident::new(&format!("{name}Iter"), input.span());
    let boxed_iterator_name = Ident::new(&format!("{name}BoxedIter"), input.span());

    let output = quote! {
        impl #impl_generics crate::spreadsheet::HeadingTrait for #name #ty_generics #where_clause {
            type SheetEnum = #sheet_enum_name;

            fn sheet(&self) -> Self::SheetEnum {
                #sheet_variant_name
            }

            fn title(&self) -> &'static str {
                match self {
                    #title_matches
                }
            }

            fn rowcol(&self) -> crate::spreadsheet::RowCol {
                match self {
                    #rowcol_matches
                }
            }

        }

        struct #boxed_iterator_name {
            it: #iterator_name
        }

        impl #boxed_iterator_name {
            fn new() -> Self {
                Self {
                    it: #name::iter()
                }
            }
        }
        
        impl Iterator for #boxed_iterator_name {
            type Item = Box<dyn HeadingTrait<SheetEnum = #sheet_enum_name>>;
        
            fn next(&mut self) -> Option<Self::Item> {
                match self.it.next() {
                    Some(h) => Some(Box::new(h)),
                    None => None
                }
            }
        }

    };

    proc_macro::TokenStream::from(output)
}

fn parse_heading_enum_attributes(input: &DeriveInput) -> Result<HeadingEnumAttributes, proc_macro::TokenStream> {
    let headings_attr = match input.attrs.iter().find(|a| a.path().is_ident(ATTR_HEADINGS_IDENT)) {
        Some(a) => a,
        None => return Err(Error::new(input.span(), "HeadingEnum: headings() is a required attribute")
            .to_compile_error().into())
    };

    let mut sheet_enum: Option<syn::Path> = None;
    let mut sheet_variant: Option<syn::Path> = None;

    let result = headings_attr.parse_nested_meta(|meta| {
        if meta.path.is_ident(ATTR_HEADINGS_SHEET) {
            let content;
            parenthesized!(content in meta.input);
            let variant_path: syn::Path = content.parse()?;
            let mut enum_path: syn::Path = variant_path.clone();
            enum_path.segments.pop();
            enum_path.segments.pop_punct();
            sheet_enum = Some(enum_path);
            sheet_variant = Some(variant_path);
            Ok(())
        } else {
            Err(meta.error(format!( "HeadingEnum: Unrecognized meta: {}", meta.path.get_ident().unwrap())))
        }
    });

    if let Err(e) = result {
        return Err(e.to_compile_error().into());
    }

    if sheet_enum.is_none() || sheet_variant.is_none() {
        Err(syn::Error::new(input.span(), "HeadingEnum: headings(sheet()) is a required attribute")
            .to_compile_error()
            .into())
    } else {
        Ok(HeadingEnumAttributes{
            sheet_enum: sheet_enum.unwrap(), //safe
            sheet_variant: sheet_variant.unwrap() //safe
        })
    }
}

fn parse_heading_variants(data_enum: &DataEnum) -> Result<Vec<HeadingEnumVariant>, proc_macro::TokenStream> {
    let variants: Result<Vec<HeadingEnumVariant>, Error> = data_enum.variants.iter()
        .map(|v| {
            let mut variant = HeadingEnumVariant {
                ident: v.ident.clone(),
                title: v.ident.to_string(),
                column: 0,
                row: 0 };

            let attr = match v.attrs.iter().find(|a| a.path().is_ident(ATTR_HEADING_IDENT)) {
                Some(a) => a,
                None => return Err(Error::new(v.span(), "HeadingEnum: heading() is a required variant attribute"))
            };

            let mut column_set = false;
            let mut row_set = false;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(ATTR_HEADING_TITLE) {
                    let content;
                    parenthesized!(content in meta.input);
                    let litstr: LitStr = content.parse()?;
                    variant.title = litstr.value();
                    Ok(())
                } else if meta.path.is_ident(ATTR_HEADING_COLUMN) {
                    let content;
                    parenthesized!(content in meta.input);
                    let lit: LitInt = content.parse()?;
                    variant.column = lit.base10_parse()?;
                    column_set = true;
                    Ok(())
                } else if meta.path.is_ident(ATTR_HEADING_ROW) {
                    let content;
                    parenthesized!(content in meta.input);
                    let lit: LitInt = content.parse()?;
                    variant.row = lit.base10_parse()?;
                    row_set = true;
                        Ok(())
                } else {
                    Err(meta.error(format!(
                        "HeadingEnum: Unrecognized meta: {}", meta.path.get_ident().unwrap())))
                }
            })?;
    
            if !column_set || !row_set {
                return Err(Error::new(v.span(), "HeadingEnum: row() and column() are required attributes"));
            }

            Ok(variant)
        })
        .collect();

    match variants {
        Ok(v) => Ok(v),
        Err(e) => Err(e.to_compile_error().into())
    }
}

 