// Copyright 2024-2026, Horizen Labs, Inc.
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#[allow(dead_code)]
fn g1_bn254() -> crate::G1 {
    G1(hex_literal::hex!(
        "
            7ad8259f4d9dd8115874e0e45bcf7cf139cf375c1d4fd5a807a74ff5f299fd02
            53eae2b51a285e56e2f1f8de327a58ba6b75623ee7f3cb99c35009b7ca27d5a3
        "
    )
    .to_vec())
}

#[allow(dead_code)]
fn g1_bls12_381() -> crate::G1 {
    G1(hex_literal::hex!(
        "
            0c5f20234d022490c77c18f9a9ec845811a9faa539361b166ee752ddd1cc71ba
            2a2c37d9b0b1d43b8dd04994d9b8da040d0ddcda16a88407515a1435d0283527
            b0366be975d2e7a2a802e15ea57b9401cd7cd5d1e164364eb4322e9ef2802354
        "
    )
    .to_vec())
}

#[allow(dead_code)]
fn g2_bn254() -> crate::G2 {
    G2(hex_literal::hex!(
        "
            13bd3a516acfdeb2bf6947de578418f0e9b271b91035e0f723782987f2e9b81e
            5e048e6a4fe71e2b98d7af421a62327de748fee1d6159e3863eadac42d124821
            4db9290ab94dd451b100c9e084283691deb3aeb41adfc4984d3dfb31e3975f0e
            b6a8144325ca753721c712800a4a6d2739276c9e61f40f416703b76dece3040e
        "
    )
    .to_vec())
}

#[allow(dead_code)]
fn g2_bls12_381() -> crate::G2 {
    G2(hex_literal::hex!(
        "
            0a3972621539a60bd9e884d234b87b38393cc487db6c5e0a6cfd650697461b3c
            ba186ff64d9d79cec1ebf44f5b05f4750d40e91a8c4002a395e74cf468f6cad4
            4d6584c0e8ecd408f3f61a8e8a63911c0744715cd0d64dbc9ae3e16a3ae3a777
            10e9d672791b82d012bb5e07c416cc5cf6396a3d951274cc1028ad60840642be
            e72afa1d82d50403ec843b11d089b5200c185b8acda70418e972de93cf6e6a0c
            3789125dfef39737c64127782af7308df6ce2b3931b4f6742ec05cf4086d9be3
        "
    )
    .to_vec())
}

#[allow(dead_code)]
fn scalar_bn254() -> crate::Scalar {
    crate::Scalar(
        hex_literal::hex!("a75d1fe3e7eb2f0bd2d88886c679582b85a74ee4a6b77b2d07617b85089da420")
            .to_vec(),
    )
}

#[allow(dead_code)]
fn scalar_bls12_381() -> crate::Scalar {
    crate::Scalar(
        hex_literal::hex!("7a497eeda6d8ed1f38c9324427a251a98a215f13d93c3a138a727b5c27019269")
            .to_vec(),
    )
}

#[allow(dead_code)]
fn proof_bn254() -> crate::Proof {
    Proof{
        a: crate::G1(hex_literal::hex!("976e8832975ade909192a185fb553f7f66d7ff0b58b2ac69e63635632213011f2fad7e996a95ecdbdf251a2526c7c856f894035765fd8c6e6ebde0bd25f9660d").to_vec()),
        b: crate::G2(hex_literal::hex!("5bc1574562bdb6279caa6e0fe6c228aea9b4ed14d7411f080e5a365d86c30c1901a3f19010881db71db8d73af7ffb80303455625bbd34a8e7e3e3a2d2e194324a86a07c4faf9ba2d96c52af5dc265958b2a9d98823461828fa9d0a65d3830f19fee8146afff5565b27514ab317b08647624a49804081542994ebd7b6e6b20d14").to_vec()),
        c: crate::G1(hex_literal::hex!("538bf8dbeaaaff652d564afe07733ea37c07adf360174a700330a1e4f1c6030b589f8f49709d6d626a822ce2bcb020bfde05c2ad11dd1bf7107088af967be4a4").to_vec()),
    }
}

#[allow(dead_code)]
fn proof_bls12_381() -> crate::Proof {
    Proof{
        a: crate::G1(hex_literal::hex!("00d841bdea9286e2a2adc5ae33a7e3f8d09570f67b7431fe8e636409eb9552d9eec1e2fe6aee4aab09330bb2b180fee9069f01acf405008ce386f4c0e3856d3a8b58720e0d56e7b10c73a3bc50c7bb33bf3971bf590084701cce9e9c47a170a1").to_vec()),
        b: crate::G2(hex_literal::hex!("122168ba79db16383f0634a7d8242892aed0fe96138ae2eea55dfae20f24b1b054509d0aee07603c53819278c18444b810ae4da00c1d90213b43f8ea5ba5a6b81f339a51b411d17a2409eaed43b7f9e5a4cd9491246106dfc1a5276d4ba2331f16c948a580906c475dae94005f8e4da6ce2da9e396593b34d4b43658c4fb7dea8cd3420d47153e4060a6cb3ce93ad23d05d53974645dc6a8fc680e2084e3e423d083674e225ffe0de4ee886bfa23b59d4433b982da14998d6b2f8921c14e3d3b").to_vec()),
        c: crate::G1(hex_literal::hex!("016b4288c6fb9b5e4a914971c3d5fbcd51bdac2f3503e7b3cab6f08957882fa6b9de63d46b5d5548607658a83a47336f0151a80ef063dbaa5acd8a4174bf0923ba1eea10c7b698c4163348fabbc0f5562212fdf7175b594bb9ef1b5ccc6387c6").to_vec()),
    }
}

#[allow(dead_code)]
fn verification_key_bn254() -> crate::VerificationKey {
    VerificationKey {
        alpha_g1: crate::G1(hex_literal::hex!("f23ecc6fdae0957b6f9901baa097ec1192a97795a65ef10147345343eb4901183096f9296b8d74135878afea791ad1e053c33460fefb392c61925bb086a3dda5").to_vec()),
        beta_g2: crate::G2(hex_literal::hex!("a17eb8514763a6f1bb824ee9da47097c8529e799f026f544e8e5bdb565f027007313fe210c046dca53e3ecbe79fe12a6dcbadec7e6e370854c49c7768a9088091512d8f91c6c6f2e78b0438ecb511fbd63e0235534d09a0b1643222d841a130cd3b32b17890c6e832aca76c4e28cb31cab8876cf0550881d115edaa9e39da4ad").to_vec()),
        gamma_g2: crate::G2(hex_literal::hex!("c57b18d336c2bfd4693a08c7ad91d82c9bc761f569273f15d0b3d3b341f0e11cdf8728fb8d2375eeba14f081b7ed4cb67f7c10197ea90cbb5012bfb8ee820001485d3dd137e7baf0594b73c7b954fa60f0bf5344299d80349ad3a44e2fef962365a8188bf3e4b3769246ef2fb123c5354e868ed667953f513ff72d042678cd02").to_vec()),
        delta_g2: crate::G2(hex_literal::hex!("66bdd7020e111de2367423d630c6b046a1d23ef4aa4983f4476d87bf705b4328ffe5147b93264bf90e0ed74585f43910b43bf0188d86cbd236ea687d0ff7e22d3f2f288e408e98937c1febcbe43874c5ce465bde5cbd6e9628138c26a656dd222d493505af528ff9e12dcd0bbdefa5c97fb502440cfa097045abef314456050a").to_vec()),
        gamma_abc_g1: vec![
            crate::G1(hex_literal::hex!("2c3c89c560512b2d0b08da1e848f41d6ca559d1b58df315625e95ab0310e3b0f4976fe82316d238aa35b63cdff2f0ef108b9d76c6b45f1eb57dbdfcbe663dc9d").to_vec()),
            crate::G1(hex_literal::hex!("254fe8f76591c219562ede7a5807212abc9427bdb012a9145fe48fe49077711d36bef432122d026d20ed95a2c1e3d7f0c63e6349e112d6786722f40fa6589811").to_vec()),
        ],
    }
}

#[allow(dead_code)]
fn verification_key_bls12_381() -> crate::VerificationKey {
    VerificationKey {
        alpha_g1: crate::G1(hex_literal::hex!("106423f8700d22ac69a94702ebdfe99cfec3f4484173977251444d559f3461606d4105b538a6bb6ee6a63c2f799c3d4f078eaff18d677ca59225dfca97bd947b15facd6497e5594b172b94dcbdad2a4f5054b1c7dc96805f557f4f2bb7176816").to_vec()),
        beta_g2: crate::G2(hex_literal::hex!("1676a945222b351e2743126c5faa42771d49b4f0488d67aedb805783712d2188731151d53be3e228d50fa6480e59ec9b06aa6bf2da38beefbc2a4c1cc63697680b7b70e1a5310b9000bfd89053e89e6aaeecb68a1818aeb47d122fbd6788fef010eaa45e0b817c7d084ee0b489b5b37a643d701af7d612ca06a38c09d168122cc5fa045c3781d919d46b9174efcb172c0ba13e37b8a2da8a35d127a4e9479127f7a782893a3e785a9488cd180a3164982c47f4bd322d677ee58216f426690958").to_vec()),
        gamma_g2: crate::G2(hex_literal::hex!("107a2fc93ccb639ed8da1ec1ac6367e51eed99760ac70f0410279f349420c147289fbf988d9a6313d0c3f6bc2fa471a6061a3b90aecc3569e1203d465c9f47945431ea90810118d87c3025b3ddfbda57a227122bbcaee25f1c5a4465ee60b3c818e7cd599e2ddc64fd0691ceb1d6c3c7bb3c630ad80a4fce5762d2d656a85e802fdb3981e882dc79d4c09d5f998fbdd70ef095eb581348d80e74b2b629bc2a1bfee20d713172192b6b7d1c825340d9ea0c312a7cdcb70f4c2d2e31415a194b05").to_vec()),
        delta_g2: crate::G2(hex_literal::hex!("189a82b999b4352e5b07f9516866eb91d16243cbbfbde9da7201ea6d274d3f9eca91c55a7f4a09f2842f945de0303564046e73685f68ac9884668f757a18bc672b3302818ab47fffcc56996cc2b5da32b7cceec82e7687653abb37cf18fe9b7c06a481f6e6d6d32921256096b6d532294230ab6c15b50587481d077bede3b19c742c19b965f222e04754b6475f83d92511466261021defc063c62d81cebaabda34f35159be2a46c186b63d4d025f9bda99697a8d7a976874d422fc182316c2b8").to_vec()),
        gamma_abc_g1: vec![
            crate::G1(hex_literal::hex!("01afa1c3c8ac2da2aa2c62b9675c781ca8674cf6d399a9750cc0226cdf3555dfab4c159dbda7d391b78c17825008f139016286464c4d8fa7aa8ad6a57cb4722facb907af9ec4d73e8227598f735c7eb1cbcc0fc9e15e51e206146cf99a44a4f1").to_vec()),
            crate::G1(hex_literal::hex!("19d3493cb611308780df4191941cbcea6e29bdba873d8b1e82b130968ff6de85ad367bf240f3ec95960bfa36a3e6bb4101dcec6bbb3fe9e7209b41e1752b59bb37dde57611419ea06d820dba8a107b68606b263c47a05bf6ce86a16d91562ed3").to_vec()),
        ],
    }
}

#[allow(dead_code)]
fn inputs_bn254() -> Vec<crate::Scalar> {
    vec![crate::Scalar(
        hex_literal::hex!("a75d1fe3e7eb2f0bd2d88886c679582b85a74ee4a6b77b2d07617b85089da420")
            .to_vec(),
    )]
}

#[allow(dead_code)]
fn inputs_bls12_381() -> Vec<crate::Scalar> {
    vec![crate::Scalar(
        hex_literal::hex!("7a497eeda6d8ed1f38c9324427a251a98a215f13d93c3a138a727b5c27019269")
            .to_vec(),
    )]
}
