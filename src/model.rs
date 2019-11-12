use std::path::Path;

use vulkan_ray_tracing::geometry_instance::{ImageBuffer, Material, Vertex};
use vulkan_ray_tracing::glm;

pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub materials: Vec<Material>,
    pub textures: Vec<ImageBuffer>,
}

impl Model {
    pub fn new(filename: &Path) -> Model {
        let (models, mats) = tobj::load_obj(filename).expect("Cannot load model");

        let mut indices = vec![];
        let mut vertices = vec![];
        let mut materials = vec![];
        let mut textures = vec![];

        for mat in mats.iter() {
            let mut texture_id = -1;
            if !mat.diffuse_texture.is_empty() {
                let texture = Self::load_texture(&mat.diffuse_texture);
                textures.push(texture);
                texture_id = textures.len() as i32 - 1;
            }

            let material = Material {
                ambient: glm::make_vec3(&mat.ambient),
                diffuse: glm::make_vec3(&mat.diffuse),
                specular: glm::make_vec3(&mat.specular),
                dissolve: mat.dissolve,
                ior: mat.optical_density,
                illum: mat.illumination_model.unwrap_or(0) as i32,
                texture_id,
                ..Material::default()
            };
            materials.push(material);
        }

        if materials.is_empty() {
            materials.push(Material::default());
        }

        for model in models.iter() {
            let current_indices: Vec<u32> = model
                .mesh
                .indices
                .iter()
                .map(|x| x + vertices.len() as u32)
                .collect();
            indices.extend_from_slice(&current_indices);

            vertices.reserve(model.mesh.positions.len() / 3);
            for v in 0..model.mesh.positions.len() / 3 {
                let tex_coord = if model.mesh.texcoords.is_empty() {
                    glm::vec2(0.0, 1.0)
                } else {
                    glm::vec2(
                        model.mesh.texcoords[2 * v],
                        1.0 - model.mesh.texcoords[2 * v + 1],
                    )
                };

                let vertex = Vertex {
                    pos: glm::vec3(
                        model.mesh.positions[3 * v],
                        model.mesh.positions[3 * v + 1],
                        model.mesh.positions[3 * v + 2],
                    ),
                    nrm: glm::vec3(
                        model.mesh.normals[3 * v],
                        model.mesh.normals[3 * v + 1],
                        model.mesh.normals[3 * v + 2],
                    ),
                    color: glm::vec3(1.0, 1.0, 1.0),
                    tex_coord,
                    mat_id: model.mesh.material_id.unwrap_or(0) as i32,
                };

                vertices.push(vertex);
            }
        }

        Model {
            vertices,
            indices,
            materials,
            textures,
        }
    }

    fn load_texture(filename: &str) -> ImageBuffer {
        let path = Path::new("assets/textures/").join(filename);
        let image = image::open(path).unwrap().to_rgba();
        let width = image.width();
        let height = image.height();

        ImageBuffer {
            pixels: image.into_raw(),
            tex_width: width,
            tex_height: height,
            tex_channels: 1,
        }
    }
}
