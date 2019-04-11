use heck::*;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{parenthesized, parse_macro_input, token, Ident, Token, Type, Lifetime};

pub fn make(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(item as Able);
    let t = quote!(#parsed);
    dbg!(format!("{:#}", t));
    proc_macro::TokenStream::from(t)
}

pub(crate) struct Able {
    name: Ident,
    _paren: Option<token::Paren>,
    params: Option<Punctuated<Type, Token![,]>>,
    _colon: Option<Token![:]>,
    extends: Option<Punctuated<Ident, Token![+]>>,
}

impl Parse for Able {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut extends_present = false;
        let mut params = None;
        Ok(Self {
            name: input.parse()?,
            _paren: {
                let lookahead = input.lookahead1();
                if lookahead.peek(token::Paren) {
                    let content;
                    let paren = parenthesized!(content in input);
                    params = Some(content);
                    Some(paren)
                } else {
                    None
                }
            },
            params: params.map(|content| content.parse_terminated(Type::parse).unwrap()),
            _colon: {
                let lookahead = input.lookahead1();
                if lookahead.peek(Token![:]) {
                    extends_present = true;
                    Some(input.parse()?)
                } else {
                    None
                }
            },
            extends: if extends_present {
                Some(input.parse_terminated(Ident::parse).unwrap())
            } else {
                None
            },
        })
    }
}

impl ToTokens for Able {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.name.to_string().to_camel_case();
        let on_ident = Ident::new(&format!("On{}", ident).to_camel_case(), Span::call_site());
        
        let ident_able = Ident::new(&format!("{}able", ident).to_camel_case(), Span::call_site());
        let ident_able_inner = Ident::new(&format!("{}ableInner", ident).to_camel_case(), Span::call_site());
        
        let ident_fn = Ident::new(&format!("{}", ident).to_snake_case(), Span::call_site());
        let on_ident_fn = Ident::new(&format!("on_{}", ident).to_snake_case(), Span::call_site());
        
        let oopify = &crate::oopify::Oopify {
	        ident: ident_able.clone()
        };
        
        let params = self
            .params
            .as_ref()
            .map(|punct| punct.iter().map(|i| i.clone()).collect::<Vec<_>>())
            .unwrap_or(vec![]);
        let params2 = params.clone();
        let param_names = (0..params.len())
            .map(|i| Ident::new(&format!("arg{}", i), Span::call_site()))
            .collect::<Vec<_>>();
            
        let params_inner = params.clone();
        let params2_inner = params2.clone();
        let param_names_inner = param_names.clone();    
        
        let params_callback = params.clone();     
            
        let extends = self.extends.as_ref()
            .map(|punct| punct.iter().map(|i| i.clone()).collect::<Vec<_>>())
            .unwrap_or(vec![]);
        let extends_inner = self.extends.as_ref()
            .map(|punct| punct.iter().map(|i| Ident::new(&format!("{}Inner", i.to_string().to_camel_case()), Span::call_site())).collect::<Vec<_>>())
            .unwrap_or(vec![]);   
            
        let static_ = Lifetime::new("'static", Span::call_site());  
        let static_inner = Lifetime::new("'static", Span::call_site());   
            
        let expr = quote! {
            pub trait #ident_able: #static_ + AsAny #(+#extends)*{
                fn #ident_fn(&mut self, #(#param_names: #params,)* skip_callbacks: bool);
                fn #on_ident_fn(&mut self, callback: Option<#on_ident>);

                #oopify
            }
            pub trait #ident_able_inner: #(#extends_inner+)* #static_inner {
                fn #ident_fn(&mut self, #(#param_names_inner: #params_inner,)* skip_callbacks: bool);
                fn #on_ident_fn(&mut self, callback: Option<Box<FnMut(&mut #ident_able #(,#params2_inner)* )>>);
            }
            
            pub struct #on_ident(CallbackId, Box<dyn FnMut(&mut #ident_able #(,#params_callback)* )>);
        };
        expr.to_tokens(tokens);
    }
}
