#[doc = r" Loads the shader as a `ShaderModule`."]
#[allow(unsafe_code)]
#[inline]
pub fn load(
    device: ::std::sync::Arc<::vulkano::device::Device>,
) -> ::std::result::Result<
    ::std::sync::Arc<::vulkano::shader::ShaderModule>,
    ::vulkano::Validated<::vulkano::VulkanError>,
> {
    let _bytes = ::std::include_bytes!("/home/dirons/src/rust/playground/src/shader.vert.glsl");
    static WORDS: &[u32] = &[
        119734787u32,
        65536u32,
        851978u32,
        44u32,
        0u32,
        131089u32,
        1u32,
        393227u32,
        1u32,
        1280527431u32,
        1685353262u32,
        808793134u32,
        0u32,
        196622u32,
        0u32,
        1u32,
        655375u32,
        0u32,
        4u32,
        1852399981u32,
        0u32,
        21u32,
        34u32,
        40u32,
        41u32,
        43u32,
        196611u32,
        2u32,
        460u32,
        655364u32,
        1197427783u32,
        1279741775u32,
        1885560645u32,
        1953718128u32,
        1600482425u32,
        1701734764u32,
        1919509599u32,
        1769235301u32,
        25974u32,
        524292u32,
        1197427783u32,
        1279741775u32,
        1852399429u32,
        1685417059u32,
        1768185701u32,
        1952671090u32,
        6649449u32,
        262149u32,
        4u32,
        1852399981u32,
        0u32,
        196613u32,
        9u32,
        116u32,
        327685u32,
        11u32,
        1936617315u32,
        1953390964u32,
        115u32,
        327686u32,
        11u32,
        0u32,
        1635017060u32,
        0u32,
        458758u32,
        11u32,
        1u32,
        1684956530u32,
        1834971749u32,
        1769108577u32,
        120u32,
        196613u32,
        13u32,
        25456u32,
        327685u32,
        21u32,
        1769172848u32,
        1852795252u32,
        0u32,
        393221u32,
        32u32,
        1348430951u32,
        1700164197u32,
        2019914866u32,
        0u32,
        393222u32,
        32u32,
        0u32,
        1348430951u32,
        1953067887u32,
        7237481u32,
        458758u32,
        32u32,
        1u32,
        1348430951u32,
        1953393007u32,
        1702521171u32,
        0u32,
        458758u32,
        32u32,
        2u32,
        1130327143u32,
        1148217708u32,
        1635021673u32,
        6644590u32,
        458758u32,
        32u32,
        3u32,
        1130327143u32,
        1147956341u32,
        1635021673u32,
        6644590u32,
        196613u32,
        34u32,
        0u32,
        327685u32,
        40u32,
        1601467759u32,
        1869377379u32,
        114u32,
        262149u32,
        41u32,
        1869377379u32,
        114u32,
        262149u32,
        43u32,
        1836216174u32,
        27745u32,
        327752u32,
        11u32,
        0u32,
        35u32,
        0u32,
        262216u32,
        11u32,
        1u32,
        5u32,
        327752u32,
        11u32,
        1u32,
        35u32,
        16u32,
        327752u32,
        11u32,
        1u32,
        7u32,
        16u32,
        196679u32,
        11u32,
        2u32,
        262215u32,
        21u32,
        30u32,
        0u32,
        327752u32,
        32u32,
        0u32,
        11u32,
        0u32,
        327752u32,
        32u32,
        1u32,
        11u32,
        1u32,
        327752u32,
        32u32,
        2u32,
        11u32,
        3u32,
        327752u32,
        32u32,
        3u32,
        11u32,
        4u32,
        196679u32,
        32u32,
        2u32,
        262215u32,
        40u32,
        30u32,
        0u32,
        262215u32,
        41u32,
        30u32,
        2u32,
        262215u32,
        43u32,
        30u32,
        1u32,
        131091u32,
        2u32,
        196641u32,
        3u32,
        2u32,
        196630u32,
        6u32,
        32u32,
        262167u32,
        7u32,
        6u32,
        4u32,
        262176u32,
        8u32,
        7u32,
        7u32,
        262168u32,
        10u32,
        7u32,
        4u32,
        262174u32,
        11u32,
        7u32,
        10u32,
        262176u32,
        12u32,
        9u32,
        11u32,
        262203u32,
        12u32,
        13u32,
        9u32,
        262165u32,
        14u32,
        32u32,
        1u32,
        262187u32,
        14u32,
        15u32,
        1u32,
        262176u32,
        16u32,
        9u32,
        10u32,
        262167u32,
        19u32,
        6u32,
        3u32,
        262176u32,
        20u32,
        1u32,
        19u32,
        262203u32,
        20u32,
        21u32,
        1u32,
        262187u32,
        6u32,
        23u32,
        1065353216u32,
        262165u32,
        29u32,
        32u32,
        0u32,
        262187u32,
        29u32,
        30u32,
        1u32,
        262172u32,
        31u32,
        6u32,
        30u32,
        393246u32,
        32u32,
        7u32,
        6u32,
        31u32,
        31u32,
        262176u32,
        33u32,
        3u32,
        32u32,
        262203u32,
        33u32,
        34u32,
        3u32,
        262187u32,
        14u32,
        35u32,
        0u32,
        262176u32,
        37u32,
        3u32,
        7u32,
        262176u32,
        39u32,
        3u32,
        19u32,
        262203u32,
        39u32,
        40u32,
        3u32,
        262203u32,
        20u32,
        41u32,
        1u32,
        262203u32,
        20u32,
        43u32,
        1u32,
        327734u32,
        2u32,
        4u32,
        0u32,
        3u32,
        131320u32,
        5u32,
        262203u32,
        8u32,
        9u32,
        7u32,
        327745u32,
        16u32,
        17u32,
        13u32,
        15u32,
        262205u32,
        10u32,
        18u32,
        17u32,
        262205u32,
        19u32,
        22u32,
        21u32,
        327761u32,
        6u32,
        24u32,
        22u32,
        0u32,
        327761u32,
        6u32,
        25u32,
        22u32,
        1u32,
        327761u32,
        6u32,
        26u32,
        22u32,
        2u32,
        458832u32,
        7u32,
        27u32,
        24u32,
        25u32,
        26u32,
        23u32,
        327825u32,
        7u32,
        28u32,
        18u32,
        27u32,
        196670u32,
        9u32,
        28u32,
        262205u32,
        7u32,
        36u32,
        9u32,
        327745u32,
        37u32,
        38u32,
        34u32,
        35u32,
        196670u32,
        38u32,
        36u32,
        262205u32,
        19u32,
        42u32,
        41u32,
        196670u32,
        40u32,
        42u32,
        65789u32,
        65592u32,
    ];
    unsafe {
        ::vulkano::shader::ShaderModule::new(
            device,
            ::vulkano::shader::ShaderModuleCreateInfo::new(&WORDS),
        )
    }
}
#[allow(non_camel_case_types, non_snake_case)]
#[derive(
    :: vulkano :: buffer :: BufferContents,
    :: std :: clone :: Clone,
    ::
std :: marker :: Copy,
)]
#[repr(C)]
pub struct constants {
    pub data: [f32; 4usize],
    pub render_matrix: [[f32; 4usize]; 4usize],
}
