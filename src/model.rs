use std::path::Path;

use vulkan_helpers::images::Image;
use vulkan_helpers::material::Material;
use vulkan_helpers::vertex::Vertex;

pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub materials: Vec<Material>,
    pub textures: Vec<Image>,
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
                ambient: mat.ambient,
                diffuse: mat.diffuse,
                specular: mat.specular,
                dissolve: mat.dissolve,
                ior: mat.optical_density,
                illum: mat.illumination_model.unwrap_or(0) as i32,
                texture_id,
                ..Material::default()
            };
            println!(
                "Diffuse material [{} {} {}]",
                material.diffuse[0], material.diffuse[1], material.diffuse[2]
            );
            materials.push(material);
        }

        if materials.is_empty() {
            materials.push(Material::default());
        }

        for model in models.iter() {
            indices.reserve(model.mesh.indices.len());
            indices.extend(model.mesh.indices.iter());
            vertices.reserve(model.mesh.positions.len() / 3);

            for v in 0..model.mesh.positions.len() / 3 {
                let vertex = Vertex {
                    pos: [
                        model.mesh.positions[3 * v],
                        model.mesh.positions[3 * v + 1],
                        model.mesh.positions[3 * v + 2],
                    ],
                    nrm: [
                        model.mesh.normals[3 * v],
                        model.mesh.normals[3 * v + 1],
                        model.mesh.normals[3 * v + 2],
                    ],
                    color: [0.0, 1.0, 1.0],
                    tex_coord: [
                        model.mesh.texcoords[2 * v],
                        1.0 - model.mesh.texcoords[2 * v + 1],
                    ],
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

    fn load_texture(filename: &str) -> Image {
        let path = Path::new(filename);
        let image = image::open(path).unwrap().to_rgba();
        let width = image.width();
        let height = image.height();

        Image {
            pixels: image.into_raw(),
            tex_width: width,
            tex_height: height,
            tex_channels: 1,
        }
    }
}
