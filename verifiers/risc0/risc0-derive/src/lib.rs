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

#[proc_macro_derive(R0Proof, attributes(unsupported))]
pub fn risc0_proof(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    match valid_data(ast) {
        Ok(data) => impl_risc0_proof(data),
        Err(err) => {
            let err = err.into_iter().map(|err| err.to_compile_error());
            quote! {
                #(#err)*
            }
        }
    }
    .into()
}

#[derive(Debug, PartialEq)]
struct Variant {
    name: Ident,
    supported: bool,
}
#[derive(Debug)]
struct EnumData {
    name: Ident,
    variants: Vec<Variant>,
}

fn valid_data(ast: DeriveInput) -> Result<EnumData, Vec<syn::Error>> {
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
        Ok(EnumData {
            name,
            variants: valid_variants.into_iter().map(Result::unwrap).collect(),
        })
    }
}

fn valid_variant(variant: &syn::Variant) -> Result<Variant, syn::Error> {
    let supported = !variant
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("unsupported"));
    match &variant.fields {
        syn::Fields::Unnamed(inner) if inner.unnamed.len() == 1 => (),
        _ => {
            return Err(syn::Error::new_spanned(
                variant,
                "expected an unnamed tuple with just one element",
            ))
        }
    };
    Ok(Variant {
        name: variant.ident.clone(),
        supported,
    })
}

fn impl_risc0_proof(data: EnumData) -> proc_macro2::TokenStream {
    let proof = &data.name;
    let variants = data.variants.as_slice();
    let local_variants = variants.iter().map(|v| &v.name).collect::<Vec<_>>();
    let r0_proof = format_ident!("R0{proof}");
    let r0_variants = variants
        .iter()
        .filter(|v| v.supported)
        .map(|v| &v.name)
        .collect::<Vec<_>>();

    let try_from_impls = variants
        .iter()
        .map(|v| {
            let variant = &v.name;
            if v.supported {
                quote! {
                    Ok(Self::#variant(risc0_proof))
                }
            } else {
                quote! {
                    Err(ConvertProofError::UnsupportedVersion)
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        enum #r0_proof {
            #(#r0_variants(risc0_verifier::Proof)),*
        }

        impl #r0_proof {
            fn proof(&self) -> &risc0_verifier::Proof {
                match self {
                    #(#r0_proof::#r0_variants(p) => p),*
                }
            }

            fn take_proof(self) -> risc0_verifier::Proof {
                match self {
                    #(#r0_proof::#r0_variants(p) => p),*
                }
            }
        }

        #[derive(Debug)]
        enum ConvertProofError {
            UnsupportedVersion,
            DeserializeError,
        }

        impl From<ConvertProofError> for hp_verifiers::VerifyError {
            fn from(err: ConvertProofError) -> Self {
                match err {
                    ConvertProofError::UnsupportedVersion => hp_verifiers::VerifyError::UnsupportedVersion,
                    ConvertProofError::DeserializeError => hp_verifiers::VerifyError::InvalidProofData,
                }
            }
        }

        impl TryFrom<&#proof> for #r0_proof {
            type Error = ConvertProofError;

            fn try_from(proof: &#proof) -> Result<Self, Self::Error> {
                let risc0_proof = ciborium::from_reader(proof.bytes()).map_err(|_| ConvertProofError::DeserializeError)?;
                match proof {
                    #(#proof::#local_variants(_) => #try_from_impls),*
                }
            }
        }

        impl #proof {
            fn len(&self) -> usize {
                match self {
                    #(#proof::#local_variants(proof_bytes) => proof_bytes.len()),*
                }
            }

            fn bytes(&self) -> &[u8] {
                match self {
                    #(#proof::#local_variants(proof_bytes) => proof_bytes.as_slice()),*
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
                    #[unsupported]
                    First(some),
                    FirstSupported(some),
                    #[unsupported]
                    Second(other),
                    More(many),
                  }"#,
            )
            .unwrap();
            let data = valid_data(data).unwrap();

            assert_eq!("MyEnum", data.name.to_string());
            assert_eq!(
                data.variants,
                vec![
                    ("First", false),
                    ("FirstSupported", true),
                    ("Second", false),
                    ("More", true)
                ]
                .into_iter()
                .map(|(id, supported)| Variant {
                    name: format_ident!("{id}"),
                    supported,
                })
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
        let variants = [
            ("V1_0", false),
            ("V1_1", false),
            ("V1_2", true),
            ("V2_0", true),
        ]
        .into_iter()
        .map(|(n, supported)| Variant {
            name: format_ident!("{n}"),
            supported,
        })
        .collect::<Vec<_>>();

        let out = impl_risc0_proof(EnumData { name, variants });

        let expected = quote! {
            enum R0Proof {
                V1_2(risc0_verifier::Proof),
                V2_0(risc0_verifier::Proof)
            }

            impl R0Proof {
                fn proof(&self) -> &risc0_verifier::Proof {
                    match self {
                        R0Proof::V1_2(p) => p,
                        R0Proof::V2_0(p) => p
                    }
                }

                fn take_proof(self) -> risc0_verifier::Proof {
                    match self {
                        R0Proof::V1_2(p) => p,
                        R0Proof::V2_0(p) => p
                    }
                }
            }

            #[derive(Debug)]
            enum ConvertProofError {
                UnsupportedVersion,
                DeserializeError,
            }

            impl From<ConvertProofError> for hp_verifiers::VerifyError {
                fn from(err: ConvertProofError) -> Self {
                    match err {
                        ConvertProofError::UnsupportedVersion => hp_verifiers::VerifyError::UnsupportedVersion,
                        ConvertProofError::DeserializeError => hp_verifiers::VerifyError::InvalidProofData,
                    }
                }
            }

            impl TryFrom<&Proof> for R0Proof {
                type Error = ConvertProofError;

                fn try_from(proof: &Proof) -> Result<Self, Self::Error> {
                    let risc0_proof = ciborium::from_reader(proof.bytes()).map_err(|_| ConvertProofError::DeserializeError)?;
                    match proof {
                        Proof::V1_0(_) => Err(ConvertProofError::UnsupportedVersion),
                        Proof::V1_1(_) => Err(ConvertProofError::UnsupportedVersion),
                        Proof::V1_2(_) => Ok(Self::V1_2(risc0_proof)),
                        Proof::V2_0(_) => Ok(Self::V2_0(risc0_proof))
                    }
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

        assert_eq!(expected.to_string(), out.to_string());
    }
}
