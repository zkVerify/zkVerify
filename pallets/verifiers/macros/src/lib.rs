// Copyright 2024, Horizen Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This crate expose some procedural macro utils for implementing
//! a new verifier pallet based on `pallet-verifiers` abstraction.
//!

use std::fmt::Debug;

use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote, Attribute, Ident, Token, Type, TypePath, Visibility,
};

#[derive(Clone, Debug, PartialEq)]
struct GenericType {
    pub l_angular: Token![<],
    pub t: Type,
    pub r_angular: Token![>],
}

impl syn::parse::Parse for GenericType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(GenericType {
            l_angular: input.parse()?,
            t: input.parse()?,
            r_angular: input.parse()?,
        })
    }
}

impl ToTokens for GenericType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.l_angular.to_tokens(tokens);
        self.t.to_tokens(tokens);
        self.r_angular.to_tokens(tokens);
    }
}

struct Item {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub struct_token: Token![struct],
    pub ident: Ident,
    pub generic: Option<GenericType>,
    pub semi_token: Token![;],
}

impl Parse for Item {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        let struct_token = input.parse()?;
        let ident = input.parse()?;
        let lookahead = input.lookahead1();
        let generic = if lookahead.peek(Token![<]) {
            Some(input.parse()?)
        } else {
            None
        };
        let semi_token = input.parse()?;
        Ok(Item {
            attrs,
            vis,
            struct_token,
            ident,
            generic,
            semi_token,
        })
    }
}

/// The attribute `#[verifier]` can be used on a new struct that should implement
/// `pallet-verifier::Verifier` trait: will generate the need type aliases and
/// reexport the `pallet-verifiers` substrate generated stuff needed to
/// use this crate or module as the home of the new pallet.
///
/// It accept only structs without fields and generics.
///
#[proc_macro_attribute]
pub fn verifier(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let _ = parse_macro_input!(attr as syn::parse::Nothing);
    verifier_render(parse_macro_input!(item as Item)).into()
}

fn verifier_render(item: Item) -> proc_macro2::TokenStream {
    let Item {
        attrs,
        vis,
        struct_token,
        ident,
        generic,
        semi_token,
    } = item;
    let crate_name = crate_name();
    let t = generic
        .clone()
        .map(|t| t.t)
        .unwrap_or_else(|| parse_quote! { T });
    let phantom = generic
        .as_ref()
        .map(|t| quote! { ( core::marker::PhantomData #t ) });

    quote! {
        #(#attrs)*
        #vis #struct_token #ident #generic #phantom #semi_token
        pub type Pallet<#t> = #crate_name::Pallet<#t, #ident #generic>;
        pub type Event<#t> = #crate_name::Event<#t, #ident #generic>;
        pub type Error<#t> = #crate_name::Error<#t, #ident #generic>;
        pub type Tickets<#t> = #crate_name::Tickets<#t, #ident #generic>;
        pub use #crate_name::{
            __substrate_call_check, __substrate_event_check, tt_default_parts, tt_error_token,
        };
    }
}

#[derive(Clone, Debug, PartialEq)]
struct BenchmarkingUtils {
    verifier: Ident,
    generic: Option<GenericType>,
    config: Option<TypePath>,
}

impl Parse for BenchmarkingUtils {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let verifier = input.parse()?;
        let lookahead = input.lookahead1();
        let generic = if lookahead.peek(Token![<]) {
            Some(input.parse()?)
        } else {
            None
        };
        let lookahead = input.lookahead1();
        let config = if lookahead.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self {
            verifier,
            generic,
            config,
        })
    }
}

#[proc_macro]
pub fn benchmarking_utils(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let BenchmarkingUtils {
        verifier,
        generic,
        config,
    } = parse_macro_input!(input as BenchmarkingUtils);
    let crate_name = crate_name();
    let fish = generic.clone().map(|t| quote! {  ::#t });
    let verifier_call = quote! { #verifier #fish};
    let verifier = {
        quote! { #verifier #generic }
    };
    let opt_cfg_bound = config.clone().map(|c| quote! { + #c });
    let vk_of = quote! { VkOf #generic };
    let proof_of = quote! { ProofOf #generic };
    let pubs_of = quote! { PubsOf #generic };
    quote! {
        type #vk_of = <#verifier as #crate_name::benchmarking_utils::Verifier>::Vk;
        type #proof_of = <#verifier as #crate_name::benchmarking_utils::Verifier>::Proof;
        type #pubs_of = <#verifier as #crate_name::benchmarking_utils::Verifier>::Pubs;

        /// execute verify_proof
        fn do_verify_proof<T>(
            vk: &#vk_of,
            proof: &#proof_of,
            pubs: &#pubs_of,
        ) -> Result<Option<#crate_name::benchmarking_utils::Weight>, #crate_name::benchmarking_utils::VerifyError>
        where
            T: #crate_name::Config<#verifier> #opt_cfg_bound,
        {
            #verifier_call::verify_proof(vk, proof, pubs)
        }

        /// Get a `VkEntry` from Vks storage.
        fn do_get_vk<T>(hash: &sp_core::H256) -> Option<#crate_name::VkEntry<#vk_of>>
        where
            T: #crate_name::Config<#verifier> #opt_cfg_bound,
        {
            #crate_name::Vks::<T, #verifier>::get(&hash)
        }

        /// Validate a given vk.
        fn do_validate_vk<T>(vk: &#vk_of) -> Result<(), #crate_name::benchmarking_utils::VerifyError>
        where
            T: #crate_name::Config<#verifier> #opt_cfg_bound,
        {
            #verifier_call::validate_vk(vk)
        }

        /// Compute the statement hash.
        fn do_compute_statement_hash<T>(
            vk_or_hash: &#crate_name::VkOrHash<#vk_of>,
            proof: &#proof_of,
            pubs: &#pubs_of,
        ) -> sp_core::H256
        where
            T: #crate_name::Config<#verifier>  #opt_cfg_bound,
        {
            #crate_name::compute_statement_hash::<#verifier>(vk_or_hash, proof, pubs)
        }

        /// Compute the vk hash.
        fn do_vk_hash<T>(vk: &#vk_of) -> sp_core::H256
        where
            T: #crate_name::Config<#verifier>  #opt_cfg_bound,
        {
            #verifier_call::vk_hash(vk)
        }

        /// Return a new account with enough founds to do anything.
        fn funded_account<T>() -> T::AccountId
        where
            T: #crate_name::Config<#verifier> #opt_cfg_bound,
        {
            #crate_name::benchmarking_utils::funded_account::<T, #verifier>()
        }

        /// Insert a vk in the storage.
        fn insert_vk<T>(owner: T::AccountId, vk: #vk_of, hash: sp_core::H256)
        where
            T: #crate_name::Config<#verifier>  #opt_cfg_bound,
        {
            #crate_name::benchmarking_utils::insert_vk::<T, #verifier>(owner, vk, hash)
        }

        /// Insert a vk in the storage owned by an anonymous.
        fn insert_vk_anonymous<T>(vk: #vk_of, hash: sp_core::H256)
        where
            T: #crate_name::Config<#verifier>  #opt_cfg_bound,
        {
            #crate_name::benchmarking_utils::insert_vk_anonymous::<T, #verifier>(vk, hash)
        }
    }
    .into()
}

#[cfg(not(test))]
fn crate_name() -> syn::Path {
    use proc_macro_crate::FoundCrate;
    use quote::format_ident;

    match proc_macro_crate::crate_name("pallet-verifiers")
        .expect("pallet-verifiers is present in `Cargo.toml` qed")
    {
        FoundCrate::Itself => parse_quote! { crate },
        FoundCrate::Name(name) => {
            let myself = format_ident!("{name}");
            parse_quote! { #myself }
        }
    }
}
#[cfg(test)]
fn crate_name() -> syn::Path {
    parse_quote! { pallet_verifiers }
}

#[cfg(test)]
mod tests {
    // Note: here we test just the parsing stuff. Logic and functionalities are tested
    // in the `pallet-verifiers` crate (the `FakeVerifier` in mock module use this macro)
    // .

    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    #[rstest]
    #[case("pub struct Verifier;")]
    #[case("pub struct Other;")]
    #[case::no_pub("struct Verifier;")]
    #[case::generic_with_type_reference("struct Verifier<A>;")]
    #[case::comments(
        r#"
    /// comm
    /// ents
    pub struct Verifier;"#
    )]
    fn should_parse_valid_item(#[case] input: &str) {
        assert!(syn::parse_str::<Item>(input).is_ok())
    }

    #[rstest]
    #[case::named_tuple("struct Verifier(Other);")]
    #[case::field("struct Other{a: u32}")]
    #[case::generic_with_bound("struct Verifier<A: B>;")]
    #[case::generic_lifetime("struct Verifier<'a>;")]
    #[case::generics_more_than_one("struct Verifier<A, B>;")]
    #[case::enum_type("enum Verifier;")]
    fn should_reject_invalid_item(#[case] input: &str) {
        assert!(syn::parse_str::<Item>(input).is_err())
    }

    #[test]
    fn happy_path() {
        let expected: syn::ItemMod = parse_quote! {
            mod a {
                #[a1]
                #[a2]
                pub struct Ver;
                pub type Pallet<T> = pallet_verifiers::Pallet<T, Ver>;
                pub type Event<T> = pallet_verifiers::Event<T, Ver>;
                pub type Error<T> = pallet_verifiers::Error<T, Ver>;
                pub type Tickets<T> = pallet_verifiers::Tickets<T, Ver>;
                pub use pallet_verifiers::{
                    __substrate_call_check, __substrate_event_check, tt_default_parts, tt_error_token,
                };
        }
        };
        let out = verifier_render(
            syn::parse_str(
                r#"
            #[a1]
            #[a2]
            pub struct Ver;"#,
            )
            .unwrap(),
        );
        let out = parse_quote! {
            mod a {
                #out
            }
        };

        assert_eq!(expected, out);
    }

    #[test]
    fn happy_path_with_generic() {
        let expected: syn::ItemMod = parse_quote! {
            mod a {
                pub struct Ver<R>(core::marker::PhantomData<R>);
                pub type Pallet<R> = pallet_verifiers::Pallet<R, Ver<R>>;
                pub type Event<R> = pallet_verifiers::Event<R, Ver<R>>;
                pub type Error<R> = pallet_verifiers::Error<R, Ver<R>>;
                pub type Tickets<R> = pallet_verifiers::Tickets<R, Ver<R>>;
                pub use pallet_verifiers::{
                    __substrate_call_check, __substrate_event_check, tt_default_parts, tt_error_token,
                };
        }
        };
        let out = verifier_render(
            syn::parse_str(
                r#"
            pub struct Ver<R>;"#,
            )
            .unwrap(),
        );
        let out = parse_quote! {
            mod a {
                #out
            }
        };

        assert_eq!(expected, out);
    }

    #[rstest]
    #[case::configurable(BenchmarkingUtils {
            verifier: syn::parse_str("Verifier").unwrap(),
            generic: syn::parse_str("<T>").ok(),
            config: syn::parse_str("crate::Config").ok(),
        }, "Verifier<T>, crate::Config")]
    #[case::no_gen(BenchmarkingUtils {
            verifier: syn::parse_str("Verifier").unwrap(),
            generic: None,
            config: syn::parse_str("crate::Config").ok(),
        }, "Verifier, crate::Config")]
    #[case::no_cfg(BenchmarkingUtils {
            verifier: syn::parse_str("Verifier").unwrap(),
            generic: syn::parse_str("<T>").ok(),
            config: None,
        }, "Verifier<T>")]
    #[case::plain(BenchmarkingUtils {
            verifier: syn::parse_str("Verifier").unwrap(),
            generic: None,
            config: None,
        }, "Verifier")]
    #[case::other(BenchmarkingUtils {
            verifier: syn::parse_str("Other").unwrap(),
            generic: syn::parse_str("<Q>").ok(),
            config: syn::parse_str("yet::another::path").ok(),
        }, "Other<Q>, yet::another::path")]
    fn should_parse_benchmarking_utils(#[case] expected: BenchmarkingUtils, #[case] s: &str) {
        let parsed: BenchmarkingUtils = syn::parse_str(s).unwrap();
        assert_eq!(expected, parsed);
    }
}
