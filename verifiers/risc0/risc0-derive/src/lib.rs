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

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::DeriveInput;

#[proc_macro_derive(R0Proof)]
pub fn risc0_proof(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    match valid_data(ast) {
        Ok((name, valid_variants)) => impl_risc0_proof(&name, valid_variants.as_slice()),
        Err(err) => {
            let err = err.into_iter().map(|err| err.to_compile_error());
            quote! {
                #(#err)*
            }
        }
    }
    .into()
}

fn valid_data(ast: DeriveInput) -> Result<(Ident, Vec<Ident>), Vec<syn::Error>> {
    let name = ast.ident.clone();
    let variants = match &ast.data {
        syn::Data::Enum(data) => &data.variants,
        _ => return Err(vec![syn::Error::new_spanned(ast.ident, "expected an enum")]),
    };
    let (valid_variants, errors): (Vec<_>, Vec<_>) = variants
        .into_iter()
        .map(valid_variant)
        .partition(Result::is_ok);

    if !errors.is_empty() {
        Err(errors.into_iter().map(Result::unwrap_err).collect())
    } else {
        Ok((
            name,
            valid_variants.into_iter().map(Result::unwrap).collect(),
        ))
    }
}

fn valid_variant(variant: &syn::Variant) -> Result<Ident, syn::Error> {
    match &variant.fields {
        syn::Fields::Unnamed(inner) if inner.unnamed.len() == 1 => (),
        _ => {
            return Err(syn::Error::new_spanned(
                variant,
                "expected an unnamed tuple with just one element",
            ))
        }
    };
    Ok(variant.ident.clone())
}

fn impl_risc0_proof(proof: &Ident, variants: &[Ident]) -> proc_macro2::TokenStream {
    let r0_proof = format_ident!("R0{proof}");
    quote! {
        enum #r0_proof {
            #(#variants(risc0_verifier::Proof)),*
        }

        impl #r0_proof {
            fn proof(&self) -> &risc0_verifier::Proof {
                match self {
                    #(#r0_proof::#variants(p) => p),*
                }
            }

            fn take_proof(self) -> risc0_verifier::Proof {
                match self {
                    #(#r0_proof::#variants(p) => p),*
                }
            }
        }

        impl TryFrom<&#proof> for #r0_proof {
            type Error = ();

            fn try_from(proof: &#proof) -> Result<Self, Self::Error> {
                let risc0_proof = ciborium::from_reader(proof.bytes()).map_err(|_| ())?;
                Ok(match proof {
                    #(#proof::#variants(_) => Self::#variants(risc0_proof)),*
                })
            }
        }

        impl #proof {
            fn len(&self) -> usize {
                match self {
                    #(#proof::#variants(proof_bytes) => proof_bytes.len()),*
                }
            }

            fn bytes(&self) -> &[u8] {
                match self {
                    #(#proof::#variants(proof_bytes) => proof_bytes.as_slice()),*
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use quote::{format_ident, quote};

    mod valid_data_should {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn accept_enum() {
            let data = syn::parse_str(
                r#"enum MyEnum{
                    First(some),
                    Second(other),
                    More(many),
                  }"#,
            )
            .unwrap();
            let (name, valid_variants) = valid_data(data).unwrap();

            assert_eq!("MyEnum", name.to_string());
            assert_eq!(
                valid_variants,
                vec!["First", "Second", "More"]
                    .into_iter()
                    .map(|id| format_ident!("{id}"))
                    .collect::<Vec<_>>()
            );
        }

        #[test]
        fn reject_struct() {
            let data = syn::parse_str(r#"struct MyStruct;"#).unwrap();
            valid_data(data).unwrap_err();
        }

        #[test]
        fn reject_union() {
            let data = syn::parse_str(
                r#"union MyUnion {
                        f1: u32,
                        f2: f32,
                    }"#,
            )
            .unwrap();
            valid_data(data).unwrap_err();
        }

        #[test]
        fn reject_enum_with_wrong_variants() {
            let data = syn::parse_str(
                r#"enum MyEnum{
                    Valid(some),
                    First,
                    Second(more, than, one),
                    Named{ a: u32 },
                  }"#,
            )
            .unwrap();
            assert_eq!(3, valid_data(data).unwrap_err().len());
        }
    }

    #[test]
    fn render() {
        let name = syn::parse_str("Proof").unwrap();
        let variants_names = ["V1_0", "V1_1", "V1_2", "V2_0"]
            .into_iter()
            .map(|n| format_ident!("{}", n))
            .collect::<Vec<_>>();

        let out = impl_risc0_proof(&name, variants_names.as_slice());

        let expected = quote! {
            enum R0Proof {
                V1_0(risc0_verifier::Proof),
                V1_1(risc0_verifier::Proof),
                V1_2(risc0_verifier::Proof),
                V2_0(risc0_verifier::Proof)
            }

            impl R0Proof {
                fn proof(&self) -> &risc0_verifier::Proof {
                    match self {
                        R0Proof::V1_0(p) => p,
                        R0Proof::V1_1(p) => p,
                        R0Proof::V1_2(p) => p,
                        R0Proof::V2_0(p) => p
                    }
                }

                fn take_proof(self) -> risc0_verifier::Proof {
                    match self {
                        R0Proof::V1_0(p) => p,
                        R0Proof::V1_1(p) => p,
                        R0Proof::V1_2(p) => p,
                        R0Proof::V2_0(p) => p
                    }
                }
            }

            impl TryFrom<&Proof> for R0Proof {
                type Error = ();

                fn try_from(proof: &Proof) -> Result<Self, Self::Error> {
                    let risc0_proof = ciborium::from_reader(proof.bytes()).map_err(|_| ())?;
                    Ok(match proof {
                        Proof::V1_0(_) => Self::V1_0(risc0_proof),
                        Proof::V1_1(_) => Self::V1_1(risc0_proof),
                        Proof::V1_2(_) => Self::V1_2(risc0_proof),
                        Proof::V2_0(_) => Self::V2_0(risc0_proof)
                    })
                }
            }

            impl Proof {
                fn len(&self) -> usize {
                    match self {
                        Proof::V1_0(proof_bytes) => proof_bytes.len(),
                        Proof::V1_1(proof_bytes) => proof_bytes.len(),
                        Proof::V1_2(proof_bytes) => proof_bytes.len(),
                        Proof::V2_0(proof_bytes) => proof_bytes.len()
                    }
                }

                fn bytes(&self) -> &[u8] {
                    match self {
                        Proof::V1_0(proof_bytes) => proof_bytes.as_slice(),
                        Proof::V1_1(proof_bytes) => proof_bytes.as_slice(),
                        Proof::V1_2(proof_bytes) => proof_bytes.as_slice(),
                        Proof::V2_0(proof_bytes) => proof_bytes.as_slice()
                    }
                }
            }
        };

        assert_eq!(format!("{:#?}", expected), format!("{:#?}", out));
    }
}
