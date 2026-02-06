// Copyright 2025, Horizen Labs, Inc.

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

#![cfg(any(test, feature = "runtime-benchmarks"))]

pub struct TestData<T: crate::Config> {
    pub vk: crate::Vk,
    pub proof: crate::RawProof,
    pub pubs: crate::Pubs,
}

pub fn get_parameterized_test_data<T: crate::Config>(
    log_circuit_size: u64,
    proof_type: ProofType,
) -> TestData<T> {
    let data = match proof_type {
        ProofType::ZK => DATA_ZK,
        ProofType::Plain => DATA_PLAIN,
    };
    let benchmark_idx = (log_circuit_size - MIN_BENCHMARKED_LOG_CIRCUIT_SIZE) as usize;

    TestData {
        vk: crate::VkWithConfig::new(config, data[benchmark_idx].vk.to_vec()),
        proof: crate::Proof::new(data[benchmark_idx].proof.to_vec()),
        pubs: data[benchmark_idx].pubs.to_vec(),
    }
}

struct Data {
    vk: &'static [u8],
    proof: &'static [u8],
    pubs: &'static [u8],
}

static DATA_PLAIN: &[Data] = &[
    Data {
        vk: include_bytes!("resources/plain/log_7/vk"),
        proof: include_bytes!("resources/plain/log_7/proof"),
        pubs: include_bytes!("resources/plain/log_7/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_8/vk"),
        proof: include_bytes!("resources/plain/log_8/proof"),
        pubs: include_bytes!("resources/plain/log_8/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_9/vk"),
        proof: include_bytes!("resources/plain/log_9/proof"),
        pubs: include_bytes!("resources/plain/log_9/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_10/vk"),
        proof: include_bytes!("resources/plain/log_10/proof"),
        pubs: include_bytes!("resources/plain/log_10/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_11/vk"),
        proof: include_bytes!("resources/plain/log_11/proof"),
        pubs: include_bytes!("resources/plain/log_11/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_12/vk"),
        proof: include_bytes!("resources/plain/log_12/proof"),
        pubs: include_bytes!("resources/plain/log_12/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_13/vk"),
        proof: include_bytes!("resources/plain/log_13/proof"),
        pubs: include_bytes!("resources/plain/log_13/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_14/vk"),
        proof: include_bytes!("resources/plain/log_14/proof"),
        pubs: include_bytes!("resources/plain/log_14/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_15/vk"),
        proof: include_bytes!("resources/plain/log_15/proof"),
        pubs: include_bytes!("resources/plain/log_15/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_16/vk"),
        proof: include_bytes!("resources/plain/log_16/proof"),
        pubs: include_bytes!("resources/plain/log_16/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_17/vk"),
        proof: include_bytes!("resources/plain/log_17/proof"),
        pubs: include_bytes!("resources/plain/log_17/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_18/vk"),
        proof: include_bytes!("resources/plain/log_18/proof"),
        pubs: include_bytes!("resources/plain/log_18/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_19/vk"),
        proof: include_bytes!("resources/plain/log_19/proof"),
        pubs: include_bytes!("resources/plain/log_19/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_20/vk"),
        proof: include_bytes!("resources/plain/log_20/proof"),
        pubs: include_bytes!("resources/plain/log_20/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_21/vk"),
        proof: include_bytes!("resources/plain/log_21/proof"),
        pubs: include_bytes!("resources/plain/log_21/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_22/vk"),
        proof: include_bytes!("resources/plain/log_22/proof"),
        pubs: include_bytes!("resources/plain/log_22/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_23/vk"),
        proof: include_bytes!("resources/plain/log_23/proof"),
        pubs: include_bytes!("resources/plain/log_23/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_24/vk"),
        proof: include_bytes!("resources/plain/log_24/proof"),
        pubs: include_bytes!("resources/plain/log_24/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_25/vk"),
        proof: include_bytes!("resources/plain/log_25/proof"),
        pubs: include_bytes!("resources/plain/log_25/pubs"),
    },
    Data {
        vk: include_bytes!("resources/plain/log_26/vk"),
        proof: include_bytes!("resources/plain/log_26/proof"),
        pubs: include_bytes!("resources/plain/log_26/pubs"),
    },
];

static DATA_ZK: &[Data] = &[
    Data {
        vk: include_bytes!("resources/zk/log_7/vk"),
        proof: include_bytes!("resources/zk/log_7/proof"),
        pubs: include_bytes!("resources/zk/log_7/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_8/vk"),
        proof: include_bytes!("resources/zk/log_8/proof"),
        pubs: include_bytes!("resources/zk/log_8/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_9/vk"),
        proof: include_bytes!("resources/zk/log_9/proof"),
        pubs: include_bytes!("resources/zk/log_9/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_10/vk"),
        proof: include_bytes!("resources/zk/log_10/proof"),
        pubs: include_bytes!("resources/zk/log_10/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_11/vk"),
        proof: include_bytes!("resources/zk/log_11/proof"),
        pubs: include_bytes!("resources/zk/log_11/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_12/vk"),
        proof: include_bytes!("resources/zk/log_12/proof"),
        pubs: include_bytes!("resources/zk/log_12/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_13/vk"),
        proof: include_bytes!("resources/zk/log_13/proof"),
        pubs: include_bytes!("resources/zk/log_13/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_14/vk"),
        proof: include_bytes!("resources/zk/log_14/proof"),
        pubs: include_bytes!("resources/zk/log_14/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_15/vk"),
        proof: include_bytes!("resources/zk/log_15/proof"),
        pubs: include_bytes!("resources/zk/log_15/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_16/vk"),
        proof: include_bytes!("resources/zk/log_16/proof"),
        pubs: include_bytes!("resources/zk/log_16/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_17/vk"),
        proof: include_bytes!("resources/zk/log_17/proof"),
        pubs: include_bytes!("resources/zk/log_17/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_18/vk"),
        proof: include_bytes!("resources/zk/log_18/proof"),
        pubs: include_bytes!("resources/zk/log_18/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_19/vk"),
        proof: include_bytes!("resources/zk/log_19/proof"),
        pubs: include_bytes!("resources/zk/log_19/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_20/vk"),
        proof: include_bytes!("resources/zk/log_20/proof"),
        pubs: include_bytes!("resources/zk/log_20/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_21/vk"),
        proof: include_bytes!("resources/zk/log_21/proof"),
        pubs: include_bytes!("resources/zk/log_21/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_22/vk"),
        proof: include_bytes!("resources/zk/log_22/proof"),
        pubs: include_bytes!("resources/zk/log_22/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_23/vk"),
        proof: include_bytes!("resources/zk/log_23/proof"),
        pubs: include_bytes!("resources/zk/log_23/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_24/vk"),
        proof: include_bytes!("resources/zk/log_24/proof"),
        pubs: include_bytes!("resources/zk/log_24/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_25/vk"),
        proof: include_bytes!("resources/zk/log_25/proof"),
        pubs: include_bytes!("resources/zk/log_25/pubs"),
    },
    Data {
        vk: include_bytes!("resources/zk/log_26/vk"),
        proof: include_bytes!("resources/zk/log_26/proof"),
        pubs: include_bytes!("resources/zk/log_26/pubs"),
    },
];

const VALID_VK: [u8; VK_SIZE] = *include_bytes!("resources/zk/log_26/vk");
const VALID_ZK_PROOF: &[u8] = include_bytes!("resources/zk/log_26/proof");
const VALID_PLAIN_PROOF: &[u8] = include_bytes!("resources/plain/log_26/proof");

#[allow(dead_code)]
fn valid_public_input() -> crate::Pubs {
    include_bytes!("resources/zk/log_26/pubs")
    .chunks_exact(32)
    .map(|c| c.try_into().unwrap())
    .collect();
}

// #[allow(dead_code)]
// static VALID_ZK_PROOF: [u8; 7488] = hex_literal::hex!(
//     "
//         0000000000000000000000000000000000000000000000042ab5d6d1986846cf
//         00000000000000000000000000000000000000000000000b75c020998797da78
//         0000000000000000000000000000000000000000000000005a107acb64952eca
//         000000000000000000000000000000000000000000000000000031e97a575e9d
//         00000000000000000000000000000000000000000000000b5666547acf8bd5a4
//         00000000000000000000000000000000000000000000000c410db10a01750aeb
//         00000000000000000000000000000000000000000000000d722669117f9758a4
//         000000000000000000000000000000000000000000000000000178cbf4206471
//         000000000000000000000000000000000000000000000000e91b8a11e7842c38
//         000000000000000000000000000000000000000000000007fd51009034b3357f
//         000000000000000000000000000000000000000000000009889939f81e9c7402
//         0000000000000000000000000000000000000000000000000000f94656a2ca48
//         000000000000000000000000000000000000000000000006fb128b46c1ddb67f
//         0000000000000000000000000000000000000000000000093fe27776f50224bd
//         000000000000000000000000000000000000000000000004a0c80c0da527a081
//         0000000000000000000000000000000000000000000000000001b52c2020d746
//         1d0ee2a4f7f1c8810836403f964498cb3523ddc475ee65251259585424ae43de
//         1d09ee0bf888eb7ad55e9a1292d1e478c1eb2772b254811d07157131f1be419c
//         1d29fdb9a8216c396f67dc6ccd7da9c9c0e0d9bc28c307e45f85080a67cfc681
//         09dd1dc888d6ddbc9a8273cd5fe430d6a87403403d3240dd1ee3e3e51679f0c4
//         23e3b77ecebe5f71fb25ec768a5d3dea1cb50010544329a1fabe8c73edd3426e
//         1f6eeadfae9c493eb08cd8ee703c621f721bc1ec0d3d021d6972682b593f5a17
//         0002892445d549b3059df9a9e7faf3e73a1a9764006223f4b81a3d48686b2d98
//         2763b11e5d6e93594f0e46e98308ed888a6d64420908cd9ee7d5f258a03439f3
//         0ad90b3af9737ccff5398f9b6e2435e6e1348fdf9f619f95216e6988ec95f805
//         2aa958f8f2ef0ab1539abf56445255699cb90a9791a7c820d3af8e43f0bc3cb2
//         179138fde548b176d262695b9fbcfdea40ac7b7483e567b18f8b5ce131a1e103
//         0b54a869b3532cff3154e56a7d9a6092b18e04ab5c1e01273f555352fe3a834c
//         1c033243d10d1de9258a1cd1c6bae8efdf05433d12a475fe280b2a2ea034927d
//         1fda658ee534603823bca01d30f0f45e2919a5b9b18fb5abac3262bd8bb406e1
//         0f0058f17519f0ea6138d386dbca3b0465fc53c3693c6b4f1771bc5aa231466d
//         193cc1106683c44f6732662157e69406b8ec0806d1c7d34f18b679c1d88c6f28
//         1838a12f473240ec3466016e5a51b23aae8bdbcb5706bdbcbc648b340f10fc98
//         0669a3e52adf72f4c3ddaf757523a18412fcc16311311d53c69f631769237344
//         04dce303c5345f1259375ad00521ea9675baeac0a887dcbda7b7809f9062751a
//         1b775879c81cac52020d885fd8d4e6341f99f6637d527b280881fd6ddc548340
//         241f6b50056092b93841bdb14b66ee0dc2705f80e8593d577491511a0d889d0a
//         133eb45b2992582cf5b669932dcfa6b527a2bfaf5b2c0f8a9916dfbfedcec100
//         1218b5bbc1006447381ea470851fafbe97b56b68d8f9c48dc06d3699451a9dd8
//         1e2b668110e23b1e86608f1a55a687c892341312e263f9b071065dd9b77b5cc0
//         0abf1fbdde7adf4f1776c867efd803dd89a4a2cf1ad41fe3e67a11dcd0640108
//         2dcc62205732132b9a18d539b1e79840fe0704b973bf618da8c64b79e3ab3946
//         1566df9f571d90ad7459ede52ae661795ea83ca79d93087fff581fc50959f9de
//         0ef335df47afb985e0c183a07b7d71a69e475ded1c0f9c7b92aea677331dd507
//         18d3d19ddad05d21568e5d7e06cea5bef41f0b1432db92262f7c94bceecd1f99
//         068ee30f4dcf6059fa0524801e0327fee03648c8c1173efb837650bee8535424
//         105a5c276a09bd539816a1e895cd79f031fa1b2a884a6f1e1d4dba35eb1f6043
//         07591e7dddd116e0859ee53304a257c044caf8fd402173575a48dd988a2f8332
//         1772b44173cd899f10301fad32a6e4b281a56c553b49904f3665b6e0b3019ca6
//         26db0ccd45eca51877625fd911531a51bf18f5f0c20d2153b6fd7ac61b55f39f
//         14eb3ce9cb171204afe7a0d56342cdce25f1b472050b0834d3a0649478e0ca39
//         216a0ae92d3dfc658d150ebcd1f296ef95657c59e1aa29e45ab87b3de77ebb5b
//         2205e4a69aa28d41f2f9fca6a51f105357308431076d580b3d21758b41257d44
//         056ca2e7d05b0e3dceb52a686a2dff10cb11a604cba6cd9430d84f7421cae9d1
//         037bfed6417ff8bb169b519f8cf327e9bba5a505bfcb4a7d470c1fbcf325c93d
//         1c688d427e76095998716d8bbccf112c6191da674bba269b0d44bbad934f6ca2
//         2fe98b8d85d435611276ecbdfe451e86fcaccc2a05413ee0bd9fd604c4535535
//         1541c8039db6c6d1bfcbc12236dfa875dbf042ea50b4394a909ef7db19d45aeb
//         263ad6749ca9838e67d4757736bb0f3f97783e7e7c55a69f8139266feef6582b
//         0fae0bf4f8ffb240f8962b39d0eb7adf36a8c8f8ccedb89affd2f199fc751415
//         1fe33029003ff628368d64c49820b4fd9a99fda82f4677798cafc87a0aa9aff8
//         218e0d3a61fa755f501cdce5dcaf672fb50fe2d74e15ed15dafee357d61a4852
//         000cc120929495f5e80cfd29fc6268afece30bcf395fab582665d7570430017d
//         2ec4f1a485c5d9632d8064665c9c409517815a65aa15a06bde022fa7b1776074
//         2832da02891d13e5230789f843e098d6cceff15c6a5e690679e494640a927163
//         0951a0615164613e10b8da239494725a6790e200f2010942decdf3731e283e93
//         216f1820442774ca55a1c88c724cf382f8135b7f82451496ca4f5bbc6f07e125
//         23c3ddd8541b0ed84d4a5c960aff68fb43320d0ca545392a7cb2b963f0eb58e8
//         20e2d8579e2499b1fb6997a6d5868a1f537f7f6bfcfcf65cbc100d0ee4b3e99c
//         20be30415ce0ca48ea0fb057b1f66e2bde8a80b7b2054b86629c530d9c227485
//         2f9670ede7e2e045912784af83e5f14a55b1da7b4849284dee287489cd79a842
//         11b93c25e22ed6d44cf8fdc8db29e17b602b4a81d1f0e9fdd2513f329ab635bf
//         0c9ad5c4f1fd836eaad89f59c904eb09ceccdf3b02ef7e09adc578ce7b01146d
//         1cba228a7d1da155ef346c4268826aba48dbfc2cff93ad79de444ee57995567d
//         272e93bff7a400d178ea64f07ecabe977352ce5d4d262f7f5f4c1827f7ef3112
//         01db64dd0a4bfb06006b348909118ef866216c27f0f389f72379bcf5d37738b3
//         142b04011feb6e1df8966195835f60cadb6f83b88aad1fd9a7eead902a4eb000
//         288b55a750c4a62a87bd0572e6f2b06f494cac505429cc794b7ab6ec113d783c
//         1c9eae9fe904773fa2b5bd5b0c35d63369444a4e7093418f38ea2af31612f444
//         24df841bcbf8406e7a4a00a14f269c6fca964070307a97bab25ec0998898fd9c
//         2de3f6081b3da9be228fbad17f1febc8f8c7a2abf0aaaaccdca95686f226b38c
//         257cfb5a55c0fb320ae7ad32e4ac5400379a72ec84c6d6c36e087b4b269c5928
//         24cf9281d0b454009ed821099234b8b6f4cc401658a0f2e0277ad6c0e2992bbf
//         1507a364f5bed1ca792fe71ae443c30b49b44dadd52fd3f7ebdb5856619db052
//         2cdfe123c2062b7d16fd8703192fb8c0b0cd9f979dd1018dbffb4905fca6ad73
//         2f5d89c2abec729248f410098051f8d2ff77cd55175afdf3229aa2857fb6efca
//         2c0476dc8978e582d7256fa70153ffccb1761dfbf3b17a665766d2e1d7c35e29
//         26bdc6dc93e7950f4681d8602c31ec31ae61d4a1bebb1e20a50fa08b803ce531
//         21256ee3157766f9b4dde3d3585ffd7ca7ba974cf91eb3d00d4f6b604049aa4f
//         1eb98a0e747e2e55111e59e0498b20ab4bae2857ad32cd0b1ffa55d8f301421d
//         2bf814c581640bf6ad16ab3f7d9142b8a4d555ba28db35a3e3f6bb2257ec087e
//         1ddb5de6c6ab969f3df3cbd669c80fe0ff02764fe201ad98244dbc59ab4d275b
//         1d4bee22b6c747208e76e3f3823b024e7005f02df779866ad15ca69c81045982
//         01546c46ed1a175d70485cb2f69252332544037d21a8447fa66082b9f217c519
//         06d6286be8b128612bae3b61b373a4552cde1b48ca0556b82164953208e8cd76
//         2465f4bb2d5fa0435278efd60ecbfadcd51ed0b3c1901d91647de369a68e8324
//         2fa4d09d0aa9b4ba0fd2684f179c05acf136928c534b73b86238d88e84b3a186
//         0d8da9cfd30c85fe5435a34ba6207842aa4a65fc69a17e61b4008540b8ec9cf1
//         1d04af20f16c19165ed5957568c03e590687c481a83aa15df709cc6d4b3972b4
//         10b9cb4c033d83954ef6e3ef96a8ea63a2f41aa7236c020a66d513b55181ee2d
//         1306f04a496c118fce07f470940e0c8119f3e4ecde55f5187ea6a0e2b812a7f3
//         1382eaa71c5b59df787b74a4b1900446ff79b4e424a2fe62af7c605447b9ba7f
//         16484baeac4842a085e7e529d59c7338292f7d7929eb7dc9f494d5c51f3343ce
//         0d19da4225f4c94c34d4c036373e785127eac67213865215178dde2c720f404d
//         24f1a02da2ba1e4dafe8e2574ac891318f6386994b035a26a7932d8fae154364
//         178129f7d0c8a4d223f5413d8367d707e674e45866d1b356ba5cc1cede65cda1
//         289384cb91bda4b9e75d40f1026b5084eec5dbc6a908583f3e22568b29ed4e65
//         2fcac8aefae4d596c79ba898ec602ae30ee22693404503f1d6d01b8f0b4fde29
//         226a14df30849e48e34e799164b76f423af9dc0a15ab520f4ed10a9b6b0369ab
//         13a70ad00f9e31cdf84ebd7119982c31dbecc983edf4dfeb172833376c842887
//         087f1cb38f0ad3459394740bc1dc4b92a3921f5ddf5763d80f18ec2aeb47be5b
//         113fdbfe896596353ddf65c923244a9d6efea988b08d0da99f66dc4239d1d49c
//         163eb3b11912ca843124f00e13ad48177943bc52f738e8cd761c25cd4e6c60f8
//         02284fb8f3588243b26ae4506a93b6d7bbce1a4f11c8c2150b5a2846fb37567b
//         1f8e197e8223ba6c03d84446b304d3c4cf55d2fc0c47cbd58f86f9b417428c90
//         1502cdda071f3ca83e46b471d0bc0a2ab5c29311b7b0097939a7468f4fb1af12
//         03b018837447d26e4756590463730b0f940a2754c00614e9eee856f96e1c4275
//         01a2be61fe902e1b5922f046eec44e21337961e17caa90f2634fa415e5e72161
//         1e3f55a13fb8db8cda822bee138c32f1363d9164d7e3ca1751f545c4908c54f4
//         26d1797b4c6e0084ff444eeb1054e85230d56a46c312e9f320df6b7dc5aac5a3
//         21d4b97c5f193a64446c53afabc98d4d6d6149306b784c779e54dc508a778e0d
//         116e483a4d21df580450648369da843117aa5c3fe06b25505f098a928699247c
//         1cde4075569d7ad69b284e5c377d105dc2e422c7a30c16091f563ff1bdd23270
//         11869c8d9f8c01566af8378671e6c0bc19b1c04c02059d1ee94a9105910da12a
//         0f8d6e7cac5e9d71b62018897a564d3f6fc2b3b035461e7536254c866a742436
//         0e3908d39ad63f9d0617f2e38b724d40e32edecc9764fdaa197ac3323f2b96e9
//         2cdad90db5d6514562430e089af026de9b8ad0dfc784b4259c04e3000053c5aa
//         0cfc6bb042fb9da60341603e30cdf14b9ffdaf6b1c33041874fc4ab499b54b38
//         1efa2c917198bb3a71ac60177ac8f03f4c404ee83754891f8cdadd87313d3385
//         06b6b861a854811c255422ed795eb891ca16857e902740837211b0248abc32eb
//         04706a13c24622ac5f202f24d8dedf677c5068bf4b1bb2c640ec7f144f1d9d8b
//         20fca5dac41b677e6d0c975dcdaeab254bfb11255f574e0ec93d093ceef0f9c8
//         1a0cddc9e4fe432aa0dc9ee75a2e0c5ba812b819e49c40a4a19d644335f91058
//         199fe769db5ebcd88afd55a4465e530f7c9dc79d3d507019d2acbd88b2a50d1d
//         280752d4ff9776657537bfc82ab4cc18b7ae47baefb2a200ae1bac670a5dbd6d
//         0e4739bb2ec5ef33482ea2bdb810b86ef8a69e27d5b934fbc4c63379befae0b1
//         082631792cd0612a398adb0627bdcdf930f3b6eff9198831c8da589fe7f8c773
//         02422c93298da0e3f9cc951586bdc860c9da9395f342b421e91c900dcce37071
//         077f1ffad02fabe725b0e8bbec674d100c554bd8c03d362d61cd0063d13c6590
//         07271dbd059d7b88756810237724b59dcc44c9ca8e5196cc4ad4c7026aa556a2
//         17577f581cc75809df04a3eae7164be9b90d9cf3795385ec1befd4a30ff57ec7
//         021d01a4e985bc623521a7b5c2b35fb9d5a92e5168065f85426a99e7f124b972
//         1f45dc9cafa11a6091729f6ba6071288895ca36ca6089a0e82c084f78e388af2
//         09a075626d437173e8601a29181dd65bf627297f3b3f67fc52eaea060136625c
//         179b1fa510b8f50c97731bee62df0449cf90e50abbb01ed1942726f722c0df0a
//         221bcd8a269a6a5cc0be672df150168637982ebd17ca70db66a4c516d6dd3fda
//         09c741a0ded88ea4dcf1f93a43f9105de06a388d64a175e9a0162f7d8c5f3b3d
//         0b0d4403ae1d7eb17511354d03bf27a2da306a62d4835be1dde958942407eea5
//         1b89cc4afaf30194ffbab7f27c24ff3cec519fc845dd55b96dd7c152bdf23740
//         0175b6cee8c1fd62758d371fb4e8c90c1256cca2bbd32ac85708c235ff793c06
//         1357691445b8493c435b6a2058acfe861900a9e8a94f6bf2697b32d590ef9d74
//         134a5982cf698dda5a0d739d2e2edd08fe53ebbd9b00c1e45240482e074759e1
//         297a1c8e928d6644972a3ff7bae2cc8a54dd3744468a09ada88f4b1d295f318e
//         10c56e612cac25ce0eaaba23ccfc37004fea2cc7f565dbeeff912900407f419a
//         2cfe5f856a09318db6a4fbbeaf1bce94d49cee2afeaccf1115f000d453310714
//         1c865bec3f19f12b76e0595bf9eb3b3ac6b70cc67ec8ffa8cb01fff92419538c
//         202c5609a6799283e527e0ea31a792d77a43e0e9509309217da5376097c92cac
//         25b14415444c152e705bf7c317d23b87a4a583fe47a705f313172574e1a5c9dd
//         093e2f4ab7a911788de538f6e3eef6ccd1095fdeaffaf862c760da2abbb36af4
//         135767b06f300a8ec3c694e2b5036253821b8fc5824b7235e60543925182877a
//         2754569faef75fc909eb8b7142067f8ea0e6e264f491c85bc44a7bd6383c82a2
//         06329323c7e527ae8f6e788e081a2e944d896fddf2cc536dd3cb7d0138f4a7e4
//         17f728f09872980a4b625617bb106fbbd8cc2fdc6b6a47be64e9f98414e87610
//         086b161e1536db414b3c9a05fb8ded13fa4ee2f4f1f4f3fc2e85eb0acd3d2f8e
//         0035d5057b670d7efaf84abd470fa308c1841a976dca766a585a96d4bcdeaba3
//         1e883efff87a5d4594cc6831fb54da36568592186da5bf7805bb3a9dac40a82a
//         21bd3d5e0c4f452755b834b3840e5e745810c18867aae70933ce3b50a2d0f14f
//         2d507cc7a1fc1bca9ba5345adf7d43810b666de2970379057f272fd52f4ecf6b
//         08f8bd52561c3b4af3c0607553cdb98be6ee3b9ed4960132b3abb6cde26c76de
//         2d2e8a7d4e5e889b33636029c69868308385077fd519861171a6972660287fa4
//         260a81d59337057bc074fa65553df10de1a10c81d15d75ef4faf20f5198f044a
//         01376fac911185e7e0fde03f0e8c73de953939425ae3ca6f39ee493c78526c80
//         0529f94c3d5b363b35ee5ea1d1baafe4b2c933bc1fcb55ee11042ac2b6d73cb4
//         24d591e0549d639b286ae126cb120eda7fb1af0c372ac21a5969adea02b589ae
//         0fb02b601d2e7c09530f509bd042fcf2806a869eb1ab8c85ae1d64870ff14171
//         04040e17a1038e615c1d31dd3fbf96b6066922e6bf64f90a708b96d3b8c95a02
//         18c2150cead68748a643d910031ec091dc2e570a77248c9f5ee83e3873e2ffa3
//         065ada77872de3b8a2ff7698857812ab639dee912d4ceaadf1be39afac837894
//         0143e5d119524733b4958e863e4d6f05a9244bd631c747cd7132e3bba6abc26e
//         1d41f4f406686077e41704bbecfd99245b9b5f6ef4013f491a5b78e85b630931
//         128a1a0c4322cb4905c8da14ec8da7bdf35276e69fc9a6677013b7b309940a30
//         0878f99a42fa25e3e353bc0d11de34fe6ef3090077e8104c16e855a5b6f6f321
//         0f43490900c78053646dfb9da95e663c078fb12b5936f942cc6fe65847a14840
//         219e86cf3c758682332fbacfb3c7eefce6fef6219c3a661773f6600ee8c7e762
//         2584931cbb37127c264f67f1f6109d18b171fbfe9541f12d9f10dcb49ebe72a2
//         07cbb090d7ad841aac8735e65a8867f41979d94cab6294cfe85f622ff12717ce
//         2d168ae609c137d46587a9954b779ee3b4741bf135d5a8cbf3f67c9d07c2b4bb
//         25b7dfdacf142da633e927bf26feb09a341e630585cceff659c3eb08dd55316b
//         02c380d1781e4dabcd3c1f0a2208cd80d35aa2d02202f992e61ec3a46469acac
//         0912e0ab75204ee9337dc41d9c2bf4c562263da68e138bb7b01ad1a95cf023e5
//         27a2e1459edd41288b46f4a30da275fe1e2342c6f726b97d20ea87ebfe24eb98
//         2fd39adac2cdc80916cf86c57bd5c20f91adfea207493463c0bac67d0511fe0d
//         23b7350953d8184712dfa9404a057743d1cc061b07218cb128724793b20b9701
//         19b61556acfb56e673470d6a2d516dc0ecac9415907b5767591011c8079ae129
//         1574571539e35b050c7a22a16031d9010ccc6fb269b80eac2306d431e27b6c3c
//         26b67846bd67b4f02f692db07e1ee6739908dfd64e6c8b466d646005c4022449
//         0cdd728d479b1f159a357fb868334d7b60a411734cade787c32fd6749fb09d44
//         0c6b0b8a75536689f8f79f1545a0ba34173a090c3986b320a68bea768d6c862d
//         2c56897d1794c6b0e9a27ca3e8d1ada1888a18a83556531ae6bfced17859edab
//         2bec73d9324018c4179c5d93e6ac4f3bb416a8da085ae23e95c01e7dfafd2c5f
//         20d62bb65a3eff58c78b908f815a4ea75552d4f5af29a217b1f6dc7c2913c7a9
//         16dc309edd4a2625789852e29ac2bfdc674148aba4b8423db86845d058f1f9e0
//         16ad996dfe2b15a2fe69e5b799c40c4d48aceeffd7b35a83024ba549857ab4c3
//         04bbcda1cc225b136dbe98d12c9ffcb18e576abac0dea8d8c1b5026f5d21cc25
//         13e7dfaa37e5de3b120ae9467a4be0116768391561394baf7064124b52f094d4
//         1329e8fea9159c7cf4a0de98030c375729e70389b870d38fc4c0b9bd861ac47d
//         293ab934f58cd1a798cd076af9334de3764a86bfa3a6f15c310ee09919edeef3
//         037e3706105d897631148a26a25e0782fc91f522dd7bf654eed4c48cbdd6fd4f
//         0fd0161bfdd7cd3db7a5c386d46b203acc8818b7ce2c35af0de1e747cc6f6ec0
//         2c48d5bf83febd002b233d43ce54820477996f67daaa2aa32f8711e9a9a51db9
//         16155020c20caa814fd382fb12988eb687ae759623ad9fd5402b488f26fc8eb9
//         236e547472d9f2ce2484e9345969aa355d62023e35383847fa44f53ff50d2949
//         23a34fea4ec226148cbdd7fbfa151216619ff16522baf1aa3699316b88752e91
//         1ee664ad5746d558df4b38c061ba9500bc64745cae028768968c3f5b8c57eee4
//         2ffa60019b5a5518bfbf00801fb3ee7d69b37032a6dd4fc150e293c5cd3db396
//         26a9bd603fd0cc84d2d5c392947eb9493a2311da872f932e212c2826e0c248c0
//         1e055f637221d0708bfb36c85c4d39c5a39e3b7f343f2edb423615ebc1752627
//         2326975072b13e829fbbcacd2fea25e122a4d88883d9876327ff8f6d0602089a
//         245707c384067685c8adeecc80cb75a3bfea5f4636578e30613d85f06a225f65
//         032d8931e8462431c01d3c2fb6607ab586ad2a469d58f66aaf53c8ddaeacf3ee
//         1dffeb2c87594e9749afeaf1fb5d7878288f5d17c688a06ba5f911a4d3a63ec6
//         1b53ddc9bebc44f18c0cd9a04d4feb42afac11550488f61ebe4e679b814cc51a
//         1682f49edd82a2620259bbcf389fd7687dbdcf08eed624212dfc0a02a3c8795c
//         1234122a2136aba6b20c7ef767525c005988a3770b35197b180b590ea67f6a7c
//         2ca8ee632186a71b20cf8cc451a776860ae70dd8b10262b52ef1a1f5d4cda727
//         128c626d94ce6be030eafe3748235a0f202fc6975cdf94d6f0eb7978d2dda337
//         1e1574bfa780eb665e9f9d482c389334bc9054b060325a6e070c77b9349d5001
//         2ae2bc3d65206f5266237570ec6d525ddd656d78dfa57ea68347e9b2e32e18db
//         0d51ca36e5df0133f030752ddf08efcb8c06621ac0d1f92fa9861e76b701aa73
//         12c981c5037ce6a98adcc40bcc17adc0101b8b86f2aa79a3bc49e241cdea861d
//         1cc8c36deae2c2380b2f6150167100de952c191541bc7fce11b7636bbe8d906c
//         04b43af422d1835cda0a2da0670deab047aaf2fd981c3779afd89458ddb389fe
//         2258c15a71d5c39b278bd5e8d5163ea5c10cdb369fe375f8e40852fb0e0db4af
//         12994794ec206886a05f33ec14d9acb7e46182abf244ee683065d9944799decb
//     "
// );

// #[allow(dead_code)]
// static VALID_PLAIN_PROOF: [u8; 6624] = hex_literal::hex!(
//     "
//         0000000000000000000000000000000000000000000000042ab5d6d1986846cf
//         00000000000000000000000000000000000000000000000b75c020998797da78
//         0000000000000000000000000000000000000000000000005a107acb64952eca
//         000000000000000000000000000000000000000000000000000031e97a575e9d
//         00000000000000000000000000000000000000000000000b5666547acf8bd5a4
//         00000000000000000000000000000000000000000000000c410db10a01750aeb
//         00000000000000000000000000000000000000000000000d722669117f9758a4
//         000000000000000000000000000000000000000000000000000178cbf4206471
//         000000000000000000000000000000000000000000000000e91b8a11e7842c38
//         000000000000000000000000000000000000000000000007fd51009034b3357f
//         000000000000000000000000000000000000000000000009889939f81e9c7402
//         0000000000000000000000000000000000000000000000000000f94656a2ca48
//         000000000000000000000000000000000000000000000006fb128b46c1ddb67f
//         0000000000000000000000000000000000000000000000093fe27776f50224bd
//         000000000000000000000000000000000000000000000004a0c80c0da527a081
//         0000000000000000000000000000000000000000000000000001b52c2020d746
//         2e387d31be1806682bd9795d4a8ae3506c19e907e4d72018624f7c3dbebcefa1
//         2f3bcfbe2ebf68ac234cb22ef053bfcd90544804e01e74be9b619f359d3d942d
//         10384baa634aab94925080d29c88c8378b47a63f3662742e85618a626aba9dec
//         1a4f90011ff7b8ab0ca22c961dfe0853d142b604bac3309ad97bb589983b4456
//         28d9c504b9a349137fe0295586f268ba2201790f3502328384d7dfa3c1b766ea
//         011d97f5e302d16430e07347789afcd9d3c24cd6a87d20a2554e563d6809fd56
//         09de4c0ce293ba3b96cbad52ea2027630b0d8ff43d9ef999a83cc1cd66bbf03c
//         117dbcfeb68ed48d23660581568ebe7f66fab0bd8e7254d8848b80222ba7cf71
//         09de4c0ce293ba3b96cbad52ea2027630b0d8ff43d9ef999a83cc1cd66bbf03c
//         117dbcfeb68ed48d23660581568ebe7f66fab0bd8e7254d8848b80222ba7cf71
//         2f3d337096c4dbd308167ac9964d5f5a23a92384c6ac2552eec57abcca17faa5
//         23aef04ebf4e63a0039c3800bf95c29c62a4f344c38f7fea7263e00a7d045d24
//         15503b722e5cf9746baa74b56dbcb1f0b1803b348f8af0f642bc109eccd7eb18
//         202745ae3b1e01e039f0b3dd928178200f6ab303676bcc0572f73f80ece589c2
//         263750e83dd936b53796e33f4ba0d77fc60a51111b247296bbf8213321559f97
//         2b792847e46fda8ec91a734f46ab4b5c10fa156c961cea474aff761f212de89a
//         186af8efa28773679f2f66e036a3abf415371df012017bc0ff867a7ad0c42be1
//         17f955833eaa2cc21920ded64addac6912fcca5867b7f4d0445b7b191f3bd420
//         17eb65dd48732dab4998eab27175d0e612b9faf491468fb2d55928b122c34cf1
//         0a2945444f3fedd66d78f7066347aa6d478564b4fe27b86b3c2f6e547532bfdb
//         07fecfd43a11bd49dcc28e205b80d17e697a2323d94386f5e1992dcd514cec11
//         0d574e82aa8abfdff8e34a4221fb0cf038c6fa3cc7b52a772e513380deec3526
//         2ac78f0d8ffe1ac1afbc64dc43c0b8e9dc853720622f988082bef91a00a34a8c
//         0c0c9122a36992e04184408670b35bcd2c1fbdf08ea8dd046e0f758502d167d8
//         2cb1ce621c238d494d7ec3982d630f45d9dc527e41650be60665fd7355929779
//         0999c2fe11c0a64a5d1de1867358db773785f8cab579b897c0278c8c9fdc82fa
//         29e13bd30c6d1a66c1b3e5df4c4a73e592a3aab4e9861d88373126fcccc013a4
//         28be1f3adf86b95c5e65d4f3bf8fa13f41d8abf26e71c311448f1a20313355ee
//         1431c08ffc197efcc61848b0eed4689682a0fd4bc85479b28f6ccd8f40ea54ad
//         017baf61083884631efe908748d538895ed6b613823006f9336caad6483a8f45
//         16cca64cb6787b5c3c174c5c62eab4bb305f77e4de2eab21297cdd9c13ec59f9
//         1844114777939330530525f6f1de65cdf9fe733fc1c74c9a1fbf2053d6fa2855
//         0fac55798434266af43a432e29cad5403769f8d29d10980984fea54375d64790
//         1d18632e3938949e95d41282b4cd48b5cd3dfc876534ad25198605c74a3ff026
//         2f60a921a5e79c4d0cba8e90be3f4edb68110c578962966c106796697849bdfa
//         0e4ac720105fc685e379c7521688b2fe66b58f7ec6a87a1416475098700f7921
//         2343f1776b83a304586cc7a001f1011cada240f06679b37b19dbdbb669de433c
//         072672487c4637a51566b698301f48075a60fd8f302f671ca376c2dffc3785be
//         22b72b3f75cc4eb3fe6409705f50a8746ad5f1ae5505b617e9c79fa43e8269c1
//         24ecfae504fcfeb154eac931f9994cfd29ae3483a5ae0e8ce3b9919c886bd949
//         0586c1d15f3b93731ca43fdd0ed8875f28d707a2e12200a1dc8b92489ed3059d
//         10eb7e11a917b7f69198a71549e2b7e259745f786630ce8bd2b16ee1eaad7f0f
//         0f28845aa9abf7d0da9d41fbc30c2d7084fcd09b669be2d519fad3f40f31673c
//         012f2e3cdbe532c5409aeb296182f1df5350aefd1f886db20ef8460eab547f0e
//         12380ecfa984a1cb1863a811b2de7c28fb52bf746ea0e4701f24ec2d31208edf
//         1f98a559c03ca53af4e32270f0c93e8c2bf548a7eb1c3526ebbd6f8e2be041b4
//         06daeb384a0ddc603c8617568f18965128347e0eca83cdbb57f56ff9205ba75e
//         1ea288eff79c94217b98fdfef4ac0c2baa6c7e2737c6d169acc76b755430385b
//         11586d5b7523bdb4ac1d165af037807ea03a3fe97f0ee794c9b074d62d0129d1
//         2f940e35890d5b8c7bc3c4565bd8d076bc5419deb195906ca414de420dc08002
//         188ada4d1970a98690bfab5f39e70c96b9519868349c3ce0fee3d19379d50bfc
//         00aa7fa19c81177034858e623df3f6f3e2a51eb29da27da59d011fd8b8cb5c83
//         2058620196411d5d08130150711abf9baae631361aa4a54f3b5e7f34bf229da9
//         12f51a29e41c06f5e769eb4457b54486ca1c922c64d3af010a45171f2b4bca8f
//         1d9a94e0b15dbc24de8752c3b982c226471a78a39e6095a0eface11fedacd706
//         0e672c6133a8626c21b31b6c682a9025d79971607c2ed74f89d9f4e7361cb645
//         10015925bfbb80642e1a63a3236d710ffb8ea1ba19b2b05a8bef9ae413aef012
//         0d90c5fe86ebf131140c86c5af28e3a33d897381d6869b8df6a8a49a7986eacd
//         14f0924e72ca03e8ea1a99fa9b968d38d6ef94034f15b1cac65a4f86dba40e53
//         0fb95d1bdc11b4b0347fb4256e2c3efdc5232e185ecb3ad02800b5ab8ca45090
//         2f73f2815228bf208ef328dfac7e35269d15ee1e96de42793b56b7bfee31d45e
//         0021d203853367c253530081b5e384c84371a883852a600d7c61fa752600963e
//         1d6a452fa5e153e3fb16dc42c1317d5c70981c36d407ad10f98fc7752d1d488e
//         0dc29b2eee2e70925fdaa1ce625bfe3eb62dd3020d1cdbe1aec039c939c69575
//         173963002447d2da224eded55f592e7d6c82a4d32b85215f426dee2aff8e5fac
//         2c4be219f0525c74de04aebca6cc227bd97e278f5bd9fac4ecb39061f719f948
//         0d49bfe1d99a852761f6b749f380997edbc1a482e8712ada8a492f4b635ff9c5
//         14892b3db95d8dea5929fd9922a160328454ca508e611e77b543e01bc7e66781
//         03ba0a46e6fad7d2ed3bcff8ebd60990ac122fbeef7706d8f0af31afdfbe17d7
//         08dd1340d9cccb96f78088ea6ad3f800dbd7b8728d435e3e151705da01738132
//         2467177059074c651952a9c73c64a666b3f1bc215429aadf948fe688ab181b6a
//         156a4b7d2d2579827808bb419448c8f6630235ba9f06b911d597c41467222dc4
//         2660860fc66b41aa7f7b5286e4c452d2a1a5e6c8a2f0eadcd148f9318d3dbbe2
//         235d44de438c5bdb1d9360756a87593cd3aaf6251db475fd48dd22b87c8d44b4
//         0b8bfd9223adf5f895994baf58e8f3db694dfb41b21732993a2605e74c020143
//         120d093eb32018b1824d80c63c65eae04d0032e9e35fb5aaffae732e39ba0ebf
//         2788dac29eb7f58b5bcb349bd783e9f202f17c4ff4478d6c2833f68c3f8e6cdf
//         2a05f74c50b8edc7c62d683aaaa284afea164c9fc5529dd3afffc38b16434e41
//         0b90157212df28c7318ddbccd2e444927e40dee48e3f9c903c39e68825c13107
//         08ac3272d1796ae7b43e657da5ad935d411a8b3514910048604439071eb6e716
//         1ea13b56a7c9d1366b0f7c1dfc3527fc857b56b767ceafb10393ac47070d5a61
//         0287a8fd7299f5a673daefd61a8097cf35a165ed60ada6dd5a32ed312a714021
//         12a1a8725bbcf73957cd89b8f3c4f484caf02a791ea5e5c727351f31beb3a186
//         1ad6e13187b786522ba66cb9c5faead068b008e2c37d06d897ab733b9b1cf10c
//         2c823c5936babb195db22833318900c1c4097458755cfea909e8ef32ac8431e7
//         257e379c59e09bae22bcfbca89e9ecb869b1b6d61b88a1ed2199e2edb4c36b3a
//         292333b7d706ee3c0614208f90874cb9e8813187166665d2ca9afd1722a77e90
//         176522a2c85a3d9d3d863a2a6b1375501fe393739db43df538d1e48bde0014ba
//         0517af3dd9e8520f7921e4fd3af129f685eae54621075e1321fc65d42a99b425
//         25abb89c31f26d25bbe4469ec93d751bc001698254aab2244d671d1a06aef5ea
//         26c53333a1bc8339f783c0c630cd59dea53bb0133d26be599cb67b2b66f2a77e
//         2113eb6960c0c624926a39945cb53cc8ada5c04f2cf51e2cf568ec4da36e1496
//         12e39599c11db3c81c575bcaa0f4208f6bea01e6b516ae9b6554e7d28091f6f3
//         23dbff07e78ac78071470a7bf1615c0cfbd47b966c9545c9e0209e2bd1ef9aa9
//         1a25f34bc5eef45ee8fcd3f1e021d82ebd5c7d6dd1411cbceea41d58b5d59110
//         13e13b71cf1a96a7a414610ee4049016277d3f9ca565821f5a1ea473e85960fe
//         001cb30c62a111740f33fc003b4b220ab2a0a3989e34f7e08500ce3a1a0f2117
//         0eb864a56e0eb706bee1745dda66c8b10da4a653248353e5817cbd0085873472
//         18e7cb126f4b6ae03b76aec7c052b779f62ca986fad798d07c24f254db00fea5
//         2252b864dedcac5b7193259a2ce4f028c4f722b96c18f666e5fbaa2a7211c687
//         0a45200b925492ec8f2a3232650ce18506dd58f26362007946dca3f71fe757b6
//         15510de2831aec5f17a33c6ed2730c360fd00ba8099f73fa80f0ca828fcc9d13
//         2eeab8949b3d473335de7251ebb69b5e4bcbb7d569887b8a98e3e27233bc42ac
//         23ff0a44191e763f63a1c8043aa03fdd1dadfdab1a02ea47c4c337f339fa5374
//         3010e779bdc50364f03567eea315e84ff5053511acb8d260101b984dceddfe4b
//         2331b6c7303f7b695d6ba9f568e8ccb6b574c26bb36880de95684b437ad5d5ba
//         1593fafe05f87d36e32ff43c7c07e69d919bed350d3421e62660a4702c181370
//         1aebb5043714fd3af053b8267e194ce69ab27e347bd60d6d60578b991cce757d
//         142cd42b528d58f9639bdb807ddfd17bc443b290e6d3c697a09b3e1fcf400ffa
//         0303aa30a8ba0ccab66c8d965614e84fae37104e2c1bb7fecfb09567bb7ae5b1
//         268a8ace306fd9cbe701b1e5e622612657c7cb851d0ebd58570c976b2a11d1f6
//         25edd8aeefd9442da00e63cc9f89e334d0619be5e7571a25cb13123ca5e8d855
//         28ac5383f4a4674dbccbb5c20f0f4821d2b2fb8d0c79d69a41f037c4e7e12020
//         0977ca15e933830add9e0943031abed062ba45147fe1560e49ad46625052d8a6
//         0697ef7a5a0f9d451c4abcabd71a2482133ebfa2a5d29d8b35da9f92a3ef4e32
//         273fd0de49bf1478807b437db93ff7dc8c99ed04d51e7ff37cb52420d9d8f1f5
//         0ce7a75994b572cbbc6f56d8279923bee1c553d5f04168e1f0ccdea049339b52
//         05902cb2b6abf15c80dcced2e5454f482dcb821ee47d8eb67f488482bb682e1d
//         040345ea71a9871788cb32124e5165d72e9ee046081a90a89c9b7d6fa3bacf40
//         23f1ce1bee5e10551de3e29c8c2d2883218c52b9a8deaf9797d5d59ccee6f1e9
//         16ef77df6b4a2d039b81ed405816ace95babff0b75a00d9427336dc1327ad06a
//         2982a7624e16cff3dd63a7dc985ae380b34ace55133904dc68054b0bd6689c8d
//         093eff5899f316e182efeb0a622d928bbcc2a175945036492f1759405d449ff2
//         0552f21fa9ed22540e6d9c48b793f8940e5eef8a3ff2370d4bb21c7b6fcfa8c1
//         147f479420f623f8891798c99fd434b68e458ac13c9c5b072f9eb7e9aec21acc
//         2dfc7db588626f37fe38777fb410c9f88322486c268f5b99cf84109bf9918d2c
//         2e9e234b2e836b43016e7381264ccd1090934807600cf178ff238a982b0725b2
//         11edcf5d005f26bfb4d305aa9b333fc6076633e065dcef17c604043c143ae9f2
//         00054cbb4eaac4c35b386e16f133ed40bcb4312d73e5f850f0119b827b84ba8d
//         263c0163a2b4ef9dd79e3d3865ee3368c807cbba777c6d3299b95d0e8c1ef435
//         0772dc33a765d694e78163925e699445fd6e22ba6a72cfd5ee3d45dfd7662891
//         0b9d7a859fe7ad01fd32cbd367f88116f4bc907f6e25a42768b5e95dc7932937
//         073d31c08ccdcf82eb20eefae29af4a31a97513172d01856906b28e5b605deda
//         206a43631d7a0a2d5705138606e8ebf8a1a5fe4cde9c40eb8c7c4e79b744bc48
//         2199d19935a0d0de790c82384d4e83fe322826589bd726cebf0a76f00eca4e47
//         189edf4f6f2431ce8b588d3116a7f446e77222ee8d68b1a9d9ca483a25eff2a9
//         23730c7b285d858c2950441d2078799f8f1e34df6004e50e32afeca0ca744d10
//         227c5fb4627feb427c573c59bacee95147dc291eb49255de62bfb132a5c83cfa
//         2452d95586e3fd46cc99fe7162c025e374d65e7b255ece1debabbca5299300c7
//         03a05b55aea206e7629631d6c76fc7003bee0855ccd28b6b949bc842e6fa96c9
//         171988476b163cf8f2ccda220320efc882069fd1eaeeec8863b057d3d86ed948
//         2a00c2a21445132d35003d9be0e8f252db327cca16f9a4f0908c0ab176047dfb
//         1016759bf7f9e0689c6fec89e28ddf944569d1711eb9ed0cdbe89efc7017c777
//         18dc6d1c32774e66e5a339745e01898dbcc13381d77daf0d0c40d3aec3c59822
//         1d52d06a37e118060b796730a4f53d74f44ad167865a64ef5a07eb0ee1dfd747
//         2ebc91cdeaaa325c9f6fc942d1fba73f0c21e099039f8268ecaa82b00dfe0987
//         206516b2cac5b7bf28f9a8ae57bab1bfef81ba443df9c7fdbf6259d04f3bc2f0
//         206516b2cac5b7bf28f9a8ae57bab1bfef81ba443df9c7fdbf6259d04f3bc2f0
//         122b645b4d7f5853ca7e4cf70010593817f2f3df6d4e2e112ec98228d807e501
//         0e53cb8598542e71490715a40983ac1c86710499e5611ea4e077660653752ac8
//         21e8623d16a7e688d16f2266843701648d76a95cce2530650d83adabd24ea76b
//         0d9f882633601a615f7f4d6fc55bce47e9531a9c85ff803e31c94355e4fe33a3
//         021719eeee23be3bb0ceaf9b0c8fb39896cdd684115f04a0cf063add3d54e4c4
//         0a3d00f5085158d6490b85813a99f83ca9aeb197856245b1177560da0e4ec695
//         278aba6e25054d8888aaef915150842367d22b4a1f49061f0b2a25657ac64778
//         05b0cb70a63b503a2e48af382d4e73e07a3102e65a2c1dd96e3de2cb17fadd97
//         0aca6a6fac9aa2340b3249224b696282973d331f929e484b3f98fcb4ca2b3e67
//         2b41bc77473edf34c686fad184e539022f9567521db9c5bde71246d340847928
//         2b502225463dc2201d4c9276fae01c543bace6e139f44f4fa718b2caa9c5fd60
//         0050ad60c29f3f12da5a69a605fe95c873e26de05bb9a52ccab5c29354f1dbed
//         24804039fa8d5520f3b0bf4526057f2f30328d2ae6c2137368095ba98d71229c
//         03d105d862daeeffa7c0c78a3b2d98193c06f38ab936bce5819234289af92d64
//         2620f1f447562afbf1608d106baf4382a3237a59e4cd18495816bfe8e6508e77
//         26cdbfb46b5e48019851694b7a0b04ea40d99da1cc0bbb4be6d7d505852b5ff6
//         0fd045acef45c1bf981fd9ff87ae9e103fe699b3887d43f525a908e56d35931b
//         2286eaf3dc95cfb05e469dd5be999ec6451ed7dfd6d8210ab9c5fffd5f004a4c
//         24a118b435e726ab69e9544cd3f1432ffe8ade60c75f7f1261c14bad63050955
//         30341a14e7aa7cdb984b4f70e26be113b8fd99607b3b8f7a76aee9d127d38ab3
//         1f5d3cc5540d6e5cc9166ed02a8458ab73e31f5291fbe29c55d9eac9956fb5b6
//         1683122424c9749361de6ba69b31e315a363762db5d89048a06967895d3a94b4
//         1693bd45c80e2b7a1315fbe13035572c03d6be6cd012a5499c3134e7ac4526d1
//         2b64e6d55f3702e93992495645b9376902b505bf0eef63b4148ab93ad9029428
//         1fd19a0c49a3a2f3942804a8c3cddc5973e8d49a36f73ff5f79175a23a195b30
//         26845c2ae42db83d2962e7e89152edaecca0a6dd7c2edd63a354c26b493c3481
//         06de0011d27f541b7c104d8269495c630332ad113fb7bbd9aa1ce84191a47442
//         0b1f33c291e3d3f79c7485eab5ac4153ba467d519dcd4fab9f58e701911d192a
//         17c68d617b1df0ba129d121448dde82ee1df09a8a8c548d22b295c111eaad361
//         0420cc2ef121f62750bd58145d466a21cc270a5d4f1da437cb5448fcff8edb86
//         28cd8dd8e083528c0ec9c78b4d705741935eccbd50b99899e00c2a099b3b70b5
//         0de9ca0226d7c42571a526c9227508535cb1647c67399ca3d053067d15b38769
//         1724f85e57dbc37c4eec9340d4a9a99c401a822f5d4ae41ef2e2e06d61a26dd3
//         26718dd41b0e6cde48a08d34dbadcfed6dd985e8cc270599d4aecbc7562f0a9c
//         00b07c0f5f7db7ef184fe5c769881928c968eec6355e233e938404445b28ec5e
//         30597c2afb9ffd9c1fae35007689d9fc4e0f021b97fba488d10e47f9c49d55f1
//         029668aa123aeb7ed6630112d22ac8ce03f31e10892e926e0dbfb1380818733d
//         08129cc15a750dbf4a24115ad6b0ffdd2c8632293fecd4feac11df9109f0afdf
//         2409eb162e4427721c9cdae3a0805667dfa69e678ecc39ab3fcc72af92013cb9
//         13dccb841aa8d20bcf80816f01424c20c2986b01bf946dac089212ef6aa72bde
//         2841faf37cbbb27d311a5da45d6cd1cb107a91722022ab70fd0b20a548bd78c7
//         2fb3d68844d822b53d18facad6ecda43f94a2ad968ac3856e105085be9d39407
//         1afe98edc91550a8436e95a5dec62bdbf1a10565cde95e1a531b5371f0f66c63
//     "
// );

// #[allow(dead_code)]
// static VALID_VK: crate::Vk = hex_literal::hex!(
//     "
//         000000000000000000000000000000000000000000000000000000000000000c
//         0000000000000000000000000000000000000000000000000000000000000011
//         0000000000000000000000000000000000000000000000000000000000000001
//         142bd66bdb7a2bc125c78e040da5a5cbe6f296ee1a11b55dce82f38413640a64
//         0d1415082e63c88eaa34836fe60428f70dee92853dc8a5d19d0bf85b0fa95ad4
//         2deae537974aa5697c77ce4f20f0fd5a3a264861cd51216bd8c56683467cd704
//         068627460599c3db714496966bf5f4374fb6087ba1179c7a8ed5c59a1015e784
//         06681df238df2a0f864a67847d46222c9aee090f36d34df5c2aab80f85a218f2
//         18e37167fd19d013b9c1b8da3c671ec025fe40eebc5d11d2d55e4ac8adccae27
//         177c7a07701c29d13dc4669f3d97f847e96e4bcefe3f9cf39c6f73896b06e821
//         2d31ea12ba12ee2338f1a638b192ffc9b995fd687c23cf6fe4a49a8e4f4c5aba
//         2d623ef9f6f62903ac68b01fa7f3faaa5a881854d8b0a3fb6597416a145754e3
//         20b75671e0dd20da52b442fa3ce1643a24c7ac8e6059e6db24a7e1bfc51be2ac
//         07ea8dd8b4d3fd18e2edafe7a56dfd287d677b48528aeba6bdb01913c3236ff8
//         2826602478d64dc4e23f777f35827a35ea2716bc853ad38b76968342e932d84b
//         0c4032c3079594eb75a8449d3d5ce8bc3661650d53f9b24d923d8f404cb0bbc9
//         1084d709650356d40f0158fd6da81f54eb5fe796a0ca89441369b7c24301f851
//         3057f9cfd5edb0e7c2c72a3c00da41d4c7c7b6d01c518c7ea0b38257198a9523
//         027eb0839ef4980d6a98575cedc65ee5700a7148f647b842154c13fa145167b7
//         1775fbd3ba5e43163b9a8bc08ae2fdbd8e9dc005befcd8cd631818993e143702
//         1d8011ee756abfa19e409fcb2e19a72238c07631bdde06678d3bce4455d2086f
//         09c706e73a00d84450fb0eae1d72783faba96bc3990b1eaa86226b3574e5c43f
//         276d401f1c0f9a2668fcae74683a84de1084739f9b1f547ec36638d7b5a1ecd9
//         12b12523f7d04a83276f3537c64334419e05b13fc20dedd7ff81c5677d3286ce
//         2e741be4fe42cc1155a526829445f3fda95e243c4e602d1c13a77a2472f082da
//         16a1350662e14b4ce4e8e364439d0abba02dc62a5526834f93c6db848d56dcb0
//         0563b1f480cad9069296d71489889503dda143f0b2e746ed0b6e85782c26040e
//         20e1bb3056279dc342f6c756379f231d5f472088e4d88b5517e40b2a0133f401
//         23ee36ecb11b62789eb4da763de77062d23ce01e2c8d1a5a6b3bd0ec93b42e77
//         0d1611c856951969fdda50b3205d5aa4486b632519d18424d0e92f60a31671d9
//         0ce97ee59d45d76230c0b534ea958de4c47e2f4c94aa3cadd7cd42e719521e0f
//         20c3e9857d73168eb049a6954dc31925100f44ca6398ee5652af680e254a4fc3
//         0be4d7b7685a137af9634d95d97f6024ba3216202ee80fc5fccac6b5cf2e4582
//         17e8098747feaf6d1854b8f18b6cb27185a672f5f5f16c2d0d6e7789a8d6ae00
//         0be3a351f7b48a0266a64c4eb69c19bbd4064ee848538cd46be7f549bb19fa05
//         2234971cd4054b723b6dca8ef55e4c62d67459e24f49c3326d2bfe01486af77a
//         0e96a04cc899f1e6aecc74fc7409cce5bd5318cd1bb72d1735d0cdb199cd179a
//         2c1a60dd4bc15efd19338957640268374683cc8417ae895b99f3215f597e7c48
//         1102b6dd02b49e3ea8f160d2e0ac2b9db8285906c17ad250a1bfa97f1731a183
//         13d9a8f63fda3aafaf71dd2eca25b293627bede4427bb7d9484fd637ec9c3339
//         0a08d0e381b054e808a8a780038559f35c06b650aa54cd26caac1f3f317ad73a
//         07ccc476d535a06f9f7388b04387bf331db992875edc3658257a7c25651f395c
//         00651fb2654053aedf8c01651ca7e5c11988ef1c0d084df3a8988bd89d930f83
//         0c133f1122a6aa216331840ab987b2f15217d3ee50ec9f9702abeb71b79e9645
//         2ff1de9d5413b8ccf0a625d78323e3c0f0beedb8abd96cda2f25a7f615ace981
//         16de4faf175d977b285160405a07f4a6503eaf75a2445d0be75cb81b1fb244af
//         226e2bc5a7f92698bd1ecfbcf1259a054128400368d40f11b37956404b1b6668
//         099e3bd5a0a00ab7fe18040105b9b395b5d8b7b4a63b05df652b0d10ef146d26
//         0015b8d2515d76e2ccec99dcd194592129af3a637f5a622a32440f860d1e2a7f
//         1b917517920bad3d8bc01c9595092a222b888108dc25d1aa450e0b4bc212c37e
//         305e8992b148eedb22e6e992077a84482141c7ebe42000a1d58ccb74381f6d19
//         061f64497996e8915722501e9e367938ed8da2375186b518c7345c60b1134b2d
//         1b84d38339321f405ebaf6a2f830842ad3d7cb59792e11c0d2691f317fd50e6e
//         043d063b130adfb37342af45d0155a28edd1a7e46c840d9c943fdf45521c64ce
//         261522c4089330646aff96736194949330952ae74c573d1686d9cb4a00733854
//         0000000000000000000000000000000000000000000000000000000000000001
//         0000000000000000000000000000000000000000000000000000000000000002
//         06a032e44c27b0ce9ed4d186a2debd4bfe72be9bc894b742744cf102a554d06f
//         053396ef4f905183ad76960162ff0d8c34d25b6126660c8385d13a63d2078399
//     "
// );
