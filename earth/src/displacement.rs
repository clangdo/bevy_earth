//! This material is adapted from the bevy [StandardMaterial] source
//! code (at version 0.9.1). (Licensed under the MIT license which is
//! included below.) This file provides very similar functionality so
//! that we can use the standard material fragment shader with vertex
//! displacement.

/*
MIT License

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
 */

use bevy::{
    pbr::{
        PBR_SHADER_HANDLE,
        StandardMaterialFlags,
        StandardMaterialUniform,
        MaterialPipeline,
        MaterialPipelineKey,
    },
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayout,
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup,
            AsBindGroupShaderType,
            Face,
            RenderPipelineDescriptor,
            ShaderRef,
            SpecializedMeshPipelineError,
            TextureFormat,
        },
    },
    reflect::TypeUuid,
};

#[derive(Clone, Debug)]
pub struct TextureOption {
    pub color: Color,
    pub texture: Option<Handle<Image>>,
}

impl TextureOption {
    pub fn texture_only(texture: Handle<Image>) -> Self {
        Self {
            color: Color::from(Vec4::splat(1.0)),
            texture: Some(texture),
        }
    }

    pub fn color_only(color: Color) -> Self {
        Self {
            color,
            texture: None,
        }
    }

    pub fn value_only(value: f32) -> Self {
        Self {
            color: Color::from(Vec4::splat(value)),
            texture: None,
        }
    }

    pub fn color_multiplied_texture(color: Color, texture: Handle<Image>) -> Self {
        Self {
            color,
            texture: Some(texture),
        }
    }

    pub fn value_multiplied_texture(value: f32, texture: Handle<Image>) -> Self {
        Self {
            color: Color::from(Vec4::splat(value)),
            texture: Some(texture),
        }
    }

    pub fn get_value(&self) -> f32 {
        self.color.r()
    }
}

impl<'a> From<&'a TextureOption> for Option<&'a Handle<Image>> {
    fn from(option: &TextureOption) -> Option<&Handle<Image>> {
        option.texture.as_ref()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CullMode {
    Front,
    Back,
    None,
}

impl From<CullMode> for Option<Face> {
    fn from(cull_mode: CullMode) -> Option<Face> {
        match cull_mode {
            CullMode::Front => Some(Face::Front),
            CullMode::Back => Some(Face::Back),
            CullMode::None => None,
        }
    }
}

/// A version of the bevy [StandardMaterial] that takes a vertex
/// displacement map.
///
/// It's much the same as the [StandardMaterial] but it's been cleaned
/// up a bit at the cost of offering slightly less functionality.
/// Some specific differences are outlined below.
#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "48718392-ab22-de09-928d-194882d1ec1e"]
#[uniform(0, StandardMaterialUniform)]
#[bind_group_data(DisplacementMaterialKey)]
pub struct DisplacementMaterial {
    #[texture(1)]
    #[sampler(2)]
    pub albedo: TextureOption,
    #[texture(3)]
    #[sampler(4)]
    pub emissive: TextureOption,
    #[texture(5)]
    #[sampler(6)]
    pub metallic_roughness: TextureOption,
    pub reflectance: f32,

    /// In this material there's no way to flip Y, the map must be in
    /// OpenGL format.
    #[texture(7)]
    #[sampler(8)]
    pub tangent_normal: TextureOption,
    #[texture(9)]
    #[sampler(10)]
    pub occlusion: TextureOption,

    /// In this material cull mode of None *will* enable double sided
    /// lighting, unlike the [StandardMaterial].
    pub cull_mode: CullMode,
    pub unlit: bool,
    pub alpha_mode: AlphaMode,
    pub depth_bias: f32,
    #[texture(11)]
    #[sampler(12)]
    pub displacement: Option<Handle<Image>>,
    #[texture(13)]
    #[sampler(14)]
    pub world_normal: Option<Handle<Image>>,
    #[uniform(15)]
    pub amplitude: f32,
}

impl Default for DisplacementMaterial {
    fn default() -> Self {
        Self {
            albedo: TextureOption::color_only(Color::WHITE),
            emissive: TextureOption::color_only(Color::BLACK),
            metallic_roughness: TextureOption::color_only(Color::rgba(0.0, 0.5, 0.01, 1.0)),
            reflectance: 0.5,
            // This is not necessary, as we don't use the color to modify the normal globally.
            tangent_normal: TextureOption::color_only(Color::rgba(0.0, 0.0, 0.5, 1.0)),
            occlusion: TextureOption::color_only(Color::WHITE),
            cull_mode: CullMode::Back,
            unlit: false,
            alpha_mode: AlphaMode::Opaque,
            depth_bias: 0.0,
            displacement: None,
            world_normal: None,
            amplitude: 1.0,
        }
    }
}

impl AsBindGroupShaderType<StandardMaterialUniform> for DisplacementMaterial {
    fn as_bind_group_shader_type(&self, images: &RenderAssets<Image>) -> StandardMaterialUniform {
        let mut flags = StandardMaterialFlags::from(self);

        if let Some(handle) = &self.tangent_normal.texture {
            if let Some(image) = images.get(handle) {
                update_normal_flags(&mut flags, image.texture_format);
            }
        }
        
        let alpha_cutoff = match self.alpha_mode {
            AlphaMode::Mask(cutoff) => cutoff,
            _ => 0.5,
        };

        StandardMaterialUniform {
            base_color: self.albedo.color.as_linear_rgba_f32().into(),
            emissive: self.emissive.color.into(),
            roughness: self.metallic_roughness.color.g(),
            metallic: self.metallic_roughness.color.b(),
            reflectance: self.reflectance,
            flags: flags.bits(),
            alpha_cutoff,
        }
    }
}

impl From<&DisplacementMaterial> for StandardMaterialFlags {
    fn from(material: &DisplacementMaterial) -> StandardMaterialFlags {
        let mut flags = StandardMaterialFlags::NONE;
        if material.albedo.texture.is_some() {
            flags |= StandardMaterialFlags::BASE_COLOR_TEXTURE;
        }

        if material.emissive.texture.is_some() {
            flags |= StandardMaterialFlags::EMISSIVE_TEXTURE;
        }

        if material.metallic_roughness.texture.is_some() {
            flags |= StandardMaterialFlags::METALLIC_ROUGHNESS_TEXTURE;
        }

        if material.occlusion.texture.is_some() {
            flags |= StandardMaterialFlags::OCCLUSION_TEXTURE;
        }

        if matches!(material.cull_mode, CullMode::None) {
            flags |= StandardMaterialFlags::DOUBLE_SIDED;
        }

        if material.unlit {
            flags |= StandardMaterialFlags::UNLIT;
        }



        flags |= match material.alpha_mode {
            AlphaMode::Opaque => StandardMaterialFlags::ALPHA_MODE_OPAQUE,
            AlphaMode::Mask(_) => StandardMaterialFlags::ALPHA_MODE_MASK,
            AlphaMode::Blend => StandardMaterialFlags::ALPHA_MODE_BLEND,
            AlphaMode::Premultiplied => StandardMaterialFlags::ALPHA_MODE_PREMULTIPLIED,
            AlphaMode::Add => StandardMaterialFlags::ALPHA_MODE_ADD,
            AlphaMode::Multiply => StandardMaterialFlags::ALPHA_MODE_MULTIPLY,
        };

        flags
    }
}

// This function is yanked from as_bind_group_shader_type function in the Bevy StandardMaterial source code.
fn update_normal_flags(flags: &mut StandardMaterialFlags, texture_format: TextureFormat) {
    match texture_format {
        // All 2-component unorm formats
        TextureFormat::Rg8Unorm
            | TextureFormat::Rg16Unorm
            | TextureFormat::Bc5RgUnorm
            | TextureFormat::EacRg11Unorm => {
                *flags |= StandardMaterialFlags::TWO_COMPONENT_NORMAL_MAP;
            }
        _ => {}
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DisplacementMaterialKey {
    normal_map: bool,
    cull_mode: Option<Face>,
}

impl From<&DisplacementMaterial> for DisplacementMaterialKey {
    fn from(material: &DisplacementMaterial) -> DisplacementMaterialKey {
        DisplacementMaterialKey {
            normal_map: material.tangent_normal.texture.is_some(),
            cull_mode: material.cull_mode.into(),
        }
    }
}

impl Material for DisplacementMaterial {
    // This function is copied essentially wholesale from the StandardMaterial source code.
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if key.bind_group_data.normal_map {
            descriptor
                .fragment
                .as_mut()
                .unwrap()
                .shader_defs
                .push(String::from("STANDARDMATERIAL_NORMAL_MAP").into());
        }

        descriptor.primitive.cull_mode = key.bind_group_data.cull_mode;
        Ok(())
    }
    
    fn vertex_shader() -> ShaderRef {
        ShaderRef::from("shaders/displacement.wgsl")
    }

    fn fragment_shader() -> ShaderRef {
        PBR_SHADER_HANDLE.typed().into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    fn depth_bias(&self) -> f32 {
        self.depth_bias
    }
}
