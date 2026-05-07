/// Material properties for ASCII PBR rendering
/// Simulates metalness and roughness in terminal characters

#[derive(Debug, Clone, Copy)]
pub struct Material {
    /// 0.0 = non-metal (dielectric), 1.0 = fully metallic
    pub metalness: f32,
    /// 0.0 = mirror-smooth, 1.0 = fully rough
    pub roughness: f32,
    /// Base reflectance (0.0-1.0)
    pub reflectance: f32,
}

impl Material {
    pub const fn new(metalness: f32, roughness: f32, reflectance: f32) -> Self {
        Self { metalness, roughness, reflectance }
    }
}

/// Material LUT indexed by BlockType ordinal
pub const MATERIAL_LUT: [Material; 20] = [
    Material::new(0.0, 1.0, 0.0),   // Air
    Material::new(0.0, 0.9, 0.2),   // Grass
    Material::new(0.0, 1.0, 0.1),   // Dirt
    Material::new(0.0, 0.7, 0.3),   // Stone
    Material::new(0.0, 0.8, 0.4),   // Sand
    Material::new(0.0, 0.1, 0.9),   // Water (high reflectance)
    Material::new(0.0, 0.8, 0.2),   // Wood
    Material::new(0.0, 0.9, 0.1),   // Leaves
    Material::new(0.0, 0.6, 0.3),   // Flower
    Material::new(0.0, 0.9, 0.1),   // TallGrass
    Material::new(0.0, 1.0, 0.0),   // CaveAir
    Material::new(0.8, 0.3, 0.7),   // RedstoneDust (metallic)
    Material::new(0.6, 0.4, 0.6),   // RedstoneTorch
    Material::new(0.7, 0.5, 0.5),   // Lever
    Material::new(0.3, 0.2, 0.8),   // RedstoneLamp (glowy)
    Material::new(0.0, 0.9, 0.1),   // Netherrack
    Material::new(0.0, 0.6, 0.3),   // NetherBrick
    Material::new(0.0, 0.4, 0.5),   // Obsidian (smooth, dark)
    Material::new(0.0, 0.0, 1.0),   // Portal (fully reflective)
    Material::new(0.0, 0.2, 0.9),   // Lava (high emissive reflectance)
];

#[inline(always)]
pub fn get_material(block_type: crate::block::BlockType) -> Material {
    let idx = block_type as usize;
    if idx < MATERIAL_LUT.len() {
        MATERIAL_LUT[idx]
    } else {
        Material::new(0.0, 1.0, 0.1)
    }
}

/// Apply PBR-inspired lighting to a color based on material properties
/// In terminal context: roughness affects character choice, metalness affects color tint
#[inline(always)]
pub fn apply_pbr(
    r: u8, g: u8, b: u8,
    material: Material,
    face_light: f64,
    day_brightness: f64,
) -> (u8, u8, u8) {
    // Metallic surfaces tint toward their base color more strongly
    let metal_boost = 1.0 + material.metalness as f64 * 0.3;

    // Rough surfaces diffuse light more evenly
    let diffuse = if material.roughness > 0.5 {
        face_light * 0.8 + 0.2 // more ambient
    } else {
        face_light // more directional
    };

    // High reflectance surfaces get specular highlights from sky
    let specular = material.reflectance as f64 * day_brightness * 0.2;

    let brightness = (diffuse * day_brightness * metal_boost + specular).min(1.0);

    (
        (r as f64 * brightness).min(255.0) as u8,
        (g as f64 * brightness).min(255.0) as u8,
        (b as f64 * brightness).min(255.0) as u8,
    )
}

/// Select character variant based on material roughness
/// Rough surfaces use textured chars, smooth surfaces use solid chars
#[inline(always)]
pub fn pbr_glyph(base_glyph: char, roughness: f32) -> char {
    if roughness > 0.8 {
        base_glyph // rough: keep texture
    } else if roughness > 0.4 {
        '▓' // medium: semi-solid
    } else {
        '█' // smooth: solid block
    }
}
