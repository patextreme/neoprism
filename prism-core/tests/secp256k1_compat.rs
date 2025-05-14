use std::str::FromStr;

use apollo::hex::HexStr;
use prism_core::crypto::secp256k1::Secp256k1PublicKey;
use prism_core::crypto::{ToPublicKey, Verifiable};

/// test vector from https://github.com/hyperledger/identus-apollo/pull/154/files
const TEST_VECTOR: [TestInput; 50] = [
    TestInput {
        public_key: "025ec3069f260463ab79c6ada107de5ef43da1663eb4092d1718f5d26f57f2884b",
        _private_key: "66f4120c9e9a4b78bb095129faa4c8d7b90df3f9d205ff175f80c84dfb47ba21",
        data: "33121a19313e349bafada4113e364a01d578007a02dc",
        signature: "3046022100ce972f4df5ab2d6aa20151bd56d92f9db42a6b6e9bdbd78971ea80828e183683022100a30cef9d2d28bc1710cec2c1966eb1e5ac965d0be4774a60ba38a462d56c7e7c",
    },
    TestInput {
        public_key: "02e2c22135cba30f2b4796626819c9fc7029ae3be3a924802078ff204f9fb263a9",
        _private_key: "27c33d32edd9e9e26441fddefcd5d0caefa1c89f0b924252eac8ab61aae52699",
        data: "15931303fc9afa09fd8ac1",
        signature: "3046022100da8e9a5724b99de4afe938233530fa1c6c8363eb67f9408d53d4941a0ec79fbb02210086b6bba2687e3d412e6e33455c98250a04fb334ae5bca1f46bc76dba6115f726",
    },
    TestInput {
        public_key: "03752cb21c3687c1a5a2d18fbb1557b97e6aa666a5af099bbbe4f629ea6d83b1e1",
        _private_key: "a66cb1e5ed0b7ded6c024244ba153968fd9f7309f97fe47da5db3f1c10d0c295",
        data: "226d3d0df1f3c475684457bf656dd59e",
        signature: "3044022019f198ca5d3f43d85d6621a15e560dcc9b79fe4009930d8dce03944f3bd79e2e0220516cbfa1f6917a14990a4c9d5b32eb86fc1cde62538f2e99ab95f6fd40919519",
    },
    TestInput {
        public_key: "02ac19f9dd09a00a845b6133c2832da6a5308f0be86ca53ca2425d1876c3f568b2",
        _private_key: "7ac883fa613f72cc92c5b0b4b3f8cbc53a4c90cff59b8c7ad125681994a55ee1",
        data: "19b6993da7739b086ca04cc4b3",
        signature: "304502206bca4b8bd479a15cf23b768ab19577fbd87aca3e60ba0b4db3f6ea67080d0aa9022100f1e9fe9d8d236af74622edbffed5e2e6740531a7601e9e205cdf1769ac3ea7cd",
    },
    TestInput {
        public_key: "035a9a596978d0777e2a840333636ee78cbd6861eaf73a724ca051820f312537bf",
        _private_key: "e00b4c0c7d8b76163e1f3700e75ddbe2ecc2157a802eded5368b2e96bc448e19",
        data: "99d5dd82b45eaf61d0187d2a11",
        signature: "304402207e1e98b08da0b914c7e5f91c54eb7a765c761cd725f7c118dd1210321b2a88a402204f12d0c5c9cdef8199e40b00840893f343088f216bb45e2d3c81a7cc24df55a4",
    },
    TestInput {
        public_key: "024860f24369911e01cc475a88e5455c692c55976b870728c8a8b877b5df04669d",
        _private_key: "dcfd133de55540d7babe648235a75faab0ece1409854bb401aeaf21b49c7568a",
        data: "b53857e29f52275586",
        signature: "3045022045b5fc308afe2cd9ef6f4e67e7f9a3ca9dfb4091fbbe951035cd5f55da6e3235022100b4ab20e19d55b8c235bb4e0ee4020dbbb58fab0185193b698cfe33a291c6a560",
    },
    TestInput {
        public_key: "03c41f327bd37ede1c985ead9ef23f4bac37f4d6734ea8e376b8c6bf3147e28f91",
        _private_key: "3f40a66602709f1e976ea420757fd58af6bdec08bad204fae3fcedd97ea00adc",
        data: "10aef7c7acce95f49e7799ee0205",
        signature: "30440220174473ba0c4cf1a055742bb42da5f1696086c06ac5f1004bbb12cac0ec11b81802200219e9cbeb5378600e269777fb33f4a0328dd8fbfa42352bf3284528cdbb0fcb",
    },
    TestInput {
        public_key: "0265520a34189b61a9e02c711372ac685ca5facc154918bf74d1374e173bc6abf6",
        _private_key: "24f869c5801f8e6612313d6574d1fa7ea51141e95adae277ea757faf5cba4e83",
        data: "5a39a343131e1a4b0f69f0ec5f",
        signature: "3045022100da58ad12cd4d18220eb18775e573560ec05ec0f459728f5d70603aee5f256ce2022017385e5895f341c1a5e5fe1035350c800d101a014dd8c3bdc2d19febdc6835bc",
    },
    TestInput {
        public_key: "03bed6a8a0fd4b7af194608b96b8000adc6c0fb98e1b5cf5ca88c0fec71108127d",
        _private_key: "2238a578e9a0f069b62bf62e10838fa16dcecf4e8e3db963e88b69df87bca71b",
        data: "2f6e2230c4dedafe96aa422f9b5e",
        signature: "3045022100e9d2f7876078c53db6488ba9a4dbacdba5f30341577c5427a1ef0b0ffcfd4c2b0220072f0705ef235617630b49dce2f75baf3a844ee3d88cdbcd21d89430f9a721c6",
    },
    TestInput {
        public_key: "02cdb3a84f0704a842b38b3e70a9d68accd94e712a3e03fc4270e2000dfe1480d3",
        _private_key: "7712a96be650d333455039cdf1fe1c9bc3b9e481ef45a12396f7f57303e3e207",
        data: "2ab555",
        signature: "304502206dc9e71d3fa1612a77a118145fa031191ce913f0722152e73bc2c6ed663fac98022100db27302f229bd5584f66c14c9bade299a976cd740fadbc609fcfab1f1544cfd4",
    },
    TestInput {
        public_key: "034eb34ae043352bf1f125244a3d534a5f6b98dbd82ae6146d74e8bc0e3edb9837",
        _private_key: "6f201a55d98d2c308a58f29fea8b6e0d2db5e9046a47956ffb08a9fd37477ae3",
        data: "7bac7995678ce23b85074372c28bbba3dd3e013cce91a497b71e37744d43",
        signature: "3045022025c3f5b4dca9c6dd5e1f143758ba3e071f9ce4c902b1df9b950ce422e4e352e7022100c340d8a19d5a4e71f3b35e8ac0366b49e31b884d6790d9df36d0562d925f5df1",
    },
    TestInput {
        public_key: "02198d212d5f2738202828ac10ff174ec99f0fb6710dfb759b3afcf2bb211bbf0b",
        _private_key: "c20f7da0e0a812c7238545647e2cfc50038067314af960784bd25577a3446f84",
        data: "9cf3f6f8e6047c352d217b2cdffe7925495379b15c2f48",
        signature: "304402202de1c2003449d5e2a3b2e1eaaddcb42cdf7abd630768ff737edcbb0e93a74d0102201df34e3f6cb1be6b7162b21cc294af6d73c339ff04c08c41598407f3e019ea4d",
    },
    TestInput {
        public_key: "03fd09989559b8d2f37886c9b68ab1dc09ba7620ed7f40b39ac6c2ab9acfc8b290",
        _private_key: "c9408ef52bb8486f36f7883f85ae71ac60160073dfcfda0b0e4589c0a908eec3",
        data: "8e",
        signature: "304402203f64b6333f76151efaa17eec156a9eb410942afddbdb1e988154dc26c1e670ae0220494a0e2331aab2d7d9021cfb192da40de464591a0cfed9fa4ee1b89a62c9692a",
    },
    TestInput {
        public_key: "02b5de6a254244e63a0031cdd0ec5c547d280fac45a9797490be91917ae6dd5760",
        _private_key: "5d7c17b9e1762ad06a66a9eb48deb9940459e77760ffc98fdfbfc0f17df13675",
        data: "af629bb4f46ea77424b071e4cd267bc02135c3013d7748842e6d",
        signature: "3046022100a066f2c0a20ab561858c8bc5e58cb72e931ccece537716bca10003b8fdd1167e022100c5f4248f52cb39a7378c3c4d615970ca81a18590a2fdb8ba2f30a06a0ed61d61",
    },
    TestInput {
        public_key: "03838435445b8dee456bddddedbae7401753b1bcaa99907f02d8d9002289f722fc",
        _private_key: "4389080b2f347758072cf723503b2ca7c290baa8d5e6fbef0f7788ab4b8ac86c",
        data: "49d3ba81064512e9",
        signature: "3046022100d0391d91d9f1a0269f94f1c38f418789452a9ed02ed29eafc1b23dd510d68b7c022100dca843653ade5a16492f80c9d54862c29fcdf7c58eaef085990f1b0caaca0987",
    },
    TestInput {
        public_key: "034ef4bd6214d9f86a7393a671ad5ccb2a9e526af9aeec3c3cc303465edd973b0d",
        _private_key: "d6c287de37445f559bad4b7b78aa465d7a9aa2190dd01234d8e6932f1b036ac5",
        data: "e2b1330ff2eaaf82f8364b727e1893b29956787d42e0301ba765efc259d7",
        signature: "304402201f857c891047f7704639e5e44117f544a60c4978cab905a1560cfbc03c7f796602206b62b9c1a237d07a0c0fd700f807b67de9d271a49a9eb6bfedb39e0100a728e7",
    },
    TestInput {
        public_key: "02de7e848977cf7409d5eeb6863d1cb8544d12de50f821ea844bcfad3a0af354d4",
        _private_key: "1ca3d9cba4159d89520557b2022baa730aab8a34c205a3debab9c73a012593e6",
        data: "1eb1269f2b4d92fa9b704684b5c899b3cd2a",
        signature: "3046022100b16bba6e401ff7c0945f864d471b0a42d186c0f992b66a37ec4aef0c02543bf4022100c4352d407b61bbacd9c8513deba0d05736a54e429465f884d9c390f846eed108",
    },
    TestInput {
        public_key: "0302059fff5c903632fa9bd3e2a6138b02e0873a128ca03bbfd37efec403a5669f",
        _private_key: "c55fc84f80ce30142f38209ff013121430a022e08ecf186c5dc7eca185129963",
        data: "a358696fa77d",
        signature: "3044022072f1e5dbbcc1a866a4f9a856734d0739dc71b152cecdff0398854a0478d1faf1022024cd12e3b270dd71f266b26aba5c21e726ad99fc6c591ee53d1ace62a545ccb5",
    },
    TestInput {
        public_key: "033f08fc5adeef183bbca59923a583eb91efac59cb169f4e7cb70bcc1bbee06fdc",
        _private_key: "f094d49e19b00ce0b2f2786646542c3f203972c666c846a470b5d4539ef38b75",
        data: "c8",
        signature: "3045022100faf186b545801a10c0c3b6860076a3c2c1ac3106b399ff6f926391d5f58db7590220220830b0c2fe368f5fd043e1109e9df11002ed92c9d5c317ddb46df0b508107a",
    },
    TestInput {
        public_key: "0262ae6149f128e7b727252c9ca5df84b49770b70d39accadbddfeedb34e0a7a0e",
        _private_key: "2e9aa586d4c89100180c90d421261eb5faa5bce36c48efa7035ce8143611eeea",
        data: "8b3788243f3c98e44962bc03e5da3d1caca14708bf6d",
        signature: "3045022100abc3048d145b7ee5a5a22348206b51bfc2716cb2c3e76337a217d628dc15656302200c6b85f099870234c110ff76dd40b42a80d6900a4636b843a58f99262f48d54c",
    },
    TestInput {
        public_key: "03b722749ca5e7b3face8cbce50fa3096a0bb48b714c23330a619688d3ef2685e5",
        _private_key: "6c7251782ed986786d0ef34a5ec7e6f4342a6a8904cd40c92325f59833c9eb15",
        data: "c2e083c32b6b8de993e65e00f438098af54278f6ede84709483c60b46a71",
        signature: "30440220516a705c5d42ba5858330dc8911248096ddec0df889cb6b55e0f082d818960a6022030939109a1db738090467e726997f8d90db2718e3be60c431608ebf556712eff",
    },
    TestInput {
        public_key: "03d8a2da096adac81bafcfbae8c8b7b38bdd319410899961e561e9cace7caed7eb",
        _private_key: "4a4e8b44d3eb4280eae78d7466cb86e5ad235ae269ed97885d7bb01c5b0cf745",
        data: "d662ea17e73481ae85ccbc01704d36f13680",
        signature: "3046022100a9af9a342bfa10221c6de02805fcc61217b212760e12de6c459c22ba0fe932a0022100fee33be31c2bf26d3b918f37ccebf8e457acb7882feecb65b59ef7d2bf3d1da2",
    },
    TestInput {
        public_key: "0374fc049f4b4b4fe8e17f02742989f3d6555b0c230d566ac69effa44f99d32267",
        _private_key: "02c51ab9cf4c27ebe0e1cc249cb43b23d25f9172df2a859ccc1b19cc926af1c0",
        data: "3799b0dedc31bded83878b11fdeb10",
        signature: "3044022023fc95d7283b96fdedfe7a7c1073f0d5f4a0ce4fb3ca4940c2a6d47ca2236033022033201ce70e10de6a01f4a12d36e92ae62d0b25a7a40da5d32f879c116ba0c2d2",
    },
    TestInput {
        public_key: "02af493b8a7940f7f1580c2b5fed5a90bb0c131ca4eee3a453416f46d21ad60bf6",
        _private_key: "4115262f24cd7c6d674ed17ac9538a3c7cbfc4391b85fa674e45ed65e8f687a9",
        data: "3343bd7649655d2cacd889e4fb620b0a386f43925dae",
        signature: "3046022100d08b2d1d37c625f1f8022d38ef13ed78694dd0cc3cb19e1017bf8f39f92961a2022100ae60e66603a11487bd8d1c314fd47ac91b292f251e9ba33f825c0dc496924be4",
    },
    TestInput {
        public_key: "02327d44e810121b390fbec18ef203acb0975be6b7961b6597be6cccf52e105513",
        _private_key: "10e11a836ec4709e8fec59265c025b24b0748f53bb839441a33f383bd804b050",
        data: "4a516c6ae3bd5d5f5811585572a4b07d8e93",
        signature: "304502202114844a03bf832640487acdb62bb7730eec150be51447593f7a66224b04ce5f022100cc7f2501c2a75d71f289c73607dd828a89b4f5e11f19589e0fe184b2eff22947",
    },
    TestInput {
        public_key: "03407ef29f05e5bc26220ae32d0d7fda2ef94d9a2f001744a0362765626faede2b",
        _private_key: "add032872b57aa957dc4d8f5b76a1971c63175d93b758b7a3f1838e9a27ad1af",
        data: "43eb42d1ab3ac46fc933e21d677c5b228ff3f3fcb6",
        signature: "3046022100fb32798e1a023fa2e8bdcc8a58770372a79cc0566c354f45a8520265dd58cf34022100f4eaab9354f7d80f1515531b5a1cb3a3b722f096f24dde0331899a37b77fe85e",
    },
    TestInput {
        public_key: "03ef0b5f7b167c770e53c26f9ce0593e41168e307503f418d41a98948a1b88f16b",
        _private_key: "7cff6a499533df1ab5560acb5d94d7f445883022aa1f6efd8c29fa7712da809d",
        data: "98dfc7b2473392057c6020dcc4b2ef7c68474b5788bb555de693a5",
        signature: "3046022100fc4252d4cdbe4e0470f8fc29bb7d32d80713f582b0e1a5d338c8b03de0d7a0da0221008d56beea3b68b9b65d2cb19ecd856d057d0c60dfb732e554d4dd31303e064158",
    },
    TestInput {
        public_key: "021ff1182f20929374355b6a35f2a4707bb78a052ef6d134fb9ce063979c72f0e8",
        _private_key: "c021dc6d05a314c6d5cefeb6441b7a91340aa33efa5285087dcebb2c085d6e7b",
        data: "1159598e30cff4",
        signature: "3046022100bdf6db8ea1108b7027f12e33bc02f2caf56afc206c09efa3a94df038088ed786022100d22298a6e2cd64a8f12dcd35382273525725f583b7fe3d05f33003238c7e5e8f",
    },
    TestInput {
        public_key: "031776f58bb1626a7f7d17f19c9ac861226e907aba53fe6661a242e69594de85d4",
        _private_key: "8199320a51fc7c78ed433013b32e15dfd8ca68ba95b73c3fda3f04c984efcc02",
        data: "f269dc1dd8e5983ad4734d6757a422099e",
        signature: "3045022100e6c630074b4913e40db01b6df329400abb1ae905ce392bd2c2517633ce217a33022025c41a1762d96a3572ceb8a262af1081330ff897953de31d6eab18adfea5b9d9",
    },
    TestInput {
        public_key: "03df832c689a34534a499ea817a8f28a17b8761eeed3cedf795d240013bef115ce",
        _private_key: "0192a4c25325e7a95fb96ee61eefb5026868ea1a499e8f9c8eccda04e81b4a10",
        data: "f89d62a52aab9e30609884676821",
        signature: "304402207731ab999e07d14431a0e42ef1274607a7a0eb888a883568ef8d0825c933d75702202ae4096abbc2a4fd079ea66d14095051332304ee721d2c2b357de723abe0914d",
    },
    TestInput {
        public_key: "020b78dcdbdf7a67ef85131ecee3b89ae3eb285cff1efd153806fd216caa0bc26a",
        _private_key: "00b0945219568ca61fae049a77420d9389f28a225c70840e8ecd5fcf223b9de5",
        data: "c2ce3f",
        signature: "3045022056d2de55da1f5c2758d3dd0e9647a34e479ac0e61343e6f912e3669f078f9daa022100f212f759ac98d142ee8493f87d7a4d5785dccf252add8b372be3f32623200f18",
    },
    TestInput {
        public_key: "025e4730a5c0921731987e18d504ab0ebe17e0c788407611d4022a0ffc718ce404",
        _private_key: "3f2bb270294493f2fe1e94f434a79c79f35593f556190f90878e947499229abd",
        data: "ddd82c334e27e6c0999589",
        signature: "30450221009eb04a98b50fb3419ad7d5a429e23be06905b93834daa1b141b543700d8c012a02203dfbd110655ab5f5776ddd686bc7a1482c0c98ec87ea7c81efc6a4afd4a4c74e",
    },
    TestInput {
        public_key: "02809c15e681190bab0e955877e839cb8a0011c2dc696617299c0b63a43c437053",
        _private_key: "92dc854aad3669b2aa55be04110c577e600d3eb5cca5267e2cd54105b1fda7ec",
        data: "10",
        signature: "3045022072aefed285b9450262364a64762de0999344e53b67d6763878e050e108e76344022100cc4bb5949fe5a333ccc42a901b2e70ebb80cba68c6de5a8f1108595282e4106a",
    },
    TestInput {
        public_key: "020d22e7de2551bc343277246f6cfc73c31dc21ff33ea8b7cfb185b560908a1d47",
        _private_key: "ff41770ae2a328174e8e53e7c737b27a570b59b5324771da73037226b1bdcf6b",
        data: "c1ff9b49291dd5cc3e627a7e147ebd7a08e2ac57cb",
        signature: "30440220788f775677c063bcc22a3271bdd3a9a423584b87dab4c52f881c2d7df36f89fd02201a5edfd0a0efa06503eba4351f1decc1b5dce15ef4d5869aef76f5e4ea305148",
    },
    TestInput {
        public_key: "02a5196e147210027dc644544b45df692b2643e16a6ff5bb3d0a42c0a1a23cdc81",
        _private_key: "8d9ca03f0eefba53f8afe9b453696347e2b21141c0955956ef102d31a313026f",
        data: "22e96f58574f191b754fff1501ad6a71e2f2c4745ab4d98903800bb3d8ab6c",
        signature: "3046022100a5e8e79f368d683f952bea8627a3fbe411e110b2fab42bfe507300c163d8813f022100d5f68e36d3cd37b6ba16c493f988e3f33c0e48d60d9459f33c6d60ba84460862",
    },
    TestInput {
        public_key: "033fca0e582fca02c7c4afe97ce582592e978ad85fc4f7e75534a6f841c5d9f893",
        _private_key: "0d7fef5de41c2e6e2ec715c68ce1660a93c80e19137239ccc1da6cc1883c5034",
        data: "6589a9a397d4b88af7422dba8196959595bedd77f1e700c8ba9121",
        signature: "30450220044df23de60f24aba40d39c72ed5cf37021408ea0ce316e072c9aaacb39b8c13022100cffb8a2889e60918caa3c065aa04ecf56d5ade73a44f7b845046bb5ffd185b0c",
    },
    TestInput {
        public_key: "0214a6c938517c424018f5c0538f14c220bf48e79c4d4bfd36ae73a6f600c60074",
        _private_key: "aca0ae92ee36ed28cf7ab0c10464ead04816098322cb3ddc0f78cf51d1ccdfba",
        data: "3646f36c04d0fe01f66a9521728c",
        signature: "304402202411573b0656b90bad369ea05b69866b0b7834adff9f1726af4e99ff3f67a6ba0220511eea49cbd6ebd3e57f7d8b30a1174c1ce4355d42955169a8be076efaac3f89",
    },
    TestInput {
        public_key: "024df3c641960d555b130cff5db8a7bc51acf4f16434b75462efc2582b3dd4be85",
        _private_key: "e6865ced0c1330a09b7c0378321de9a22c6670a34dbbb2c128a01ee4123372aa",
        data: "c517f53b392c1bc83606376bf492517c4502d99955458b5f652dd81794",
        signature: "3045022100b70a2586b76b56c45712eae944e419822b0ece808e884b849613fceaf29e746a02203e3fa765df4eb70e8e625e72f3d35561d96a8e2e07579bc6e7adcd7f4a37fefd",
    },
    TestInput {
        public_key: "0242c4aa466b5d222f3d0b0de78156ab2c9082b201d06fee5180bc56231ab33d94",
        _private_key: "e0be47beb97d96c6271be964b0660f90d7827f81362d44ebe4c92d4c00a70b05",
        data: "1f34f679ad6d144d302af05e79d7205074f8",
        signature: "304502201a5a1cb22e8a3b68d4c04417412c75f1f5d48e25f2a85ae7449324f51afdcf79022100ea1b26ed808af0ec4f73286c5d1a4ef6f45ca0ebd78c3bbfc18b2745e65d35cd",
    },
    TestInput {
        public_key: "035d7de2b10ca92de612138a8cbd7244d50afaae55003ff6e19b408ac4ccbe5dd3",
        _private_key: "b34319776d92d0b550d16be5c221151d81dc2837854584de5f31a6d126746ea1",
        data: "970ffecfff8ddb75",
        signature: "3045022016d9c1b9bd23339ee8ba4d5b17a92b9a8522a03f72e59c241584419e9ec5ad170221009b19f711f3f7c0a8c7a9f743079e3a01ef5005eccf57a7b9f77f1a86c31e4924",
    },
    TestInput {
        public_key: "02c11800de3ebcf9c542049b105cec5c049b620af3c0223c9441b2804404d48922",
        _private_key: "89b25a30ca137fc81036d0bbeca795f568009e66676b3d53201359a80a85336f",
        data: "4d7055505e3c",
        signature: "304402204d7b2c809c4f029e2d6b8bc8b90778bb0f42b2d5ab741f9317c047e4ff7e7ad102206b7da13955d3bc034eef4e97f9e97d43e7374be6fb44c0668e5d0c7a943edf71",
    },
    TestInput {
        public_key: "0205fe2ab3963918fb728f06c76413e0ac08fe352b7e0c32d6e0861fa79b764ed1",
        _private_key: "70cc17d0ec1f6f5007468681d934f14f56c79a34c9be5725330bce4d8755661e",
        data: "32ed02f0235c1422044afa2d6844c5e0235f380aba033b",
        signature: "304402201b323994477164a267fecb69ffd6a3ea63ee3c01648bdd639e224ece2ac3f51202206e38a3c8239114dff4dc0a6e1ac0717657a362a410e98984461661c78b435cd0",
    },
    TestInput {
        public_key: "034e2ad47b1ab64f27e5caf914c4c789da94cdaf51ab74487d9a6a8d640de614c1",
        _private_key: "29c1f03f85a96195d5f4355d7c33071e64cadca59b9e3b0366cf1c25936a6110",
        data: "11fcf2f5751ba64bd83be54514",
        signature: "30440220680ee3489736f3494bd11c944d51757e137b60955fed3934c6b214ef5a7c6b0c02201c79f1da2d0333442d8998d49e12aba9d3072101fe42b0b35de58b07bfa0b796",
    },
    TestInput {
        public_key: "03a3d3642f4a218278a5a22b6c59c5c6551e0afc1efbe3c5be2d60f3bb07653390",
        _private_key: "8c6b7990d2372ee190455cf86cc5591727d5380b19230a19d903298ea27a7e4a",
        data: "2b8193708c31",
        signature: "3045022076de9c7e5234ffa158662186f48aaf308b5cb10b55c0e6172e4f34d0429c9ca70221009fbe8b7d126e1c787828ddc51dfc7b8243dfc5ea877c7339d5212dd50fa84cc3",
    },
    TestInput {
        public_key: "02502991121053f932ca83629b90d0e6428300361a3db40e473f624af2db013b25",
        _private_key: "17cf61cc400f6f46d85fd6cb8d569b8361898b7c7c57076f3514a80dffa104e5",
        data: "156092baa666ff43615f96782797ce41d1c520",
        signature: "3044022039ae7f6ae0f567e337d97575a630803447842f478c2cfec005b6c4b86777703602202ad47c286258141d10ebbbef3547ed465bcb9dd42223c18cfd3351d57555a0d3",
    },
    TestInput {
        public_key: "0252f1f61f2ed6c0dc2a2dd8dc477e8c37a35eab6d14c94238e1ac6eb1774ead4f",
        _private_key: "8d86e83acacce5580e7ba5fab7b1ef90b07aad9f8425b8d8d8d9c5bf759fb97c",
        data: "d1214a30fa539b1c49dd385b32",
        signature: "3046022100b9c0756e6ccdbbda4f4d9853362c638c88c6e1d1399c057980c1709ed89f644d02210082bba757ecb29d6f3d4703c718389646c4d176eb6fb33e1f6b04399428c00d6d",
    },
    TestInput {
        public_key: "03e79fed143dec70d18ce101015d4402fa3fa70bced3892854427a1df646142ade",
        _private_key: "1dbceb07db7f6847d16f092259e9424ce5764bcf78d4cea943429951ae80f171",
        data: "3daecf8a4ce732110da3ec0789f5cfd8c955fd153003e48d",
        signature: "304502206d164be29709cd2cef1d0d7d7f216cfd16d48ff1fb0cd25a39d2d8061a2e2a420221009dd261934dc194c865a2fcf22d7e2b7e80b403cd5793d79bbda7f1eefd0009bb",
    },
    TestInput {
        public_key: "03d6915116866f308b9c386ef9065a9a3be3d84f860ee6910cda32a070403bf12a",
        _private_key: "678d19c6f7188c90634929533a57b5cc1c5d5890cd24b8dd6ff1124ee498dc76",
        data: "05e7f6e57ee6fc59",
        signature: "3044022022b387bc6bd817ba8b7678a2f2d7407863f7270d670f34563904fe270b64490602201475444356d3a96969a66d3fb3d22c0b6666d7443f58ec2f57d9bfa8863c097d",
    },
    TestInput {
        public_key: "034be48a452a2399542306cef31ed61d69136cf71af5b8aced080c213fceb4c7e3",
        _private_key: "ab0ba2449c0a81722972c9e228566e6da899c8d7c8119e08a480a4feddae0c7c",
        data: "ea9dec5729710760ab767790dff3",
        signature: "304502210099a7d8dbf0b610e7a4c4acb5a7bf432fd4cdd79cabb8de369194a4074080604702201f6d3e45d7765be344b622f53495821c14dd5150d3bbff7b4b37241ec49a8821",
    },
    TestInput {
        public_key: "024db5e63fb022543e378be3433e8d3300be01a7dc1b13d41e41cf954e6828caa8",
        _private_key: "acf4db496ba3d7af2b6a51d23a16ca4f31ba892de260d5e76369e09c8fcafb72",
        data: "a70a915cb32710d63aec0c3b2c2f49c219775dc4ec9c",
        signature: "3045022016e85fc22a190b455b5226d1c08ad0f215896a7c2b2db0ed8eedf9c35ae13479022100faf8aade3145665e3e51945ab8a24d81974a2aa1af6d344d9925c434290c7107",
    },
];

struct TestInput {
    public_key: &'static str,
    _private_key: &'static str,
    data: &'static str,
    signature: &'static str,
}

impl TestInput {
    pub fn public_key(&self) -> Secp256k1PublicKey {
        let bytes = HexStr::from_str(self.public_key).unwrap().to_bytes();
        bytes.to_public_key().unwrap()
    }

    pub fn signature(&self) -> Vec<u8> {
        HexStr::from_str(self.signature).unwrap().to_bytes().to_vec()
    }

    pub fn data(&self) -> Vec<u8> {
        HexStr::from_str(self.data).unwrap().to_bytes().to_vec()
    }
}

#[test]
fn test_secp256k1_apollo_test_cases() {
    for ti in TEST_VECTOR {
        let public_key = ti.public_key();
        let data = ti.data();
        let signature = ti.signature();
        let is_valid = public_key.verify(&data, &signature);
        assert!(is_valid)
    }
}
